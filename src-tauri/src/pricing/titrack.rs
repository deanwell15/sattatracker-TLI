//! Read-only TITrack PostgREST transport. No runtime configuration of host/key.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const ENDPOINT: &str = "https://qhjulyngunwiculnharg.supabase.co/rest/v1/aggregated_prices";
// Public client configuration, not a secret/service-role credential. Provenance:
// astockman99/TITrack 7798e02db8cfc5fb3b5eb927e3600fef99815b5e sync/client.py.
const PUBLISHABLE_KEY: &str = include_str!("titrack-publishable-key.txt");
const SELECT: &str = "config_base_id,season_id,price_fe_median,price_fe_p10,price_fe_p90,submission_count,unique_devices,updated_at";
const PAGE_SIZE: usize = 1000;
const MAX_PAGES: usize = 50;
const MAX_NETWORK_ITEMS: usize = 10_000;
const MAX_PAGE_BYTES: usize = 1024 * 1024;
const TOTAL_TIMEOUT: Duration = Duration::from_secs(45);
pub const FULL_SYNC_SECONDS: i64 = 6 * 60 * 60;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncStatistics {
    pub received: usize,
    pub accepted: usize,
    pub skipped: usize,
    pub skip_reasons: BTreeMap<String, usize>,
    // Diagnostic only, never an additional quorum requirement.
    pub low_observation_rows: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncMetadata {
    pub cursor: Option<String>,
    pub last_full_at: Option<String>,
    pub last_success_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistics: Option<SyncStatistics>,
}
#[derive(Clone)]
pub struct SyncRequest {
    pub metadata: SyncMetadata,
    pub cancellation: Option<(Arc<AtomicU64>, u64)>,
}
impl SyncRequest {
    pub fn cancelled(&self) -> bool {
        self.cancellation
            .as_ref()
            .map(|(g, expected)| g.load(Ordering::Acquire) != *expected)
            .unwrap_or(false)
    }
}
pub struct ProviderBatch {
    pub records: BTreeMap<String, PriceRecord>,
    pub sync: Option<SyncMetadata>,
}
#[derive(Clone, Debug)]
pub struct ProviderError {
    pub message: String,
    pub retry_after_seconds: Option<u64>,
    pub access_denied: bool,
    pub cancelled: bool,
}
impl ProviderError {
    fn new(message: &str) -> Self {
        Self {
            message: message.into(),
            retry_after_seconds: None,
            access_denied: false,
            cancelled: false,
        }
    }
    fn cancelled() -> Self {
        Self {
            cancelled: true,
            ..Self::new("Price refresh cancelled by source/season change")
        }
    }
}
impl From<String> for ProviderError {
    fn from(message: String) -> Self {
        Self {
            message: message.replace(PUBLISHABLE_KEY.trim(), "[redacted]"),
            ..Self::new("")
        }
    }
}

struct HttpResponse {
    bytes: Vec<u8>,
    status: u16,
    retry_after: Option<String>,
}
trait HttpTransport {
    fn get(
        &self,
        parameters: &[(String, String)],
        timeout: Duration,
    ) -> Result<HttpResponse, ProviderError>;
}
struct HttpsTransport {
    client: reqwest::blocking::Client,
    endpoint: String,
}
impl HttpsTransport {
    fn production() -> Result<Self, ProviderError> {
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .user_agent("Deanfinite/1.2.11 prices-readonly")
            .build()
            .map_err(|_| ProviderError::new("Cannot initialize TITrack HTTPS client"))?;
        Ok(Self {
            client,
            endpoint: ENDPOINT.into(),
        })
    }
    #[cfg(test)]
    fn loopback(endpoint: String, timeout: Duration) -> Self {
        assert_eq!(
            reqwest::Url::parse(&endpoint).unwrap().host_str(),
            Some("127.0.0.1")
        );
        Self {
            client: reqwest::blocking::Client::builder()
                .no_proxy()
                .timeout(timeout)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            endpoint,
        }
    }
}
impl HttpTransport for HttpsTransport {
    fn get(
        &self,
        parameters: &[(String, String)],
        timeout: Duration,
    ) -> Result<HttpResponse, ProviderError> {
        let mut url = reqwest::Url::parse(&self.endpoint)
            .map_err(|_| ProviderError::new("Invalid fixed TITrack endpoint"))?;
        url.query_pairs_mut().extend_pairs(
            parameters
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        );
        let mut response = self
            .client
            .get(url)
            .header("apikey", PUBLISHABLE_KEY.trim())
            .header("Accept", "application/json")
            .timeout(timeout)
            .send()
            .map_err(|error| {
                // Do not format reqwest errors/requests: headers/key must never leak.
                ProviderError::new(if error.is_timeout() {
                    "TITrack request timed out"
                } else {
                    "TITrack HTTPS connection failed (DNS, TLS, proxy or network)"
                })
            })?;
        let status = response.status().as_u16();
        eprintln!("[pricing] TITrack HTTPS status={status}");
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .map(str::to_owned);
        if !(200..300).contains(&status) {
            return Ok(HttpResponse {
                bytes: Vec::new(),
                status,
                retry_after,
            });
        }
        if response
            .content_length()
            .is_some_and(|size| size > MAX_PAGE_BYTES as u64)
        {
            return Err(ProviderError::new("TITrack response exceeds 1 MiB"));
        }
        let mut bytes = Vec::new();
        (&mut response)
            .take((MAX_PAGE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| ProviderError::new("Cannot read bounded TITrack response"))?;
        if bytes.len() > MAX_PAGE_BYTES {
            return Err(ProviderError::new("TITrack response exceeds 1 MiB"));
        }
        Ok(HttpResponse {
            bytes,
            status,
            retry_after,
        })
    }
}
fn retry_after(value: Option<&str>, now: i64) -> Option<u64> {
    let value = value?.trim();
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(seconds);
    }
    chrono::DateTime::parse_from_rfc2822(value)
        .ok()
        .map(|date| date.timestamp().saturating_sub(now).max(0) as u64)
}
fn exact_time(value: &str) -> chrono::DateTime<chrono::FixedOffset> {
    chrono::DateTime::parse_from_rfc3339(value).expect("timestamp already validated")
}

pub struct TiTrackPriceProvider;
impl PriceProvider for TiTrackPriceProvider {
    fn load(&self, season: u32, now: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
        self.fetch(
            season,
            now,
            &SyncRequest {
                metadata: SyncMetadata::default(),
                cancellation: None,
            },
        )
        .map(|batch| batch.records)
        .map_err(|e| e.message)
    }
    fn fetch(
        &self,
        season: u32,
        now: i64,
        request: &SyncRequest,
    ) -> Result<ProviderBatch, ProviderError> {
        Self::mapped_season(season)?;
        if request.cancelled() {
            return Err(ProviderError::cancelled());
        }
        fetch_table(
            &HttpsTransport::production()?,
            season,
            now,
            request,
            PAGE_SIZE,
            MAX_PAGES,
        )
    }
}
fn fetch_table(
    transport: &dyn HttpTransport,
    season: u32,
    now: i64,
    request: &SyncRequest,
    page_size: usize,
    max_pages: usize,
) -> Result<ProviderBatch, ProviderError> {
    TiTrackPriceProvider::mapped_season(season)?;
    for time in [&request.metadata.cursor, &request.metadata.last_full_at]
        .into_iter()
        .flatten()
    {
        validate_time(time, now)?;
    }
    let deadline = Instant::now() + TOTAL_TIMEOUT;
    let mut total_bytes = 0;
    let mut get_rows =
        |mut query: Vec<(String, String)>| -> Result<Vec<serde_json::Value>, ProviderError> {
            if request.cancelled() {
                return Err(ProviderError::cancelled());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(ProviderError::new("TITrack sync exceeded 45 seconds"));
            }
            query.push(("select".into(), SELECT.into()));
            query.push(("season_id".into(), format!("eq.{season}")));
            let response = transport.get(&query, remaining.min(Duration::from_secs(10)))?;
            if !(200..300).contains(&response.status) {
                let mut error = ProviderError::new(&format!("TITrack HTTP {}", response.status));
                error.access_denied = matches!(response.status, 401 | 403);
                error.retry_after_seconds =
                    retry_after(response.retry_after.as_deref(), now_secs());
                return Err(error);
            }
            if request.cancelled() {
                return Err(ProviderError::cancelled());
            }
            total_bytes += response.bytes.len();
            if response.bytes.len() > MAX_PAGE_BYTES || total_bytes > MAX_BYTES {
                return Err(ProviderError::new(
                    "TITrack sync response byte limit exceeded",
                ));
            }
            let rows: Vec<serde_json::Value> = serde_json::from_slice(&response.bytes)
                .map_err(|_| ProviderError::new("Invalid TITrack JSON array"))?;
            // Validate every field/row even in the high-watermark query.
            let text = std::str::from_utf8(&response.bytes)
                .map_err(|_| ProviderError::new("Invalid TITrack UTF-8"))?;
            TiTrackPriceProvider::decode_aggregates(text, season, now)?;
            Ok(rows)
        };
    // Server-data high watermark bounds every page. Concurrent changes beyond it
    // are picked up by the next inclusive delta, even behind the item-ID cursor.
    let head = get_rows(vec![
        ("order".into(), "updated_at.desc,config_base_id.asc".into()),
        ("limit".into(), "1".into()),
    ])?;
    if head.len() > 1 {
        return Err(ProviderError::new(
            "TITrack high-watermark query returned excess rows",
        ));
    }
    let upper = head
        .first()
        .and_then(|row| row.get("updated_at"))
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    let expired_full = request
        .metadata
        .last_full_at
        .as_ref()
        .and_then(|t| parse_time(t).ok())
        .map(|t| now.saturating_sub(t) >= FULL_SYNC_SECONDS)
        .unwrap_or(true);
    let mut full = expired_full || request.metadata.cursor.is_none();
    if let (Some(previous), Some(upper)) = (&request.metadata.cursor, &upper) {
        if exact_time(upper) < exact_time(previous) {
            full = true;
        }
    }
    let lower = if full {
        None
    } else {
        request.metadata.cursor.clone()
    };
    let mut records = BTreeMap::new();
    let mut statistics = SyncStatistics::default();
    let mut after_id = 0u64;
    let mut cursor: Option<String> = None;
    let mut finished = upper.is_none();
    if let Some(upper) = &upper {
        for _ in 0..max_pages {
            let mut query = vec![
                ("order".into(), "config_base_id.asc".into()),
                ("limit".into(), page_size.to_string()),
                ("updated_at".into(), format!("lte.{upper}")),
            ];
            if after_id > 0 {
                query.push(("config_base_id".into(), format!("gt.{after_id}")));
            }
            if let Some(lower) = &lower {
                query.push(("updated_at".into(), format!("gte.{lower}")));
            }
            let rows = get_rows(query)?;
            if rows.len() > page_size {
                return Err(ProviderError::new("TITrack page exceeds requested size"));
            }
            statistics.received += rows.len();
            if statistics.received > MAX_NETWORK_ITEMS {
                return Err(ProviderError::new("TITrack record limit exceeded"));
            }
            for row in &rows {
                let devices = row["unique_devices"].as_u64();
                let submissions = row["submission_count"].as_u64();
                if devices.is_some_and(|n| n < 3)
                    || devices.zip(submissions).is_some_and(|(d, s)| s < d)
                {
                    statistics.low_observation_rows += 1;
                }
                let id = row["config_base_id"]
                    .as_u64()
                    .ok_or_else(|| ProviderError::new("Invalid TITrack item ID"))?;
                let time = row["updated_at"]
                    .as_str()
                    .ok_or_else(|| ProviderError::new("Invalid TITrack timestamp"))?;
                if id <= after_id
                    || exact_time(time) > exact_time(upper)
                    || lower
                        .as_ref()
                        .is_some_and(|lower| exact_time(time) < exact_time(lower))
                {
                    return Err(ProviderError::new(
                        "TITrack pagination order or time boundary violation",
                    ));
                }
                after_id = id;
                if cursor
                    .as_ref()
                    .map(|previous| exact_time(time) > exact_time(previous))
                    .unwrap_or(true)
                {
                    cursor = Some(time.into());
                }
            }
            records.extend(TiTrackPriceProvider::decode_aggregates(
                &serde_json::to_string(&rows)
                    .map_err(|_| ProviderError::new("Cannot decode TITrack page"))?,
                season,
                now,
            )?);
            statistics.accepted = records.len();
            if records.len() > MAX_NETWORK_ITEMS {
                return Err(ProviderError::new("TITrack record limit exceeded"));
            }
            // Finish only on an empty page: a server may cap responses below
            // our requested limit. A short non-empty page is not proof of EOF.
            if rows.is_empty() {
                finished = true;
                break;
            }
        }
    }
    if !finished {
        return Err(ProviderError::new(
            "TITrack page limit exceeded; partial sync discarded",
        ));
    }
    // Never advance to local time or the preliminary high watermark. Only data
    // actually read/applied may advance the cursor. Empty deltas retain it.
    if let Some(previous) = &request.metadata.cursor {
        if cursor
            .as_ref()
            .map(|next| exact_time(next) < exact_time(previous))
            .unwrap_or(true)
        {
            cursor = Some(previous.clone());
        }
    }
    eprintln!(
        "[pricing] TITrack validated batch: season={season}, received={}, accepted={}, skipped={}, skip_reasons={:?}, low_observation_rows={} (accepted server aggregates)",
        statistics.received, statistics.accepted, statistics.skipped,
        statistics.skip_reasons, statistics.low_observation_rows
    );
    Ok(ProviderBatch {
        records,
        sync: Some(SyncMetadata {
            cursor,
            last_full_at: if full {
                Some(timestamp(now))
            } else {
                request.metadata.last_full_at.clone()
            },
            last_success_at: Some(timestamp(now_secs())),
            statistics: Some(statistics),
        }),
    })
}

#[cfg(test)]
#[path = "titrack_tests.rs"]
mod tests;

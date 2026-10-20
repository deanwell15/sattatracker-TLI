//! Season-scoped valuations. No log-derived market observations or uploads.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub const REFRESH_SECONDS: u64 = 5 * 60;
pub const STALE_SECONDS: i64 = 24 * 60 * 60;
pub const MAX_PRICE: f64 = 1_000_000_000.0;
const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_ITEMS: usize = 50_000;
pub const ACCESS_BLOCK: &str = "TITrack is not enabled: third-party read-only permission, credentials, rate limits and live SS14 coverage have not been confirmed. Local JSON and cached prices remain available.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PriceStatus {
    Known,
    Unknown,
    Stale,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceQuote {
    pub value: Option<f64>,
    pub status: PriceStatus,
    pub source: Option<String>,
    pub updated_at: Option<String>,
    pub received_at: Option<String>,
}
impl PriceQuote {
    pub fn unknown() -> Self {
        Self {
            value: None,
            status: PriceStatus::Unknown,
            source: None,
            updated_at: None,
            received_at: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceMode {
    Automatic,
    LocalJson,
}
impl Default for SourceMode {
    fn default() -> Self {
        Self::LocalJson
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceDocument {
    pub schema_version: u32,
    pub season_id: u32,
    pub currency: String,
    pub source: String,
    pub updated_at: String,
    #[serde(deserialize_with = "unique_map")]
    pub prices: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceRecord {
    pub value: f64,
    pub source: String,
    pub updated_at: String,
    pub received_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheDocument {
    schema_version: u32,
    season_id: u32,
    currency: String,
    #[serde(deserialize_with = "unique_map")]
    prices: BTreeMap<String, PriceRecord>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PricingSettings {
    schema_version: u32,
    source: SourceMode,
}

#[derive(Clone, Debug, Serialize)]
pub struct PricingSnapshot {
    pub revision: u64,
    pub season_id: u32,
    pub source: SourceMode,
    pub connection: String,
    pub last_success_at: Option<String>,
    pub error: Option<String>,
    pub available_count: usize,
    pub refresh_seconds: u64,
    pub titrack_enabled: bool,
    pub prices: BTreeMap<String, PriceQuote>,
}

struct PricingState {
    revision: u64,
    season_id: u32,
    source: SourceMode,
    local: BTreeMap<String, PriceRecord>,
    automatic: BTreeMap<String, PriceRecord>,
    last_success_at: Option<String>,
    error: Option<String>,
}

/// Providers return validated records; consumers do not know any HTTP API.
/// A future approved network provider must apply request timeout and bounded
/// pagination/retries before returning a complete batch. No credentials ship here.
pub trait PriceProvider {
    fn load(&self, season_id: u32, now: i64) -> Result<BTreeMap<String, PriceRecord>, String>;
}
pub struct LocalJsonPriceProvider<'a> {
    pub json: &'a str,
}
impl PriceProvider for LocalJsonPriceProvider<'_> {
    fn load(&self, season_id: u32, now: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
        if self.json.len() > MAX_BYTES {
            return Err("Price JSON exceeds 4 MiB".into());
        }
        let doc: PriceDocument =
            serde_json::from_str(self.json).map_err(|e| format!("Invalid price JSON: {e}"))?;
        if doc.schema_version != 1 || doc.currency != "FE" {
            return Err("Expected schema_version 1 and currency FE".into());
        }
        if doc.season_id != season_id {
            return Err(format!(
                "Season mismatch: expected {season_id}, got {}",
                doc.season_id
            ));
        }
        validate_source(&doc.source)?;
        validate_time(&doc.updated_at, now)?;
        if doc.prices.len() > MAX_ITEMS {
            return Err("Too many prices".into());
        }
        let received_at = timestamp(now);
        let mut records = BTreeMap::new();
        for (id, value) in doc.prices {
            validate_value(&id, value)?;
            records.insert(
                id,
                PriceRecord {
                    value,
                    source: format!("local_json:{}", doc.source),
                    updated_at: doc.updated_at.clone(),
                    received_at: received_at.clone(),
                },
            );
        }
        Ok(records)
    }
}

pub struct TiTrackPriceProvider;
impl PriceProvider for TiTrackPriceProvider {
    fn load(&self, season_id: u32, _now: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
        Self::mapped_season(season_id)?;
        // Deliberate production gate. A published client key is NOT consent.
        Err(ACCESS_BLOCK.into())
    }
}
impl TiTrackPriceProvider {
    pub fn mapped_season(deanfinite_season: u32) -> Result<u32, String> {
        match deanfinite_season {
            1601 => Ok(1601),
            _ => Err(
                "No approved TITrack season mapping (only SS14 1601 -> 1601 is researched)".into(),
            ),
        }
    }
    /// Adapter for the researched PostgREST aggregated_prices response. Fixture
    /// tests exercise this method; production does not mistake fixtures for data.
    #[allow(dead_code)]
    pub fn decode_aggregates(
        json: &str,
        season_id: u32,
        now: i64,
    ) -> Result<BTreeMap<String, PriceRecord>, String> {
        #[derive(Deserialize)]
        struct Row {
            config_base_id: u64,
            season_id: u32,
            price_fe_median: f64,
            unique_devices: u32,
            submission_count: u32,
            updated_at: String,
        }
        let mapped = Self::mapped_season(season_id)?;
        if json.len() > MAX_BYTES {
            return Err("TITrack response exceeds 4 MiB".into());
        }
        let rows: Vec<Row> =
            serde_json::from_str(json).map_err(|e| format!("Invalid TITrack response: {e}"))?;
        if rows.len() > MAX_ITEMS {
            return Err("Too many TITrack rows".into());
        }
        let mut records = BTreeMap::new();
        for row in rows {
            if row.season_id != mapped {
                return Err("TITrack season mismatch".into());
            }
            if row.unique_devices < 3 || row.submission_count < row.unique_devices {
                return Err("TITrack aggregate has insufficient community observations".into());
            }
            let id = row.config_base_id.to_string();
            validate_value(&id, row.price_fe_median)?;
            validate_time(&row.updated_at, now)?;
            if records
                .insert(
                    id,
                    PriceRecord {
                        value: row.price_fe_median,
                        source: "titrack_aggregate".into(),
                        updated_at: row.updated_at,
                        received_at: timestamp(now),
                    },
                )
                .is_some()
            {
                return Err("Duplicate TITrack item".into());
            }
        }
        Ok(records)
    }
}

pub struct PricingManager {
    root: PathBuf,
    state: RwLock<PricingState>,
    // Serialize import/source/season/load so a late result cannot cross seasons.
    operation: Mutex<()>,
}
impl PricingManager {
    pub fn new(season_id: u32, root: PathBuf) -> Self {
        Self {
            root,
            state: RwLock::new(PricingState {
                revision: 0,
                season_id,
                source: SourceMode::default(),
                local: BTreeMap::new(),
                automatic: BTreeMap::new(),
                last_success_at: None,
                error: None,
            }),
            operation: Mutex::new(()),
        }
    }
    pub fn initialize(&self) {
        let _op = self.operation.lock().unwrap();
        let mut state = self.state.write().unwrap();
        match read_bounded(&self.root.join("settings.json")) {
            Ok(Some(data)) => match serde_json::from_str::<PricingSettings>(&data) {
                Ok(settings) if settings.schema_version == 1 => state.source = settings.source,
                _ => {
                    state.error = Some(
                        "Invalid pricing settings; using Local JSON without overwriting file"
                            .into(),
                    )
                }
            },
            Err(e) => state.error = Some(e),
            _ => {}
        }
        self.load_season(&mut state);
        state.revision += 1;
    }
    fn season_path(&self, season: u32, name: &str) -> PathBuf {
        self.root.join("v1").join(season.to_string()).join(name)
    }
    fn load_season(&self, state: &mut PricingState) {
        state.local.clear();
        state.automatic.clear();
        state.last_success_at = None;
        for (file, automatic) in [("local.json", false), ("automatic.json", true)] {
            match self.read_cache(state.season_id, file, automatic) {
                Ok(Some(prices)) => {
                    if automatic {
                        state.automatic = prices;
                    } else {
                        state.local = prices;
                    }
                }
                Err(e) => state.error = Some(e),
                _ => {}
            }
        }
        state.last_success_at = state
            .local
            .values()
            .chain(state.automatic.values())
            .map(|r| r.received_at.clone())
            .max();
    }
    fn read_cache(
        &self,
        season: u32,
        name: &str,
        automatic: bool,
    ) -> Result<Option<BTreeMap<String, PriceRecord>>, String> {
        let Some(data) = read_bounded(&self.season_path(season, name))? else {
            return Ok(None);
        };
        let doc: CacheDocument =
            serde_json::from_str(&data).map_err(|e| format!("Invalid {name} cache: {e}"))?;
        if doc.schema_version != 1
            || doc.season_id != season
            || doc.currency != "FE"
            || doc.prices.len() > MAX_ITEMS
        {
            return Err(format!("Invalid season/schema/currency in {name}"));
        }
        let now = now_secs();
        for (id, r) in &doc.prices {
            validate_value(id, r.value)?;
            validate_time(&r.updated_at, now)?;
            validate_time(&r.received_at, now)?;
            if automatic && r.source != "titrack_aggregate"
                || !automatic && !r.source.starts_with("local_json:")
            {
                return Err("Cache provider mismatch".into());
            }
            if r.source.len() > 100 {
                return Err("Invalid cached source".into());
            }
        }
        Ok(Some(doc.prices))
    }
    pub fn set_season(&self, season: u32) -> Result<(), String> {
        if season == 0 || season > 100_000 {
            return Err("Invalid season ID".into());
        }
        let _op = self.operation.lock().unwrap();
        let mut state = self.state.write().unwrap();
        if state.season_id != season {
            state.season_id = season;
            state.error = None;
            self.load_season(&mut state);
            state.revision += 1;
        }
        Ok(())
    }
    pub fn set_source(&self, source: SourceMode) -> Result<(), String> {
        let _op = self.operation.lock().unwrap();
        let settings = PricingSettings {
            schema_version: 1,
            source: source.clone(),
        };
        atomic_write(
            &self.root.join("settings.json"),
            &serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?,
        )?;
        let mut state = self.state.write().unwrap();
        state.source = source;
        state.error = None;
        state.revision += 1;
        Ok(())
    }
    pub fn import_json(&self, json: &str) -> Result<(), String> {
        let _op = self.operation.lock().unwrap();
        let season = self.state.read().unwrap().season_id;
        let records = LocalJsonPriceProvider { json }.load(season, now_secs())?;
        let doc = CacheDocument {
            schema_version: 1,
            season_id: season,
            currency: "FE".into(),
            prices: records.clone(),
        };
        atomic_write(
            &self.season_path(season, "local.json"),
            &serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?,
        )?;
        let mut state = self.state.write().unwrap();
        // An import is a complete (possibly partial-coverage) local price table.
        state.local = records;
        state.last_success_at = Some(timestamp(now_secs()));
        state.error = None;
        state.revision += 1;
        Ok(())
    }
    pub fn refresh(&self) {
        self.refresh_from(&TiTrackPriceProvider);
    }
    fn refresh_from(&self, provider: &dyn PriceProvider) {
        let _op = self.operation.lock().unwrap();
        let (season, source) = {
            let state = self.state.read().unwrap();
            (state.season_id, state.source.clone())
        };
        if source == SourceMode::Automatic {
            // Never hold the valuation lock while a provider loads data. The
            // production provider is gated; tests inject fixture/error providers.
            let now = now_secs();
            let result = provider.load(season, now).and_then(|incoming| {
                if incoming.len() > MAX_ITEMS {
                    return Err("Too many provider prices".into());
                }
                for (id, record) in &incoming {
                    validate_value(id, record.value)?;
                    validate_time(&record.updated_at, now)?;
                    validate_time(&record.received_at, now)?;
                    if record.source != "titrack_aggregate" {
                        return Err("Unexpected automatic price source".into());
                    }
                }
                let mut merged = self.state.read().unwrap().automatic.clone();
                for (id, record) in incoming {
                    // An incomplete batch or an older observation cannot destroy
                    // existing known estimates. Their original timestamps age.
                    let replace = merged
                        .get(&id)
                        .map(|previous| {
                            parse_time(&record.updated_at).unwrap()
                                >= parse_time(&previous.updated_at).unwrap()
                        })
                        .unwrap_or(true);
                    if replace {
                        merged.insert(id, record);
                    }
                }
                let doc = CacheDocument {
                    schema_version: 1,
                    season_id: season,
                    currency: "FE".into(),
                    prices: merged.clone(),
                };
                atomic_write(
                    &self.season_path(season, "automatic.json"),
                    &serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?,
                )?;
                Ok(merged)
            });
            let mut state = self.state.write().unwrap();
            match result {
                Ok(prices) => {
                    state.automatic = prices;
                    state.last_success_at = Some(timestamp(now));
                    state.error = None;
                }
                Err(error) => state.error = Some(error),
            }
        }
        // Also refresh stale statuses without treating them as drops.
        self.state.write().unwrap().revision += 1;
    }
    pub fn snapshot(&self) -> PricingSnapshot {
        let state = self.state.read().unwrap();
        let now = now_secs();
        let mut records = state.local.clone();
        if state.source == SourceMode::Automatic {
            for (id, record) in &state.automatic {
                // Prefer a fresh aggregate; keep a newer local fallback over stale data.
                if !is_stale(record, now) || !records.contains_key(id) {
                    records.insert(id.clone(), record.clone());
                }
            }
        }
        let mut prices: BTreeMap<_, _> = records
            .into_iter()
            .map(|(id, r)| (id, quote(&r, now)))
            .collect();
        prices.insert("100300".into(), base_currency());
        PricingSnapshot {
            revision: state.revision,
            season_id: state.season_id,
            source: state.source.clone(),
            connection: if state.source == SourceMode::Automatic {
                "blocked"
            } else {
                "offline_local"
            }
            .into(),
            last_success_at: state.last_success_at.clone(),
            error: state.error.clone().or_else(|| {
                if state.source == SourceMode::Automatic {
                    Some(ACCESS_BLOCK.into())
                } else {
                    None
                }
            }),
            available_count: prices.len(),
            refresh_seconds: REFRESH_SECONDS,
            titrack_enabled: false,
            prices,
        }
    }
    pub fn season_id(&self) -> u32 {
        self.state.read().unwrap().season_id
    }
    pub fn get_quote(&self, id: &str) -> PriceQuote {
        if id == "100300" {
            return base_currency();
        }
        let state = self.state.read().unwrap();
        let now = now_secs();
        let record = if state.source == SourceMode::Automatic {
            state
                .automatic
                .get(id)
                .filter(|r| !is_stale(r, now))
                .or_else(|| state.local.get(id))
                .or_else(|| state.automatic.get(id))
        } else {
            state.local.get(id)
        };
        record
            .map(|r| quote(r, now))
            .unwrap_or_else(PriceQuote::unknown)
    }
    // Compatibility only for existing expense arithmetic. Loot carries get_quote
    // as well: zero here is never advertised as a known market valuation.
    pub fn get_price_sync(&self, id: &str) -> f64 {
        self.get_quote(id).value.unwrap_or(0.0)
    }
}
// Reject duplicate IDs instead of silently accepting a later value that could
// conceal an invalid first entry. This also protects cache documents.
fn unique_map<'de, D, T>(deserializer: D) -> Result<BTreeMap<String, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Unique<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Unique<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a price object with unique item IDs")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut access: A,
        ) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some((id, value)) = access.next_entry::<String, T>()? {
                if result.insert(id.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!("Duplicate item ID: {id}")));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Unique(std::marker::PhantomData))
}

fn base_currency() -> PriceQuote {
    PriceQuote {
        value: Some(1.0),
        status: PriceStatus::Known,
        source: Some("base_currency".into()),
        updated_at: None,
        received_at: None,
    }
}
fn quote(r: &PriceRecord, now: i64) -> PriceQuote {
    PriceQuote {
        value: Some(r.value),
        status: if is_stale(r, now) {
            PriceStatus::Stale
        } else {
            PriceStatus::Known
        },
        source: Some(r.source.clone()),
        updated_at: Some(r.updated_at.clone()),
        received_at: Some(r.received_at.clone()),
    }
}
fn is_stale(r: &PriceRecord, now: i64) -> bool {
    parse_time(&r.updated_at)
        .map(|t| now.saturating_sub(t) > STALE_SECONDS)
        .unwrap_or(true)
}
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn timestamp(now: i64) -> String {
    chrono::DateTime::from_timestamp(now, 0)
        .unwrap()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
fn parse_time(value: &str) -> Result<i64, String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|d| d.timestamp())
        .map_err(|_| "Expected RFC3339 timestamp with timezone".into())
}
fn validate_time(value: &str, now: i64) -> Result<(), String> {
    let t = parse_time(value)?;
    if t < 0 || t > now + 300 {
        return Err("Timestamp is outside supported range or more than 5 minutes in future".into());
    }
    Ok(())
}
fn validate_source(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 80 || value.chars().any(char::is_control) {
        Err("Source must be 1..80 bytes without control characters".into())
    } else {
        Ok(())
    }
}
pub fn valid_item_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 12
        && id.as_bytes().iter().all(u8::is_ascii_digit)
        && !id.starts_with('0')
}
fn validate_value(id: &str, value: f64) -> Result<(), String> {
    if !valid_item_id(id)
        || !value.is_finite()
        || !(0.0..=MAX_PRICE).contains(&value)
        || id == "100300" && value != 1.0
    {
        Err(format!(
            "Invalid item/price for {id}; expected finite 0..{MAX_PRICE} FE (100300 must equal 1)"
        ))
    } else {
        Ok(())
    }
}
fn read_bounded(path: &Path) -> Result<Option<String>, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Cannot read price cache: {e}")),
    };
    let mut data = String::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_string(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > MAX_BYTES {
        return Err("Price cache exceeds 4 MiB".into());
    }
    Ok(Some(data))
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_BYTES {
        return Err("Price cache exceeds 4 MiB".into());
    }
    let parent = path.parent().ok_or("Missing price directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp = parent.join(format!(
        ".prices-{}-{}.tmp",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&tmp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result.map_err(|e| format!("Cannot save price cache; previous file preserved: {e}"))
}

#[cfg(test)]
#[path = "pricing/tests.rs"]
mod tests;

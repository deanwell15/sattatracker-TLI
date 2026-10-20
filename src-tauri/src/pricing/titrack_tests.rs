use super::*;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

fn row(id: u64, time: &str) -> serde_json::Value {
    serde_json::json!({"config_base_id":id,"season_id":1601,"price_fe_median":388.0,"price_fe_p10":300,"price_fe_p90":400,"submission_count":12,"unique_devices":4,"updated_at":time})
}
fn request() -> SyncRequest {
    SyncRequest {
        metadata: SyncMetadata::default(),
        cancellation: None,
    }
}
struct Mock {
    responses: Mutex<VecDeque<Result<HttpResponse, ProviderError>>>,
    queries: Mutex<Vec<Vec<(String, String)>>>,
}
impl Mock {
    fn rows(mut pages: Vec<Vec<serde_json::Value>>) -> Self {
        if pages.last().is_some_and(|page| !page.is_empty()) {
            pages.push(Vec::new());
        }
        Self {
            responses: Mutex::new(
                pages
                    .into_iter()
                    .map(|rows| {
                        Ok(HttpResponse {
                            bytes: serde_json::to_vec(&rows).unwrap(),
                            status: 200,
                            retry_after: None,
                        })
                    })
                    .collect(),
            ),
            queries: Mutex::new(Vec::new()),
        }
    }
}
impl HttpTransport for Mock {
    fn get(&self, query: &[(String, String)], _: Duration) -> Result<HttpResponse, ProviderError> {
        self.queries.lock().unwrap().push(query.to_vec());
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("Unexpected HTTP request")
    }
}
#[test]
fn full_pages_then_inclusive_delta_preserve_microsecond_boundary() {
    let now = now_secs();
    let time = format!("{}.442384+00:00", timestamp(now - 10).trim_end_matches('Z'));
    let rows = vec![row(350714, &time), row(350715, &time), row(350716, &time)];
    let mock = Mock::rows(vec![
        vec![rows[0].clone()],
        rows[..2].to_vec(),
        rows[2..].to_vec(),
    ]);
    let first = fetch_table(&mock, 1601, now, &request(), 2, 50).unwrap();
    assert_eq!(first.records.len(), 3);
    assert_eq!(first.records["350714"].value, 388.0);
    assert_eq!(
        first.sync.as_ref().unwrap().cursor.as_deref(),
        Some(time.as_str())
    );
    let queries = mock.queries.lock().unwrap();
    assert!(queries[2].contains(&("config_base_id".into(), "gt.350715".into())));
    assert!(queries[1].contains(&("updated_at".into(), format!("lte.{time}"))));
    drop(queries);
    let delta = Mock::rows(vec![vec![rows[2].clone()], vec![row(350717, &time)]]);
    let second = fetch_table(
        &delta,
        1601,
        now,
        &SyncRequest {
            metadata: first.sync.unwrap(),
            cancellation: None,
        },
        2,
        50,
    )
    .unwrap();
    assert_eq!(second.records.len(), 1);
    assert!(
        delta.queries.lock().unwrap()[1].contains(&("updated_at".into(), format!("gte.{time}")))
    );
}
#[test]
fn empty_delta_retains_cursor_and_periodic_full_has_no_lower_filter() {
    let now = now_secs();
    let time = timestamp(now - 60);
    let req = SyncRequest {
        metadata: SyncMetadata {
            cursor: Some(time.clone()),
            last_full_at: Some(timestamp(now)),
            last_success_at: None,
        },
        cancellation: None,
    };
    let empty = Mock::rows(vec![vec![row(350714, &time)], vec![]]);
    let result = fetch_table(&empty, 1601, now, &req, 1000, 50).unwrap();
    assert!(result.records.is_empty());
    assert_eq!(result.sync.unwrap().cursor, Some(time.clone()));
    let mut req = req;
    req.metadata.last_full_at = Some(timestamp(now - FULL_SYNC_SECONDS));
    let full = Mock::rows(vec![vec![row(350714, &time)], vec![row(350714, &time)]]);
    fetch_table(&full, 1601, now, &req, 1000, 50).unwrap();
    assert!(!full.queries.lock().unwrap()[1]
        .iter()
        .any(|(k, v)| k == "updated_at" && v.starts_with("gte.")));
}
#[test]
fn empty_table_is_success_not_network_error() {
    let mock = Mock::rows(vec![vec![]]);
    let result = fetch_table(&mock, 1601, now_secs(), &request(), 1000, 50).unwrap();
    assert!(result.records.is_empty());
    assert!(result.sync.unwrap().cursor.is_none());
    assert_eq!(mock.queries.lock().unwrap().len(), 1);
}
#[test]
fn failed_middle_page_never_returns_partial_success() {
    let now = now_secs();
    let t = timestamp(now - 1);
    let mock = Mock::rows(vec![
        vec![row(350714, &t)],
        vec![row(350714, &t), row(350715, &t)],
    ]);
    mock.responses.lock().unwrap().insert(
        2,
        Ok(HttpResponse {
            bytes: vec![],
            status: 503,
            retry_after: None,
        }),
    );
    let error = fetch_table(&mock, 1601, now, &request(), 2, 50)
        .err()
        .unwrap();
    assert!(error.message.contains("503"));
    assert_eq!(mock.queries.lock().unwrap().len(), 3);
}
#[test]
fn invalid_rows_and_order_are_rejected() {
    let now = now_secs();
    let t = timestamp(now - 1);
    let valid = row(350714, &t);
    for (field, value) in [
        ("config_base_id", serde_json::json!(0)),
        ("config_base_id", serde_json::json!("../x")),
        ("price_fe_median", serde_json::json!(-1)),
        ("price_fe_median", serde_json::json!(1e20)),
        ("price_fe_median", serde_json::json!("NaN")),
        ("price_fe_p10", serde_json::json!(-1)),
        ("season_id", serde_json::json!(1501)),
        ("updated_at", serde_json::json!("bad")),
    ] {
        let mut invalid = valid.clone();
        invalid[field] = value;
        let mock = Mock::rows(vec![vec![invalid]]);
        assert!(
            fetch_table(&mock, 1601, now, &request(), 1000, 50).is_err(),
            "{field}"
        );
    }
    let mock = Mock::rows(vec![vec![valid.clone()], vec![row(350715, &t), valid]]);
    assert!(fetch_table(&mock, 1601, now, &request(), 2, 50).is_err());
}
#[test]
fn malformed_json_body_size_and_page_cap_are_errors() {
    let now = now_secs();
    let t = timestamp(now - 1);
    for bytes in [b"broken".to_vec(), vec![b'x'; MAX_PAGE_BYTES + 1]] {
        let mock = Mock::rows(vec![]);
        mock.responses.lock().unwrap().push_back(Ok(HttpResponse {
            bytes,
            status: 200,
            retry_after: None,
        }));
        assert!(fetch_table(&mock, 1601, now, &request(), 2, 50).is_err());
    }
    let mock = Mock::rows(vec![
        vec![row(350714, &t)],
        vec![row(350714, &t), row(350715, &t)],
    ]);
    assert!(fetch_table(&mock, 1601, now, &request(), 2, 1)
        .err()
        .unwrap()
        .message
        .contains("page limit"));
}
#[test]
fn cancelled_generation_makes_zero_requests() {
    let mock = Mock::rows(vec![]);
    let generation = Arc::new(AtomicU64::new(2));
    let req = SyncRequest {
        metadata: SyncMetadata::default(),
        cancellation: Some((generation, 1)),
    };
    assert!(
        fetch_table(&mock, 1601, now_secs(), &req, 1000, 50)
            .err()
            .unwrap()
            .cancelled
    );
    assert!(mock.queries.lock().unwrap().is_empty());
}
#[test]
fn status_and_retry_after_are_handled_without_immediate_retries() {
    for code in [401, 403, 429, 500, 503, 302] {
        let mock = Mock::rows(vec![]);
        mock.responses.lock().unwrap().push_back(Ok(HttpResponse {
            bytes: vec![],
            status: code,
            retry_after: Some("3600".into()),
        }));
        let error = fetch_table(&mock, 1601, now_secs(), &request(), 1000, 50)
            .err()
            .unwrap();
        assert_eq!(error.access_denied, matches!(code, 401 | 403));
        assert_eq!(error.retry_after_seconds, Some(3600));
        assert_eq!(mock.queries.lock().unwrap().len(), 1);
    }
    let now = now_secs();
    assert_eq!(
        retry_after(
            Some(
                &(chrono::DateTime::from_timestamp(now + 600, 0)
                    .unwrap()
                    .to_rfc2822())
            ),
            now
        ),
        Some(600)
    );
}
// Actual reqwest wire requests hit a loopback-only mock server, never Supabase.
fn server(
    replies: Vec<(u16, String, String, u64)>,
) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for (status, headers, body, delay) in replies {
            let accept_deadline = Instant::now() + Duration::from_secs(3);
            let (mut stream, _) = loop {
                match listener.accept() {
                    Ok(connection) => break connection,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            && Instant::now() < accept_deadline =>
                    {
                        thread::sleep(Duration::from_millis(2))
                    }
                    Err(_) => {
                        panic!("Mock server did not receive expected request within deadline")
                    }
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut input = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
                input.push_str(&line);
            }
            if tx.send(input).is_err() {
                return;
            }
            thread::sleep(Duration::from_millis(delay));
            let _=write!(stream,"HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",body.len());
        }
    });
    (
        format!("http://{address}/rest/v1/aggregated_prices"),
        rx,
        handle,
    )
}
#[test]
fn actual_http_client_gets_pages_with_apikey_and_never_posts() {
    let now = now_secs();
    let t = timestamp(now - 1);
    let body = serde_json::to_string(&vec![row(350714, &t)]).unwrap();
    let (endpoint, requests, handle) = server(vec![
        (200, String::new(), body.clone(), 0),
        (200, String::new(), body, 0),
        (200, String::new(), "[]".into(), 0),
    ]);
    let result = fetch_table(
        &HttpsTransport::loopback(endpoint, Duration::from_secs(1)),
        1601,
        now,
        &request(),
        1000,
        50,
    )
    .unwrap();
    assert_eq!(result.records["350714"].value, 388.0);
    handle.join().unwrap();
    for wire in requests.try_iter() {
        // Do not print `wire` on assertion failure: it contains the public header.
        assert!(wire.starts_with("GET /rest/v1/aggregated_prices?"));
        assert!(wire
            .to_ascii_lowercase()
            .contains("apikey: sb_publishable_"));
        assert!(!wire.to_ascii_lowercase().contains("authorization:"));
        assert!(wire.contains("season_id=eq.1601"));
    }
}
#[test]
fn actual_client_surfaces_http_failures_without_following_redirects() {
    for code in [401, 403, 429, 500, 503, 302] {
        let (endpoint, requests, handle) = server(vec![(
            code,
            "Retry-After: 600\r\nLocation: https://elsewhere.invalid/\r\n".into(),
            String::new(),
            0,
        )]);
        let response = HttpsTransport::loopback(endpoint, Duration::from_secs(1))
            .get(&[], Duration::from_secs(1))
            .unwrap();
        assert_eq!(response.status, code);
        assert_eq!(response.retry_after.as_deref(), Some("600"));
        handle.join().unwrap();
        assert_eq!(requests.try_iter().count(), 1);
    }
}
#[test]
fn actual_client_timeout_is_bounded_and_key_free() {
    let (endpoint, requests, handle) = server(vec![(200, String::new(), "[]".into(), 200)]);
    let error = HttpsTransport::loopback(endpoint, Duration::from_millis(30))
        .get(&[], Duration::from_millis(30))
        .err()
        .unwrap();
    assert!(error.message.contains("timed out"));
    assert!(!error.message.contains("sb_publishable"));
    handle.join().unwrap();
    assert_eq!(requests.try_iter().count(), 1);
}

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "deanfinite-http-{}-{}-{}",
            std::process::id(),
            now_secs(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct TableProvider(Mock);
impl PriceProvider for TableProvider {
    fn load(&self, _: u32, _: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
        Err("Use fetch".into())
    }
    fn fetch(
        &self,
        season: u32,
        now: i64,
        request: &SyncRequest,
    ) -> Result<ProviderBatch, ProviderError> {
        fetch_table(&self.0, season, now, request, 2, 50)
    }
}
#[test]
fn manager_keeps_prices_cursor_and_disk_on_failed_page_then_restarts_offline() {
    let now = now_secs();
    let t = timestamp(now - 10);
    let dir = Temp::new();
    let manager = PricingManager::new(1601, dir.0.clone());
    manager.set_source(SourceMode::Automatic).unwrap();
    let initial = TableProvider(Mock::rows(vec![
        vec![row(350714, &t)],
        vec![row(350714, &t)],
    ]));
    manager.refresh_at(&initial, now, false);
    assert_eq!(manager.get_quote("350714").value, Some(388.0));
    let path = manager.season_path(1601, "automatic.json");
    let before = fs::read(&path).unwrap();
    let failed = TableProvider(Mock::rows(vec![
        vec![row(350716, &t)],
        vec![row(350715, &t), row(350716, &t)],
    ]));
    failed.0.responses.lock().unwrap().insert(
        2,
        Ok(HttpResponse {
            bytes: vec![],
            status: 503,
            retry_after: None,
        }),
    );
    manager.refresh_at(&failed, now, false);
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(manager.get_quote("350715").value.is_none());
    let restart = PricingManager::new(1601, dir.0.clone());
    restart.initialize();
    assert_eq!(restart.get_quote("350714").value, Some(388.0));
    assert_eq!(
        restart.state.read().unwrap().sync.cursor.as_deref(),
        Some(t.as_str())
    );
    let empty = TableProvider(Mock::rows(vec![vec![row(350714, &t)], vec![]]));
    restart.refresh_at(&empty, now, false);
    assert_eq!(restart.get_quote("350714").value, Some(388.0));
    assert_eq!(
        restart.state.read().unwrap().sync.cursor.as_deref(),
        Some(t.as_str())
    );
}
struct ErrorProvider {
    calls: AtomicU64,
    error: ProviderError,
}
impl PriceProvider for ErrorProvider {
    fn load(&self, _: u32, _: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
        unreachable!()
    }
    fn fetch(&self, _: u32, _: i64, _: &SyncRequest) -> Result<ProviderBatch, ProviderError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Err(self.error.clone())
    }
}
#[test]
fn local_mode_never_calls_provider_and_retry_after_blocks_early_requests() {
    let dir = Temp::new();
    let manager = PricingManager::new(1601, dir.0.clone());
    let now = now_secs() + 10_000;
    let mut error = ProviderError::new("TITrack HTTP 429");
    error.retry_after_seconds = Some(3600);
    let provider = ErrorProvider {
        calls: AtomicU64::new(0),
        error,
    };
    manager.refresh_at(&provider, now, true);
    assert_eq!(provider.calls.load(Ordering::Relaxed), 0);
    manager.set_source(SourceMode::Automatic).unwrap();
    manager.refresh_at(&provider, now, true);
    manager.refresh_at(&provider, now + 3599, true);
    assert_eq!(provider.calls.load(Ordering::Relaxed), 1);
    assert_eq!(manager.state.read().unwrap().next_request_at, now + 3600);
    manager.refresh_at(&provider, now + 3600, true);
    assert_eq!(provider.calls.load(Ordering::Relaxed), 2);
}
#[test]
fn backoff_and_access_denial_do_not_spin_or_bypass() {
    let dir = Temp::new();
    let manager = PricingManager::new(1601, dir.0.clone());
    manager.set_source(SourceMode::Automatic).unwrap();
    let mut now = now_secs() + 10_000;
    let provider = ErrorProvider {
        calls: AtomicU64::new(0),
        error: ProviderError::new("TITrack HTTP 503"),
    };
    for delay in [300, 600, 1200, 2400, 3600, 3600] {
        manager.refresh_at(&provider, now, true);
        assert_eq!(manager.state.read().unwrap().next_request_at, now + delay);
        now += delay;
    }
    let mut error = ProviderError::new("TITrack HTTP 403");
    error.access_denied = true;
    let forbidden = ErrorProvider {
        calls: AtomicU64::new(0),
        error,
    };
    manager.refresh_at(&forbidden, now, true);
    manager.set_source(SourceMode::LocalJson).unwrap();
    manager.set_source(SourceMode::Automatic).unwrap();
    manager.refresh_at(&forbidden, now + 100_000, true);
    assert_eq!(forbidden.calls.load(Ordering::Relaxed), 1);
    assert_eq!(manager.snapshot().connection, "access_denied");
    assert!(manager.snapshot().error.unwrap().contains("403"));
}
#[test]
fn slow_provider_holds_no_price_mutex_and_late_source_result_is_discarded() {
    struct Slow {
        started: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl PriceProvider for Slow {
        fn load(&self, _: u32, _: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
            unreachable!()
        }
        fn fetch(&self, _: u32, _: i64, _: &SyncRequest) -> Result<ProviderBatch, ProviderError> {
            self.started.send(()).unwrap();
            self.release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            Ok(ProviderBatch {
                records: BTreeMap::new(),
                sync: None,
            })
        }
    }
    let dir = Temp::new();
    let manager = Arc::new(PricingManager::new(1601, dir.0.clone()));
    manager.set_source(SourceMode::Automatic).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let copy = manager.clone();
    let worker = thread::spawn(move || {
        copy.refresh_at(
            &Slow {
                started: started_tx,
                release: Mutex::new(release_rx),
            },
            now_secs(),
            true,
        )
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let (done_tx, done_rx) = mpsc::channel();
    let copy = manager.clone();
    let writer = thread::spawn(move || {
        assert_eq!(copy.get_quote("100300").value, Some(1.0));
        copy.set_source(SourceMode::LocalJson).unwrap();
        done_tx.send(()).unwrap();
    });
    let promptly = done_rx.recv_timeout(Duration::from_secs(1)).is_ok();
    release_tx.send(()).unwrap();
    worker.join().unwrap();
    writer.join().unwrap();
    assert!(promptly, "Price locks were held during provider request");
    assert!(!manager.season_path(1601, "automatic.json").exists());
}
#[test]
fn validation_errors_redact_reflected_public_key() {
    let error: ProviderError = format!("Invalid response {}", PUBLISHABLE_KEY.trim()).into();
    assert!(!error.message.contains(PUBLISHABLE_KEY.trim()));
}

#[test]
fn server_page_cap_below_requested_limit_does_not_truncate_sync() {
    let now = now_secs();
    let time = timestamp(now - 1);
    let mock = Mock::rows(vec![
        vec![row(350714, &time)],
        vec![row(350714, &time)],
        vec![row(350715, &time)],
    ]);
    let result = fetch_table(&mock, 1601, now, &request(), 1000, 50).unwrap();
    assert_eq!(result.records.len(), 2);
    assert_eq!(mock.queries.lock().unwrap().len(), 4);
}

#[test]
fn retry_after_starts_at_error_receipt_not_beginning_of_slow_sync() {
    let dir = Temp::new();
    let manager = PricingManager::new(1601, dir.0.clone());
    manager.set_source(SourceMode::Automatic).unwrap();
    let now = now_secs();
    let mut error = ProviderError::new("TITrack HTTP 429");
    error.retry_after_seconds = Some(600);
    let provider = ErrorProvider {
        calls: AtomicU64::new(0),
        error,
    };
    manager.refresh_at(&provider, now - 120, true);
    assert!(manager.state.read().unwrap().next_request_at >= now + 600);
}

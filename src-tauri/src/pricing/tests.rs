use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "deanfinite-pricing-{}-{}-{}",
            std::process::id(),
            now_secs(),
            NEXT.fetch_add(1, Ordering::SeqCst)
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
fn json(season: u32, prices: serde_json::Value) -> String {
    serde_json::json!({"schema_version":1,"season_id":season,"currency":"FE","source":"test_fixture","updated_at":timestamp(now_secs()),"prices":prices}).to_string()
}

#[test]
fn unknown_is_not_zero_and_base_currency_is_fixed() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    assert_eq!(p.get_quote("123456").value, None);
    assert_eq!(p.get_quote("123456").status, PriceStatus::Unknown);
    p.import_json(&json(1601, serde_json::json!({"123456":0,"113300":7})))
        .unwrap();
    assert_eq!(p.get_quote("123456").value, Some(0.0));
    assert_eq!(p.get_quote("123456").status, PriceStatus::Known);
    assert_eq!(p.get_quote("113300").value, Some(7.0)); // No ends-with-300 heuristic.
    assert_eq!(p.get_quote("100300").value, Some(1.0));
    assert!(p
        .import_json(&json(1601, serde_json::json!({"100300":2})))
        .is_err());
}
#[test]
fn offline_import_and_reload_and_season_isolation() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    p.import_json(&json(1601, serde_json::json!({"123456":12.5})))
        .unwrap();
    let second = PricingManager::new(1601, dir.0.clone());
    second.initialize();
    assert_eq!(second.get_quote("123456").value, Some(12.5));
    assert!(second
        .import_json(&json(1501, serde_json::json!({"123456":99})))
        .is_err());
    second.set_season(1501).unwrap();
    assert_eq!(second.get_quote("123456").value, None);
    second
        .import_json(&json(1501, serde_json::json!({"123456":3})))
        .unwrap();
    second.set_season(1601).unwrap();
    assert_eq!(second.get_quote("123456").value, Some(12.5));
    assert_eq!(TiTrackPriceProvider::mapped_season(1601).unwrap(), 1601);
    assert!(TiTrackPriceProvider::mapped_season(1701).is_err());
}
#[test]
fn malformed_invalid_and_wrong_season_do_not_change_cache_or_memory() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    p.import_json(&json(1601, serde_json::json!({"123456":2})))
        .unwrap();
    let path = p.season_path(1601, "local.json");
    let before = fs::read(&path).unwrap();
    for invalid in [
        "{broken".to_string(),
        json(1601, serde_json::json!({"123456":-1})),
        json(1601, serde_json::json!({"../x":2})),
        json(1601, serde_json::json!({"0123":2})),
        json(1501, serde_json::json!({"123456":3})),
        json(1601, serde_json::json!({"123456":1e12})),
        json(1601, serde_json::json!({"123456":"NaN"})),
        json(1601, serde_json::json!({"123456":2})).replace(":2}", ":1e400}"),
        json(1601, serde_json::json!({"123456":2}))
            .replace("\"schema_version\":1", "\"schema_version\":2"),
    ] {
        assert!(p.import_json(&invalid).is_err(), "{invalid}");
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(p.get_quote("123456").value, Some(2.0));
    }
}
#[test]
fn old_unscoped_cache_is_ignored_and_corrupt_scoped_cache_is_preserved() {
    let dir = Temp::new();
    fs::write(dir.0.join("prices_cache.json"), "{\"123456\":99}").unwrap();
    let p = PricingManager::new(1601, dir.0.clone());
    p.initialize();
    assert_eq!(p.get_quote("123456").value, None);
    let path = p.season_path(1601, "local.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "broken").unwrap();
    p.initialize();
    assert!(p.snapshot().error.is_some());
    assert_eq!(fs::read_to_string(path).unwrap(), "broken");
}
#[test]
fn staleness_uses_source_time_never_import_time() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    let mut doc: serde_json::Value =
        serde_json::from_str(&json(1601, serde_json::json!({"123456":2}))).unwrap();
    doc["updated_at"] = timestamp(now_secs() - STALE_SECONDS - 1).into();
    p.import_json(&doc.to_string()).unwrap();
    let q = p.get_quote("123456");
    assert_eq!(q.status, PriceStatus::Stale);
    assert_eq!(q.value, Some(2.0));
    doc["updated_at"] = timestamp(now_secs() + 3600).into();
    assert!(p.import_json(&doc.to_string()).is_err());
}
#[test]
fn titrack_fixture_is_aggregate_only_and_validated() {
    let fixture = include_str!("../../../tests/fixtures/titrack-aggregated-ss14.json");
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-20T09:00:00Z")
        .unwrap()
        .timestamp();
    let rows = TiTrackPriceProvider::decode_aggregates(fixture, 1601, now).unwrap();
    assert_eq!(rows["123456"].value, 12.5);
    assert_eq!(rows["654321"].value, 0.0);
    for bad in [
        fixture.replace("1601", "1501"),
        fixture.replace("12.5", "-1"),
        fixture.replace("\"unique_devices\":4", "\"unique_devices\":1"),
        fixture.replace("2026-10-20T08:00:00Z", "wrong"),
    ] {
        assert!(TiTrackPriceProvider::decode_aggregates(&bad, 1601, now).is_err());
    }
    assert!(TiTrackPriceProvider
        .load(1601, now)
        .unwrap_err()
        .contains("not enabled"));
}
#[test]
fn unavailable_automatic_keeps_local_and_persists_mode_without_network() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    p.import_json(&json(1601, serde_json::json!({"123456":2})))
        .unwrap();
    p.set_source(SourceMode::Automatic).unwrap();
    p.refresh();
    assert_eq!(p.get_quote("123456").value, Some(2.0));
    assert_eq!(p.snapshot().connection, "blocked");
    assert!(!p.snapshot().titrack_enabled);
    assert_eq!(p.snapshot().refresh_seconds, 300);
    let second = PricingManager::new(1601, dir.0.clone());
    second.initialize();
    assert_eq!(second.snapshot().source, SourceMode::Automatic);
    second.set_source(SourceMode::LocalJson).unwrap();
    assert_eq!(second.snapshot().connection, "offline_local");
}
#[test]
fn failed_atomic_save_keeps_previous_valuation() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    p.import_json(&json(1601, serde_json::json!({"123456":2})))
        .unwrap();
    let path = p.season_path(1601, "local.json");
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(p
        .import_json(&json(1601, serde_json::json!({"123456":3})))
        .is_err());
    assert_eq!(p.get_quote("123456").value, Some(2.0));
    assert!(path.is_dir());
}

#[test]
fn duplicate_ids_are_rejected_without_hiding_invalid_values() {
    let data = json(1601, serde_json::json!({"123456": 2}));
    let duplicate = data.replace("\"123456\":2", "\"123456\":-1,\"123456\":2");
    assert!(LocalJsonPriceProvider { json: &duplicate }
        .load(1601, now_secs())
        .is_err());
}

#[test]
fn local_mode_does_not_read_automatic_prices_and_stale_aggregate_can_fall_back() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    let r = PriceRecord {
        value: 8.0,
        source: "titrack_aggregate".into(),
        updated_at: timestamp(now_secs()),
        received_at: timestamp(now_secs()),
    };
    let doc = CacheDocument {
        schema_version: 1,
        season_id: 1601,
        currency: "FE".into(),
        prices: BTreeMap::from([("123456".into(), r)]),
    };
    atomic_write(
        &p.season_path(1601, "automatic.json"),
        &serde_json::to_vec(&doc).unwrap(),
    )
    .unwrap();
    p.initialize();
    assert_eq!(p.get_quote("123456").value, None);
    p.set_source(SourceMode::Automatic).unwrap();
    assert_eq!(p.get_quote("123456").value, Some(8.0));
    p.import_json(&json(1601, serde_json::json!({"123456":2})))
        .unwrap();
    assert_eq!(p.get_quote("123456").value, Some(8.0));
    p.state
        .write()
        .unwrap()
        .automatic
        .get_mut("123456")
        .unwrap()
        .updated_at = timestamp(now_secs() - STALE_SECONDS - 1);
    assert_eq!(p.get_quote("123456").value, Some(2.0));
    assert_eq!(p.snapshot().prices["123456"].value, Some(2.0));
}

struct FixtureProvider;
impl PriceProvider for FixtureProvider {
    fn load(&self, season: u32, now: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
        let fixture = include_str!("../../../tests/fixtures/titrack-aggregated-ss14.json")
            .replace("2026-10-20T08:00:00Z", &timestamp(now));
        TiTrackPriceProvider::decode_aggregates(&fixture, season, now)
    }
}
#[test]
fn aggregate_fixture_flows_through_manager_cache_and_offline_restart() {
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    p.set_source(SourceMode::Automatic).unwrap();
    p.refresh_from(&FixtureProvider);
    assert_eq!(p.get_quote("123456").value, Some(12.5));
    assert_eq!(p.get_quote("654321").status, PriceStatus::Known);
    let bytes = fs::read(p.season_path(1601, "automatic.json")).unwrap();
    p.refresh(); // Production is unavailable: known fixture cache survives.
    assert_eq!(p.get_quote("123456").value, Some(12.5));
    assert_eq!(
        fs::read(p.season_path(1601, "automatic.json")).unwrap(),
        bytes
    );
    let restart = PricingManager::new(1601, dir.0.clone());
    restart.initialize();
    assert_eq!(restart.get_quote("123456").value, Some(12.5));
    assert_eq!(
        restart.get_quote("123456").source.as_deref(),
        Some("titrack_aggregate")
    );
    restart.set_source(SourceMode::LocalJson).unwrap();
    assert!(restart.get_quote("123456").value.is_none());
}
#[test]
fn provider_invalid_batch_does_not_overwrite_valid_cache() {
    struct InvalidProvider;
    impl PriceProvider for InvalidProvider {
        fn load(&self, season: u32, now: i64) -> Result<BTreeMap<String, PriceRecord>, String> {
            let mut rows = FixtureProvider.load(season, now)?;
            rows.get_mut("123456").unwrap().value = f64::NAN;
            Ok(rows)
        }
    }
    let dir = Temp::new();
    let p = PricingManager::new(1601, dir.0.clone());
    p.set_source(SourceMode::Automatic).unwrap();
    p.refresh_from(&FixtureProvider);
    let before = fs::read(p.season_path(1601, "automatic.json")).unwrap();
    p.refresh_from(&InvalidProvider);
    assert_eq!(p.get_quote("123456").value, Some(12.5));
    assert_eq!(
        fs::read(p.season_path(1601, "automatic.json")).unwrap(),
        before
    );
    assert!(p.snapshot().error.is_some());
}

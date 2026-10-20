use linemux::MuxedLines;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::watch;
use tokio::time::{self, Instant as TokioInstant};

use crate::game_state_machine::GameStateMachine;
use crate::models::{AppSettings, GameEvent};
use crate::parser::LogParser;
use crate::pricing::PricingManager;

const SORT_DEBOUNCE_MS: u64 = 1500;

async fn find_season_id(log_path: &PathBuf) -> Option<u32> {
    let file = tokio::fs::File::open(log_path).await.ok()?;
    let mut lines = BufReader::new(file).lines();
    let mut latest = None;
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(season) = crate::season::from_log_line(&line) {
            latest = Some(season);
        }
    }
    latest
}

async fn count_lines(log_path: &PathBuf) -> u64 {
    let file = match tokio::fs::File::open(log_path).await {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let reader = BufReader::new(file);
    let mut lines_iter = reader.lines();
    let mut count = 0u64;
    while let Ok(Some(_)) = lines_iter.next_line().await {
        count += 1;
    }
    count
}

async fn replay_existing_log(
    log_path: &PathBuf,
    parser: &mut LogParser,
    fsm: &mut GameStateMachine,
    pricing: &PricingManager,
) {
    let file = match tokio::fs::File::open(log_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[tailer] Cannot open log for replay: {}", e);
            return;
        }
    };
    let reader = BufReader::new(file);
    let mut lines_iter = reader.lines();
    let mut count = 0usize;

    while let Ok(Some(line)) = lines_iter.next_line().await {
        if let Some(parsed_event) = parser.parse_line(&line) {
            let _ = fsm.process(parsed_event, pricing);
        }
        count += 1;
    }
    println!(
        "[tailer] Replayed {} existing log lines to initialise state",
        count
    );
}

pub async fn start_tailing(
    app: AppHandle,
    log_path: PathBuf,
    pricing: std::sync::Arc<PricingManager>,
    mut settings: watch::Receiver<AppSettings>,
) {
    let mut parser = LogParser::new();
    if let Some(season) = find_season_id(&log_path).await {
        let _ = pricing.set_season(season);
        let _ = app.emit("price_snapshot", pricing.snapshot());
    }
    let mut fsm = GameStateMachine::new(settings.borrow().clone());

    let total_lines = count_lines(&log_path).await;
    let _ = app.emit("game_event", GameEvent::LoadingStart { total_lines });

    replay_existing_log(&log_path, &mut parser, &mut fsm, &pricing).await;

    let _ = app.emit("game_event", GameEvent::LoadingDone);

    let mut lines = MuxedLines::new().expect("Failed to initialise linemux");
    match lines.add_file(&log_path).await {
        Ok(_) => println!("[tailer] Monitoring log: {:?}", log_path),
        Err(e) => {
            eprintln!("[tailer] Cannot monitor {:?}: {}", log_path, e);
            return;
        }
    }

    let far_future = TokioInstant::now() + Duration::from_secs(u32::MAX as u64);
    let sort_timer = time::sleep_until(far_future);
    tokio::pin!(sort_timer);
    let mut sort_pending = false;

    loop {
        tokio::select! {
            () = &mut sort_timer, if sort_pending => {
                sort_pending = false;
                sort_timer.as_mut().reset(TokioInstant::now() + Duration::from_secs(u32::MAX as u64));
                let raw_items = fsm.bag.get_all_main_inventory_items();
                println!("[tailer] Sort debounce done, {} items in bag", raw_items.len());
                let mut total_value = 0.0f64;
                let mut snapshot: Vec<crate::models::InventoryItem> = Vec::new();
                for (item_id, qty) in &raw_items {
                    let price     = pricing.get_price_sync(item_id);
                    let item_name = crate::db::DbManager::get_item_name(item_id);
                    total_value  += price * (*qty as f64);
                    snapshot.push(crate::models::InventoryItem {
                        item_id:   item_id.clone(),
                        item_name,
                        qty:       *qty,
                        price,
                        valuation: pricing.get_quote(item_id),
                        season_id: pricing.season_id(),
                    });
                }
                println!("[tailer] Snapshot: total_value={:.2}", total_value);
                let _ = app.emit("game_event", GameEvent::InventoryRefreshed { value: total_value, items: snapshot });
            }

            line_result = lines.next_line() => {
                let Ok(Some(line)) = line_result else { break; };

                if settings.has_changed().unwrap_or(false) {
                    fsm.settings = settings.borrow_and_update().clone();
                }

                let raw_line = line.line();
                if let Some(season) = crate::season::from_log_line(raw_line) {
                    let previous = pricing.snapshot().season_id;
                    if previous != season {
                        let _ = pricing.set_season(season);
                        let _ = app.emit("price_snapshot", pricing.snapshot());
                    }
                }

                if raw_line.contains("ProtoName=ResetItemsLayout") && raw_line.contains(" start") {
                    let raw_items = fsm.bag.get_all_main_inventory_items();
                    if !raw_items.is_empty() {
                        let mut total_value = 0.0f64;
                        let mut snapshot: Vec<crate::models::InventoryItem> = Vec::new();
                        for (item_id, qty) in &raw_items {
                            let price     = pricing.get_price_sync(item_id);
                            let item_name = crate::db::DbManager::get_item_name(item_id);
                            total_value  += price * (*qty as f64);
                            snapshot.push(crate::models::InventoryItem {
                                item_id:   item_id.clone(),
                                item_name,
                                qty:       *qty,
                                price,
                        valuation: pricing.get_quote(item_id),
                        season_id: pricing.season_id(),
                            });
                        }
                        println!("[tailer] Pre-sort snapshot: {} items, {:.2} FE", snapshot.len(), total_value);
                        let _ = app.emit("game_event", GameEvent::InventoryRefreshed { value: total_value, items: snapshot });
                    }
                }

                if sort_pending && raw_line.contains("BagMgr@:InitBagData") {
                    sort_timer.as_mut().reset(
                        TokioInstant::now() + Duration::from_millis(SORT_DEBOUNCE_MS)
                    );
                }

                if let Some(parsed_event) = parser.parse_line(raw_line) {
                    let mut game_events = fsm.process(parsed_event, &pricing);

                    for ev in &mut game_events {
                        if let GameEvent::Expense { item_id, total, qty, .. } = ev {
                            let unit_price = pricing.get_price_sync(item_id);
                            if unit_price > 0.0 {
                                *total = unit_price * (*qty as f64);
                            }
                        }
                    }

                    let sort_done = game_events.iter().any(|e| matches!(e, GameEvent::SortComplete));
                    if sort_done {
                        sort_pending = true;
                        sort_timer.as_mut().reset(
                            TokioInstant::now() + Duration::from_millis(SORT_DEBOUNCE_MS)
                        );
                        println!("[tailer] SortComplete, debounce armed");
                    }

                    for ev in game_events {
                        let should_emit = match &ev {
                            GameEvent::LootDrop { qty, .. } => *qty > 0,
                            GameEvent::SortComplete => false,
                            _ => true,
                        };
                        if should_emit {
                            let _ = app.emit("game_event", ev);
                        }
                    }
                }
            }
        }
    }
}

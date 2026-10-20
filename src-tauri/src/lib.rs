mod bag_manager;
mod db;
mod game_state_machine;
mod icon_resource;
mod models;
mod parser;
mod pricing;
#[cfg(test)]
mod pricing_integration_tests;
mod season;
mod settings;
mod tailer;
mod utils;

use models::AppSettings;
use pricing::{PricingManager, PricingSnapshot, SourceMode};
use std::sync::Arc;
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};
use tokio::sync::watch;
use tokio::task;

struct SettingsState {
    sender: watch::Sender<AppSettings>,
    path: std::path::PathBuf,
}

#[tauri::command]
fn set_click_through(window: tauri::Window, ignore: bool) {
    if window.label() == "main" {
        let _ = window.set_ignore_cursor_events(ignore);
    }
}

#[tauri::command]
fn save_settings(
    state: State<'_, SettingsState>,
    ignore_auction: bool,
    apply_tax: bool,
) -> Result<(), String> {
    let settings = AppSettings {
        ignore_auction,
        apply_tax,
    };

    settings::save(&state.path, &settings)?;
    // Saving also works when the game log is absent and no receiver is running.
    state.sender.send_replace(settings);

    Ok(())
}

#[tauri::command]
fn load_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    Ok(state.sender.borrow().clone())
}

#[tauri::command]
fn get_cursor_position(window: tauri::Window) -> (f64, f64) {
    match window.cursor_position() {
        Ok(pos) => (pos.x, pos.y),
        Err(_) => (0.0, 0.0),
    }
}

#[derive(serde::Serialize)]
struct PriceEntry {
    item_id: String,
    item_name: String,
    price: f64,
    valuation: pricing::PriceQuote,
}

#[tauri::command]
fn get_all_prices(state: State<'_, Arc<PricingManager>>) -> Vec<PriceEntry> {
    let mut entries: Vec<PriceEntry> = state
        .snapshot()
        .prices
        .into_iter()
        .map(|(id, valuation)| {
            let price = valuation.value.unwrap_or(0.0);
            let item_name = crate::db::DbManager::get_item_name(&id);
            PriceEntry {
                item_id: id,
                item_name,
                price,
                valuation,
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        b.price
            .partial_cmp(&a.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries
}

#[tauri::command]
fn get_pricing_snapshot(state: State<'_, Arc<PricingManager>>) -> PricingSnapshot {
    state.snapshot()
}

#[tauri::command]
async fn import_price_json(
    app: tauri::AppHandle,
    state: State<'_, Arc<PricingManager>>,
    json: String,
) -> Result<PricingSnapshot, String> {
    if json.len() > 4 * 1024 * 1024 {
        return Err("Price JSON exceeds 4 MiB".into());
    }
    let manager = state.inner().clone();
    let snapshot = tauri::async_runtime::spawn_blocking(move || {
        manager.import_json(&json)?;
        Ok::<_, String>(manager.snapshot())
    })
    .await
    .map_err(|e| e.to_string())??;
    let _ = app.emit("price_snapshot", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
async fn set_price_source(
    app: tauri::AppHandle,
    state: State<'_, Arc<PricingManager>>,
    source: SourceMode,
) -> Result<PricingSnapshot, String> {
    let manager = state.inner().clone();
    let snapshot = tauri::async_runtime::spawn_blocking(move || {
        manager.set_source(source)?;
        Ok::<_, String>(manager.snapshot())
    })
    .await
    .map_err(|e| e.to_string())??;
    let _ = app.emit("price_snapshot", &snapshot);
    app.state::<Arc<tokio::sync::Notify>>().notify_one();
    Ok(snapshot)
}

#[tauri::command]
fn get_log_path() -> String {
    match utils::get_game_log_path() {
        Some(p) => p.to_string_lossy().to_string(),
        None => "Log file not found".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("deanfinite-icon", |context, request| {
            let result = if request.method() == tauri::http::Method::GET {
                context.app_handle().path().resource_dir()
                    .map_err(|_| icon_resource::IconError::NotFound)
                    .and_then(|path| icon_resource::read_icon(&path.join("icons"), request.uri().path()))
            } else {
                Err(icon_resource::IconError::BadPath)
            };
            let (status, bytes, mime) = match result {
                Ok((bytes, mime)) => (200, bytes, mime),
                Err(icon_resource::IconError::BadPath) => (400, Vec::new(), "text/plain"),
                Err(_) => (404, Vec::new(), "text/plain"),
            };
            tauri::http::Response::builder()
                .status(status)
                .header("Content-Type", mime)
                .header("X-Content-Type-Options", "nosniff")
                .header("Cache-Control", "public, max-age=86400")
                .body(bytes)
                .expect("static icon response headers")
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut.matches(Modifiers::empty(), Code::F8) {
                        let _ = app.emit("toggle_edit_mode", ());
                    }
                    if shortcut.matches(Modifiers::empty(), Code::F7) {
                        if let Some(window) = app.get_webview_window("main") {
                            let visible = window.is_visible().unwrap_or(true);
                            if visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let config_dir = app.path().app_config_dir()?;
            let local_data_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&config_dir)?;
            std::fs::create_dir_all(&local_data_dir)?;
            let config_path = config_dir.join("config.json");
            let initial_settings = settings::load(&config_path).unwrap_or_else(|error| {
                eprintln!("[Deanfinite] {error}; using defaults without modifying the file");
                AppSettings::default()
            });
            let (settings_tx, settings_rx) = watch::channel(initial_settings);
            app.manage(SettingsState {
                sender: settings_tx,
                path: config_path,
            });
            // Target league is explicit. A detected log season replaces it; old
            // unscoped prices_cache.json is deliberately never imported.
            let pricing = Arc::new(PricingManager::new(1601, local_data_dir.join("prices")));
            app.manage(pricing.clone());
            let pricing_wake = Arc::new(tokio::sync::Notify::new());
            app.manage(pricing_wake.clone());
            let price_app = app.handle().clone();
            let background_pricing = pricing.clone();
            task::spawn(async move {
                let initial = background_pricing.clone();
                let _ = tokio::task::spawn_blocking(move || initial.initialize()).await;
                let _ = price_app.emit("price_snapshot", background_pricing.snapshot());
                loop {
                    let manager = background_pricing.clone();
                    let _ = tokio::task::spawn_blocking(move || manager.refresh()).await;
                    let _ = price_app.emit("price_snapshot", background_pricing.snapshot());
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(background_pricing.next_refresh_delay())) => {},
                        _ = pricing_wake.notified() => {},
                    }
                }
            });

            use std::str::FromStr;
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let quit_i = MenuItem::with_id(app, "quit", "Exit Deanfinite", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::new()
                .tooltip("Deanfinite")
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            if let Ok(shortcut) = Shortcut::from_str("F8") {
                let _ = app.global_shortcut().register(shortcut);
            }
            if let Ok(shortcut) = Shortcut::from_str("F7") {
                let _ = app.global_shortcut().register(shortcut);
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_decorations(false);
                let _ = window.set_skip_taskbar(true);
                let _ = window.set_ignore_cursor_events(true);
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let size = monitor.size();
                    let pos = monitor.position();
                    let _ =
                        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                            x: pos.x,
                            y: pos.y,
                        }));
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: size.width,
                        height: size.height,
                    }));
                }
                let _ = window.show();
            }

            let app_handle = app.handle().clone();

            task::spawn(async move {
                match utils::get_game_log_path() {
                    Some(log_path) => {
                        let _ =
                            app_handle.emit("log_status", format!("Watching log: {:?}", log_path));
                        tailer::start_tailing(app_handle, log_path, pricing, settings_rx).await;
                    }
                    None => {
                        let _ =
                            app_handle.emit("log_status", "Error: Game log not found!".to_string());
                        let _ =
                            app_handle.emit("game_event", crate::models::GameEvent::LoadingDone);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_click_through,
            save_settings,
            load_settings,
            get_log_path,
            get_cursor_position,
            get_all_prices,
            get_pricing_snapshot,
            import_price_json,
            set_price_source,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

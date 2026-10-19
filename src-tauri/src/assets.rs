use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AssetUpdateInfo {
    pub has_update: bool,
    pub remote_version: String,
    pub local_version: String,
}

#[tauri::command]
pub async fn check_assets_update(app: AppHandle) -> Result<AssetUpdateInfo, String> {
    let local_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let version_file = local_dir.join("assets_version.json");

    let local_version = if version_file.exists() {
        tokio::fs::read_to_string(&version_file)
            .await
            .unwrap_or_else(|_| "0.0.0".to_string())
    } else {
        "0.0.0".to_string()
    };

    let client = Client::builder()
        .user_agent("Deanfinite")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://api.github.com/repos/sattatrapy/sattatrackerUpdate/releases/tags/Assets";
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Ok(AssetUpdateInfo {
            has_update: local_version == "0.0.0",
            remote_version: local_version.clone(),
            local_version,
        });
    }

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let remote_version = json["published_at"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    Ok(AssetUpdateInfo {
        has_update: local_version != remote_version || local_version == "0.0.0",
        remote_version,
        local_version,
    })
}

#[tauri::command]
pub async fn download_assets(
    app: AppHandle,
    remote_version: String,
    window: tauri::Window,
) -> Result<(), String> {
    let local_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;

    if !local_dir.exists() {
        tokio::fs::create_dir_all(&local_dir)
            .await
            .map_err(|e| e.to_string())?;
    }

    let zip_path = local_dir.join("assets.zip");
    let client = Client::builder()
        .user_agent("Deanfinite")
        .build()
        .map_err(|e| e.to_string())?;

    let download_url =
        "https://github.com/sattatrapy/sattatrackerUpdate/releases/download/Assets/assets.zip";
    let mut res = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Download failed with status: {}", res.status()));
    }

    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::File::create(&zip_path)
        .await
        .map_err(|e| e.to_string())?;

    let total_size = res.content_length().unwrap_or(150_000_000);
    let mut downloaded: f64 = 0.0;

    let mut last_emit = std::time::Instant::now();
    while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as f64;

        if last_emit.elapsed().as_millis() > 500 {
            let progress = (downloaded / total_size as f64) * 100.0;
            let _ = window.emit("asset-download-progress", progress);
            last_emit = std::time::Instant::now();
        }
    }

    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    let _ = window.emit("asset-download-progress", 100.0);
    let _ = window.emit("asset-extract-start", ());

    let zip_bytes = std::fs::read(&zip_path).map_err(|e| format!("Failed to read zip: {}", e))?;

    let target_dir = local_dir.join("public_assets");
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir).ok();
    }
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create extract dir: {}", e))?;

    let cursor = std::io::Cursor::new(zip_bytes);
    zip_extract::extract(cursor, &target_dir, false)
        .map_err(|e| format!("Extraction failed: {:?}", e))?;

    std::fs::remove_file(&zip_path).ok();
    let version_file = local_dir.join("assets_version.json");
    std::fs::write(&version_file, &remote_version)
        .map_err(|e| format!("Failed to save version: {}", e))?;

    let _ = window.emit("asset-extract-done", ());
    Ok(())
}

#[tauri::command]
pub fn get_assets_path(app: AppHandle) -> Result<String, String> {
    let local_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let target_dir = local_dir.join("public_assets");
    Ok(target_dir.to_string_lossy().to_string())
}

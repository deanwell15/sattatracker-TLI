use crate::models::AppSettings;
use std::path::Path;

// The caller supplies Deanfinite's resolved app_config_dir; never search beside
// the executable or in the working directory, and never import upstream data.
pub fn load(path: &Path) -> Result<AppSettings, String> {
    match std::fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json)
            .map_err(|error| format!("Cannot parse settings {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(error) => Err(format!("Cannot read settings {}: {error}", path.display())),
    }
}

pub fn save(path: &Path, settings: &AppSettings) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "Settings require an application configuration directory".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("Cannot create settings directory {}: {error}", parent.display()))?;
    let json = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    std::fs::write(path, json)
        .map_err(|error| format!("Cannot write settings {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "deanfinite-settings-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn settings_survive_reload_and_create_their_directory() {
        let dir = TempDir::new();
        let path = dir.0.join("Deanfinite/config.json");
        save(&path, &AppSettings { ignore_auction: true, apply_tax: true }).unwrap();
        let settings = load(&path).unwrap();
        assert!(settings.ignore_auction && settings.apply_tax);
        save(&path, &AppSettings::default()).unwrap();
        let settings = load(&path).unwrap();
        assert!(!settings.ignore_auction && !settings.apply_tax);
        let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(json, serde_json::json!({"ignore_auction": false, "apply_tax": false}));
    }

    #[test]
    fn clean_start_does_not_import_or_modify_other_settings() {
        let dir = TempDir::new();
        let other = dir.0.join("existing-app/config.json");
        save(&other, &AppSettings { ignore_auction: true, apply_tax: true }).unwrap();
        let original = std::fs::read(&other).unwrap();
        let path = dir.0.join("Deanfinite/config.json");
        let settings = load(&path).unwrap();
        assert!(!settings.ignore_auction && !settings.apply_tax);
        save(&path, &settings).unwrap();
        assert_eq!(std::fs::read(other).unwrap(), original);
    }

    #[test]
    fn malformed_settings_are_reported_without_overwriting() {
        let dir = TempDir::new();
        let path = dir.0.join("config.json");
        std::fs::write(&path, "invalid json").unwrap();
        assert!(load(&path).unwrap_err().contains("Cannot parse settings"));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "invalid json");
    }

    #[test]
    fn write_errors_are_reported_without_removing_the_blocking_file() {
        let dir = TempDir::new();
        let blocker = dir.0.join("not-a-directory");
        std::fs::write(&blocker, "keep").unwrap();
        assert!(save(&blocker.join("config.json"), &AppSettings::default()).is_err());
        assert_eq!(std::fs::read_to_string(blocker).unwrap(), "keep");
    }
}

use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use crate::model::AppConfig;

use super::config_migrations;

fn config_path_from_current_dir() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    let direct = current.join("software-items.json");
    if direct.exists() {
        return Some(direct);
    }

    let nested = current.join("config").join("software-items.json");
    if nested.exists() {
        return Some(nested);
    }

    let parent_nested = current.parent()?.join("config").join("software-items.json");
    if parent_nested.exists() {
        return Some(parent_nested);
    }

    None
}

pub fn resolve_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(path) = config_path_from_current_dir() {
        return Ok(path);
    }

    let app_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("failed to get app config directory: {error}"))?;

    fs::create_dir_all(&app_dir)
        .map_err(|error| format!("failed to create app config directory: {error}"))?;

    Ok(app_dir.join("software-items.json"))
}

pub fn load_or_init_config(app: &AppHandle) -> Result<AppConfig, String> {
    let path = resolve_config_path(app)?;
    if !path.exists() {
        let config = AppConfig::default();
        save_config(app, &config)?;
        return Ok(config);
    }

    let data = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read config from {}: {error}", path.display()))?;

    let mut config = serde_json::from_str::<AppConfig>(&data)
        .map_err(|error| format!("failed to parse config from {}: {error}", path.display()))?;

    if config_migrations::patch_legacy_config(&mut config) {
        save_config(app, &config)?;
    }

    Ok(config)
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = resolve_config_path(app)?;
    let payload = serde_json::to_string_pretty(config)
        .map_err(|error| format!("failed to serialize config: {error}"))?;

    fs::write(&path, payload)
        .map_err(|error| format!("failed to write config to {}: {error}", path.display()))
}

use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use crate::model::{AppConfig, SoftwareItem};

const OLD_BREW_CHECK_CMD: &str = "brew outdated --quiet brew";
const OLD_BREW_UPDATE_CMD: &str = "brew update && brew upgrade brew";
const OLD_BUN_CHECK_CMD: &str = "brew outdated --quiet bun";
const OLD_BUN_UPDATE_CMD: &str = "brew upgrade bun";
const NEW_BREW_CHECK_CMD: &str = "brew outdated --quiet";
const NEW_BREW_UPDATE_CMD: &str = "brew update && brew upgrade";
const NEW_BUN_CHECK_CMD: &str =
    "if brew list bun >/dev/null 2>&1; then brew outdated --quiet bun; else echo ''; fi";
const NEW_BUN_UPDATE_CMD: &str =
    "if brew list bun >/dev/null 2>&1; then brew upgrade bun; else echo 'bun is not managed by brew'; fi";
const NEW_BREW_DESC: &str = "Check and update Homebrew packages";

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

fn resolve_config_path(app: &AppHandle) -> Result<PathBuf, String> {
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

fn patch_legacy_item_commands(item: &mut SoftwareItem) -> bool {
    let mut changed = false;

    if item.id == "brew" {
        if item.update_check_command.as_deref() == Some(OLD_BREW_CHECK_CMD) {
            item.update_check_command = Some(NEW_BREW_CHECK_CMD.to_string());
            changed = true;
        }
        if item.update_command == OLD_BREW_UPDATE_CMD {
            item.update_command = NEW_BREW_UPDATE_CMD.to_string();
            changed = true;
        }
        if item.description == "Check and update Homebrew" {
            item.description = NEW_BREW_DESC.to_string();
            changed = true;
        }
    }

    if item.id == "bun" {
        if item.update_check_command.as_deref() == Some(OLD_BUN_CHECK_CMD) {
            item.update_check_command = Some(NEW_BUN_CHECK_CMD.to_string());
            changed = true;
        }
        if item.update_command == OLD_BUN_UPDATE_CMD {
            item.update_command = NEW_BUN_UPDATE_CMD.to_string();
            changed = true;
        }
    }

    changed
}

fn patch_legacy_config(config: &mut AppConfig) -> bool {
    let mut changed = false;
    for item in &mut config.items {
        if patch_legacy_item_commands(item) {
            changed = true;
        }
    }
    changed
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

    if patch_legacy_config(&mut config) {
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

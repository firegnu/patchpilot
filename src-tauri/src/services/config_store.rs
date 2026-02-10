use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use crate::model::{AppConfig, SoftwareItem};

const OLD_BREW_CHECK_CMD: &str = "brew outdated --quiet brew";
const OLD_BREW_UPDATE_CMD: &str = "brew update && brew upgrade brew";
const OLD_BUN_CHECK_CMD: &str = "brew outdated --quiet bun";
const OLD_BUN_UPDATE_CMD: &str = "brew upgrade bun";
const OLD_CLAUDE_CHECK_CMD: &str = "npm outdated -g @anthropic-ai/claude-code --parseable";
const OLD_CLAUDE_UPDATE_CMD: &str = "npm install -g @anthropic-ai/claude-code@latest";
const OLD_CLAUDE_DESC: &str = "Example: update npm-managed claude-code";
const NEW_BREW_CHECK_CMD: &str = "brew outdated --quiet";
const NEW_BREW_UPDATE_CMD: &str = "brew update && brew upgrade";
const NEW_BUN_CHECK_CMD: &str =
    "if brew list bun >/dev/null 2>&1; then brew outdated --quiet bun; else echo ''; fi";
const NEW_BUN_UPDATE_CMD: &str =
    "if brew list bun >/dev/null 2>&1; then brew upgrade bun; else echo 'bun is not managed by brew'; fi";
const NEW_CLAUDE_CURRENT_CMD: &str =
    "claude --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'";
const NEW_CLAUDE_LATEST_CMD: &str =
    "curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/anthropics/claude-code/releases/latest | sed -E 's#.*/tag/v?##'";
const NEW_CLAUDE_UPDATE_CMD: &str = "claude update";
const NEW_CLAUDE_DESC: &str = "Auto-check via GitHub releases; update manually with claude update";
const NEW_GEMINI_CURRENT_CMD: &str =
    "if command -v gemini >/dev/null 2>&1; then gemini --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'; else echo ''; fi";
const NEW_GEMINI_LATEST_CMD: &str = "npm view @google/gemini-cli version";
const NEW_GEMINI_UPDATE_CMD: &str = "npm upgrade -g @google/gemini-cli";
const NEW_GEMINI_DESC: &str = "Auto-check via npm registry; update manually with npm upgrade";
const NEW_CODEX_CURRENT_CMD: &str =
    "codex --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'";
const OLD_CODEX_LATEST_CMD: &str =
    "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask codex --json=v2 | sed -E 's/.*\"version\":\"([^\"]+)\".*/\\1/'";
const NEW_CODEX_LATEST_CMD: &str =
    "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask codex --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1";
const NEW_CODEX_UPDATE_CMD: &str = "brew upgrade --cask codex";
const NEW_CODEX_DESC: &str =
    "Auto-check via Homebrew cask metadata; update manually with brew upgrade --cask";
const NEW_OMZ_CURRENT_CMD: &str =
    "if [ -d \"${ZSH:-$HOME/.oh-my-zsh}/.git\" ]; then git -C \"${ZSH:-$HOME/.oh-my-zsh}\" rev-parse --short=12 HEAD; else echo ''; fi";
const NEW_OMZ_LATEST_CMD: &str =
    "if [ -d \"${ZSH:-$HOME/.oh-my-zsh}/.git\" ]; then REMOTE=\"$(git -C \"${ZSH:-$HOME/.oh-my-zsh}\" config --get remote.origin.url 2>/dev/null || echo https://github.com/ohmyzsh/ohmyzsh.git)\"; git ls-remote \"$REMOTE\" HEAD 2>/dev/null | awk '{print substr($1,1,12)}' | head -n 1; else echo ''; fi";
const NEW_OMZ_UPDATE_CMD: &str =
    "if [ -x \"${ZSH:-$HOME/.oh-my-zsh}/tools/upgrade.sh\" ]; then \"${ZSH:-$HOME/.oh-my-zsh}/tools/upgrade.sh\" -v minimal; else echo 'oh-my-zsh not found'; exit 1; fi";
const NEW_OMZ_DESC: &str = "Auto-check git HEAD; update manually via OMZ upgrade script";
const NEW_BREW_DESC: &str = "Check and update Homebrew packages";
const OLD_DEFAULT_CHECK_INTERVAL_MINUTES: u64 = 360;
const NEW_DEFAULT_CHECK_INTERVAL_MINUTES: u64 = 480;

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

    if item.id == "claude-code" {
        let looks_legacy = item.description == OLD_CLAUDE_DESC
            || item.update_check_command.as_deref() == Some(OLD_CLAUDE_CHECK_CMD)
            || item.update_command == OLD_CLAUDE_UPDATE_CMD;
        if looks_legacy {
            item.enabled = true;
            item.description = NEW_CLAUDE_DESC.to_string();
            item.current_version_command = Some(NEW_CLAUDE_CURRENT_CMD.to_string());
            item.latest_version_command = Some(NEW_CLAUDE_LATEST_CMD.to_string());
            item.update_check_command = None;
            item.update_check_regex = None;
            item.update_command = NEW_CLAUDE_UPDATE_CMD.to_string();
            changed = true;
        }
    }

    if item.id == "codex-cli"
        && item.latest_version_command.as_deref() == Some(OLD_CODEX_LATEST_CMD)
    {
        item.latest_version_command = Some(NEW_CODEX_LATEST_CMD.to_string());
        changed = true;
    }

    changed
}

fn append_default_items_if_missing(config: &mut AppConfig) -> bool {
    let mut changed = false;
    if !config.items.iter().any(|item| item.id == "claude-code") {
        config.items.push(SoftwareItem {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: NEW_CLAUDE_DESC.to_string(),
            current_version_command: Some(NEW_CLAUDE_CURRENT_CMD.to_string()),
            latest_version_command: Some(NEW_CLAUDE_LATEST_CMD.to_string()),
            update_check_command: None,
            update_check_regex: None,
            update_command: NEW_CLAUDE_UPDATE_CMD.to_string(),
        });
        changed = true;
    }
    if !config.items.iter().any(|item| item.id == "gemini-cli") {
        config.items.push(SoftwareItem {
            id: "gemini-cli".to_string(),
            name: "Google Gemini CLI".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: NEW_GEMINI_DESC.to_string(),
            current_version_command: Some(NEW_GEMINI_CURRENT_CMD.to_string()),
            latest_version_command: Some(NEW_GEMINI_LATEST_CMD.to_string()),
            update_check_command: None,
            update_check_regex: None,
            update_command: NEW_GEMINI_UPDATE_CMD.to_string(),
        });
        changed = true;
    }
    if !config.items.iter().any(|item| item.id == "codex-cli") {
        config.items.push(SoftwareItem {
            id: "codex-cli".to_string(),
            name: "Codex CLI".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: NEW_CODEX_DESC.to_string(),
            current_version_command: Some(NEW_CODEX_CURRENT_CMD.to_string()),
            latest_version_command: Some(NEW_CODEX_LATEST_CMD.to_string()),
            update_check_command: None,
            update_check_regex: None,
            update_command: NEW_CODEX_UPDATE_CMD.to_string(),
        });
        changed = true;
    }
    if !config.items.iter().any(|item| item.id == "oh-my-zsh") {
        config.items.push(SoftwareItem {
            id: "oh-my-zsh".to_string(),
            name: "Oh My Zsh".to_string(),
            kind: "cli".to_string(),
            enabled: true,
            description: NEW_OMZ_DESC.to_string(),
            current_version_command: Some(NEW_OMZ_CURRENT_CMD.to_string()),
            latest_version_command: Some(NEW_OMZ_LATEST_CMD.to_string()),
            update_check_command: None,
            update_check_regex: None,
            update_command: NEW_OMZ_UPDATE_CMD.to_string(),
        });
        changed = true;
    }
    changed
}

fn patch_legacy_config(config: &mut AppConfig) -> bool {
    let mut changed = false;
    if config.check_interval_minutes == OLD_DEFAULT_CHECK_INTERVAL_MINUTES {
        config.check_interval_minutes = NEW_DEFAULT_CHECK_INTERVAL_MINUTES;
        changed = true;
    }
    for item in &mut config.items {
        if patch_legacy_item_commands(item) {
            changed = true;
        }
    }
    if append_default_items_if_missing(config) {
        changed = true;
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

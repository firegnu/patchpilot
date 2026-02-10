use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn default_command_timeout_seconds() -> u64 {
    120
}

fn default_theme_mode() -> String {
    "system".to_string()
}

fn default_auto_check_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub enabled: bool,
    pub description: String,
    pub current_version_command: Option<String>,
    pub latest_version_command: Option<String>,
    pub update_check_command: Option<String>,
    pub update_check_regex: Option<String>,
    pub update_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub check_interval_minutes: u64,
    #[serde(default = "default_command_timeout_seconds")]
    pub command_timeout_seconds: u64,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default = "default_auto_check_enabled")]
    pub auto_check_enabled: bool,
    pub shared_update_commands: Vec<String>,
    pub items: Vec<SoftwareItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub item_id: String,
    pub checked_at: String,
    pub has_update: bool,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub details: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestResultSnapshot {
    pub item_id: String,
    pub checked_at: String,
    pub has_update: bool,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestResultState {
    pub updated_at: String,
    #[serde(default)]
    pub items: HashMap<String, LatestResultSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub item_id: String,
    pub updated_at: String,
    pub output: CommandOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistoryEntry {
    pub id: String,
    pub action: String,
    pub target: String,
    pub command: Option<String>,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stderr: Option<String>,
    pub recorded_at: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: Option<u128>,
    pub summary: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            check_interval_minutes: 480,
            command_timeout_seconds: default_command_timeout_seconds(),
            theme_mode: default_theme_mode(),
            auto_check_enabled: default_auto_check_enabled(),
            shared_update_commands: vec!["brew update".to_string(), "brew upgrade".to_string()],
            items: vec![
                SoftwareItem {
                    id: "brew".to_string(),
                    name: "Homebrew".to_string(),
                    kind: "cli".to_string(),
                    enabled: true,
                    description: "Check and update Homebrew packages".to_string(),
                    current_version_command: Some(
                        "brew --version | head -n 1 | awk '{print $2}'".to_string(),
                    ),
                    latest_version_command: None,
                    update_check_command: Some("brew outdated --quiet".to_string()),
                    update_check_regex: Some(".+".to_string()),
                    update_command: "brew update && brew upgrade".to_string(),
                },
                SoftwareItem {
                    id: "bun".to_string(),
                    name: "Bun".to_string(),
                    kind: "cli".to_string(),
                    enabled: true,
                    description: "Check and update Bun (if managed via brew)".to_string(),
                    current_version_command: Some("bun --version".to_string()),
                    latest_version_command: None,
                    update_check_command: Some(
                        "if brew list bun >/dev/null 2>&1; then brew outdated --quiet bun; else echo ''; fi"
                            .to_string(),
                    ),
                    update_check_regex: Some(".+".to_string()),
                    update_command: "if brew list bun >/dev/null 2>&1; then brew upgrade bun; else echo 'bun is not managed by brew'; fi".to_string(),
                },
                SoftwareItem {
                    id: "claude-code".to_string(),
                    name: "Claude Code".to_string(),
                    kind: "cli".to_string(),
                    enabled: true,
                    description: "Auto-check via GitHub releases; update manually with claude update"
                        .to_string(),
                    current_version_command: Some(
                        "claude --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'"
                            .to_string(),
                    ),
                    latest_version_command: Some(
                        "curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/anthropics/claude-code/releases/latest | sed -E 's#.*/tag/v?##'"
                            .to_string(),
                    ),
                    update_check_command: None,
                    update_check_regex: None,
                    update_command: "claude update".to_string(),
                },
                SoftwareItem {
                    id: "gemini-cli".to_string(),
                    name: "Google Gemini CLI".to_string(),
                    kind: "cli".to_string(),
                    enabled: true,
                    description:
                        "Auto-check via npm registry; update manually with npm upgrade".to_string(),
                    current_version_command: Some(
                        "if command -v gemini >/dev/null 2>&1; then gemini --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'; else echo ''; fi"
                            .to_string(),
                    ),
                    latest_version_command: Some("npm view @google/gemini-cli version".to_string()),
                    update_check_command: None,
                    update_check_regex: None,
                    update_command: "npm upgrade -g @google/gemini-cli".to_string(),
                },
                SoftwareItem {
                    id: "codex-cli".to_string(),
                    name: "Codex CLI".to_string(),
                    kind: "cli".to_string(),
                    enabled: true,
                    description: "Auto-check via Homebrew cask metadata; update manually with brew upgrade --cask"
                        .to_string(),
                    current_version_command: Some(
                        "codex --version | sed -E 's/[^0-9]*([0-9]+\\.[0-9]+\\.[0-9]+).*/\\1/'"
                            .to_string(),
                    ),
                    latest_version_command: Some(
                        "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask codex --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1"
                            .to_string(),
                    ),
                    update_check_command: None,
                    update_check_regex: None,
                    update_command: "brew upgrade --cask codex".to_string(),
                },
                SoftwareItem {
                    id: "oh-my-zsh".to_string(),
                    name: "Oh My Zsh".to_string(),
                    kind: "cli".to_string(),
                    enabled: true,
                    description:
                        "Auto-check git HEAD; update manually via OMZ upgrade script".to_string(),
                    current_version_command: Some(
                        "if [ -d \"${ZSH:-$HOME/.oh-my-zsh}/.git\" ]; then git -C \"${ZSH:-$HOME/.oh-my-zsh}\" rev-parse --short=12 HEAD; else echo ''; fi"
                            .to_string(),
                    ),
                    latest_version_command: Some(
                        "if [ -d \"${ZSH:-$HOME/.oh-my-zsh}/.git\" ]; then REMOTE=\"$(git -C \"${ZSH:-$HOME/.oh-my-zsh}\" config --get remote.origin.url 2>/dev/null || echo https://github.com/ohmyzsh/ohmyzsh.git)\"; git ls-remote \"$REMOTE\" HEAD 2>/dev/null | awk '{print substr($1,1,12)}' | head -n 1; else echo ''; fi"
                            .to_string(),
                    ),
                    update_check_command: None,
                    update_check_regex: None,
                    update_command: "if [ -x \"${ZSH:-$HOME/.oh-my-zsh}/tools/upgrade.sh\" ]; then \"${ZSH:-$HOME/.oh-my-zsh}/tools/upgrade.sh\" -v minimal; else echo 'oh-my-zsh not found'; exit 1; fi".to_string(),
                },
                SoftwareItem {
                    id: "visual-studio-code".to_string(),
                    name: "Visual Studio Code".to_string(),
                    kind: "gui".to_string(),
                    enabled: true,
                    description:
                        "Auto-check app version via local Info.plist and Homebrew cask metadata"
                            .to_string(),
                    current_version_command: Some(
                        "if [ -d \"/Applications/Visual Studio Code.app\" ]; then defaults read \"/Applications/Visual Studio Code.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
                    ),
                    latest_version_command: Some(
                        "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask visual-studio-code --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1".to_string(),
                    ),
                    update_check_command: None,
                    update_check_regex: None,
                    update_command:
                        "echo 'Visual Studio Code update is managed manually outside PatchPilot'"
                            .to_string(),
                },
                SoftwareItem {
                    id: "antigravity".to_string(),
                    name: "Antigravity".to_string(),
                    kind: "gui".to_string(),
                    enabled: true,
                    description:
                        "Auto-check app version via local Info.plist and Homebrew cask metadata"
                            .to_string(),
                    current_version_command: Some(
                        "if [ -d \"/Applications/Antigravity.app\" ]; then defaults read \"/Applications/Antigravity.app/Contents/Info.plist\" CFBundleShortVersionString 2>/dev/null || echo ''; else echo ''; fi".to_string(),
                    ),
                    latest_version_command: Some(
                        "HOMEBREW_NO_AUTO_UPDATE=1 brew info --cask antigravity --json=v2 | sed -nE 's/.*\"version\":[[:space:]]*\"([^\"]+)\".*/\\1/p' | head -n 1 | cut -d, -f1".to_string(),
                    ),
                    update_check_command: None,
                    update_check_regex: None,
                    update_command:
                        "echo 'Antigravity update is managed manually outside PatchPilot'"
                            .to_string(),
                },
            ],
        }
    }
}

impl Default for LatestResultState {
    fn default() -> Self {
        Self {
            updated_at: chrono::Utc::now().to_rfc3339(),
            items: HashMap::new(),
        }
    }
}

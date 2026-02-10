use serde::{Deserialize, Serialize};

fn default_command_timeout_seconds() -> u64 {
    120
}

fn default_theme_mode() -> String {
    "system".to_string()
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
            check_interval_minutes: 360,
            command_timeout_seconds: default_command_timeout_seconds(),
            theme_mode: default_theme_mode(),
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
                    enabled: false,
                    description: "Example: update npm-managed claude-code".to_string(),
                    current_version_command: Some("claude --version".to_string()),
                    latest_version_command: None,
                    update_check_command: Some(
                        "npm outdated -g @anthropic-ai/claude-code --parseable".to_string(),
                    ),
                    update_check_regex: Some(".+".to_string()),
                    update_command: "npm install -g @anthropic-ai/claude-code@latest".to_string(),
                },
            ],
        }
    }
}

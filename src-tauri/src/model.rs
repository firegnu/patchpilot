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

fn default_auto_check_manual_enabled() -> bool {
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
    #[serde(default = "default_auto_check_manual_enabled")]
    pub auto_check_manual_enabled: bool,
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
            auto_check_manual_enabled: default_auto_check_manual_enabled(),
            shared_update_commands: crate::software_catalog::default_shared_update_commands(),
            items: crate::software_catalog::default_software_items(),
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

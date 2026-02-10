use chrono::Utc;
use tauri::AppHandle;

use crate::model::{CheckResult, CommandOutput, ExecutionHistoryEntry};
use crate::services::history_store;

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn next_history_id(action: &str, target: &str) -> String {
    format!("{}-{action}-{target}", Utc::now().timestamp_micros())
}

pub fn append_entry_safe(app: &AppHandle, entry: ExecutionHistoryEntry) {
    if let Err(error) = history_store::append_entry(app, entry) {
        eprintln!("failed to append history: {error}");
    }
}

pub fn check_item_entry(result: &CheckResult) -> ExecutionHistoryEntry {
    let success = result.error.is_none();
    let summary = match (&result.error, result.has_update) {
        (Some(error), _) => error.clone(),
        (None, true) => "发现可用更新".to_string(),
        (None, false) => "已是最新".to_string(),
    };

    ExecutionHistoryEntry {
        id: next_history_id("check-item", &result.item_id),
        action: "check-item".to_string(),
        target: result.item_id.clone(),
        command: None,
        stdout: None,
        stderr: None,
        recorded_at: now_rfc3339(),
        success,
        exit_code: None,
        timed_out: false,
        duration_ms: None,
        summary,
    }
}

pub fn check_all_entry(action: &str, success: bool, summary: String) -> ExecutionHistoryEntry {
    ExecutionHistoryEntry {
        id: next_history_id(action, "enabled-items"),
        action: action.to_string(),
        target: "enabled-items".to_string(),
        command: None,
        stdout: None,
        stderr: None,
        recorded_at: now_rfc3339(),
        success,
        exit_code: None,
        timed_out: false,
        duration_ms: None,
        summary,
    }
}

pub fn command_entry(
    action: &str,
    target: &str,
    output: &CommandOutput,
    summary: String,
) -> ExecutionHistoryEntry {
    ExecutionHistoryEntry {
        id: next_history_id(action, target),
        action: action.to_string(),
        target: target.to_string(),
        command: Some(output.command.clone()),
        stdout: Some(output.stdout.clone()),
        stderr: Some(output.stderr.clone()),
        recorded_at: now_rfc3339(),
        success: output.exit_code == 0 && !output.timed_out,
        exit_code: Some(output.exit_code),
        timed_out: output.timed_out,
        duration_ms: Some(output.duration_ms),
        summary,
    }
}

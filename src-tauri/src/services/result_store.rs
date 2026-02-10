use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use tauri::AppHandle;

use crate::model::{CheckResult, LatestResultSnapshot, LatestResultState};
use crate::services::config_store;

const RESULT_FILE: &str = "latest-check-results.json";

fn result_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_path = config_store::resolve_config_path(app)?;
    let base_dir = config_path.parent().ok_or_else(|| {
        format!(
            "failed to resolve latest results directory from {}",
            config_path.display()
        )
    })?;
    fs::create_dir_all(base_dir)
        .map_err(|error| format!("failed to create latest results directory: {error}"))?;
    Ok(base_dir.join(RESULT_FILE))
}

fn write_state(app: &AppHandle, state: &LatestResultState) -> Result<(), String> {
    let path = result_path(app)?;
    let payload = serde_json::to_string_pretty(state)
        .map_err(|error| format!("failed to serialize latest results: {error}"))?;
    fs::write(&path, payload)
        .map_err(|error| format!("failed to write latest results to {}: {error}", path.display()))
}

fn to_snapshot(result: &CheckResult) -> LatestResultSnapshot {
    LatestResultSnapshot {
        item_id: result.item_id.clone(),
        checked_at: result.checked_at.clone(),
        has_update: result.has_update,
        current_version: result.current_version.clone(),
        latest_version: result.latest_version.clone(),
        error: result.error.clone(),
    }
}

pub fn load_state(app: &AppHandle) -> Result<LatestResultState, String> {
    let path = result_path(app)?;
    if !path.exists() {
        return Ok(LatestResultState::default());
    }

    let data = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read latest results from {}: {error}",
            path.display()
        )
    })?;

    serde_json::from_str::<LatestResultState>(&data).map_err(|error| {
        format!(
            "failed to parse latest results from {}: {error}",
            path.display()
        )
    })
}

pub fn upsert_result(app: &AppHandle, result: &CheckResult) -> Result<(), String> {
    upsert_results(app, std::slice::from_ref(result))
}

pub fn upsert_results(app: &AppHandle, results: &[CheckResult]) -> Result<(), String> {
    if results.is_empty() {
        return Ok(());
    }

    let mut state = load_state(app)?;
    for result in results {
        state
            .items
            .insert(result.item_id.clone(), to_snapshot(result));
    }
    state.updated_at = Utc::now().to_rfc3339();
    write_state(app, &state)
}

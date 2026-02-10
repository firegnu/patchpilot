use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use crate::model::ExecutionHistoryEntry;

const HISTORY_FILE: &str = "execution-history.json";
const HISTORY_LIMIT: usize = 200;

pub fn history_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("failed to get app config directory: {error}"))?;
    fs::create_dir_all(&app_dir)
        .map_err(|error| format!("failed to create app config directory: {error}"))?;
    Ok(app_dir.join(HISTORY_FILE))
}

fn read_all(app: &AppHandle) -> Result<Vec<ExecutionHistoryEntry>, String> {
    let path = history_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read history from {}: {error}", path.display()))?;
    serde_json::from_str::<Vec<ExecutionHistoryEntry>>(&data)
        .map_err(|error| format!("failed to parse history from {}: {error}", path.display()))
}

fn write_all(app: &AppHandle, entries: &[ExecutionHistoryEntry]) -> Result<(), String> {
    let path = history_path(app)?;
    let payload = serde_json::to_string_pretty(entries)
        .map_err(|error| format!("failed to serialize history: {error}"))?;
    fs::write(&path, payload)
        .map_err(|error| format!("failed to write history to {}: {error}", path.display()))
}

pub fn append_entry(app: &AppHandle, entry: ExecutionHistoryEntry) -> Result<(), String> {
    let mut entries = read_all(app)?;
    entries.insert(0, entry);
    entries.truncate(HISTORY_LIMIT);
    write_all(app, &entries)
}

pub fn load_entries(app: &AppHandle, limit: usize) -> Result<Vec<ExecutionHistoryEntry>, String> {
    let mut entries = read_all(app)?;
    entries.truncate(limit);
    Ok(entries)
}

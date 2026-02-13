use std::collections::HashMap;

use tauri::AppHandle;

use crate::model::{
    AppConfig, CheckResult, CommandOutput, ExecutionHistoryEntry, LatestResultState, SoftwareItem,
    UpdateResult,
};
use crate::services::{
    check_all_guard, check_service, config_store, detect_service, history_events, history_store,
    result_store, shell_runner,
};

fn default_timeout_seconds(config: &AppConfig) -> u64 {
    config.command_timeout_seconds.max(1)
}

fn find_item<'a>(config: &'a AppConfig, item_id: &str) -> Option<&'a SoftwareItem> {
    config.items.iter().find(|item| item.id == item_id)
}

fn is_manual_item(item: &SoftwareItem) -> bool {
    matches!(item.id.as_str(), "brew" | "bun")
}

fn is_auto_cli_item(item: &SoftwareItem) -> bool {
    item.enabled && !is_manual_item(item) && item.kind == "cli"
}

fn is_auto_app_item(item: &SoftwareItem) -> bool {
    item.enabled && !is_manual_item(item) && (item.kind == "gui" || item.kind == "app")
}

fn is_runtime_item(item: &SoftwareItem) -> bool {
    item.enabled && item.kind == "runtime"
}

fn check_item_impl(app: &AppHandle, item_id: &str) -> Result<CheckResult, String> {
    let config = config_store::load_or_init_config(app)?;
    let timeout_seconds = default_timeout_seconds(&config);
    let item = find_item(&config, item_id).ok_or_else(|| format!("item not found: {item_id}"))?;
    let mut execute = |command: &str| shell_runner::run_shell_command(command, timeout_seconds);
    let result = check_service::check_single_item(item, &mut execute);
    if let Err(error) = result_store::upsert_result(app, &result) {
        eprintln!("failed to persist latest result: {error}");
    }
    history_events::append_entry_safe(app, history_events::check_item_entry(&result));
    Ok(result)
}

fn check_items_impl(
    app: &AppHandle,
    action: &str,
    skip_action: &str,
    skip_message: &str,
    filter: fn(&SoftwareItem) -> bool,
) -> Result<Vec<CheckResult>, String> {
    let _guard = match check_all_guard::CheckAllGuard::try_acquire() {
        Some(guard) => guard,
        None => {
            history_events::append_entry_safe(
                app,
                history_events::check_all_entry(
                    skip_action,
                    false,
                    skip_message.to_string(),
                ),
            );
            return Err("check-all is already running".to_string());
        }
    };

    let config = config_store::load_or_init_config(app)?;
    let timeout_seconds = default_timeout_seconds(&config);
    let results: Vec<CheckResult> = config
        .items
        .iter()
        .filter(|item| filter(item))
        .map(|item| {
            let mut execute =
                |command: &str| shell_runner::run_shell_command(command, timeout_seconds);
            check_service::check_single_item(item, &mut execute)
        })
        .collect();

    let error_count = results.iter().filter(|item| item.error.is_some()).count();
    let update_count = results.iter().filter(|item| item.has_update).count();
    if let Err(error) = result_store::upsert_results(app, &results) {
        eprintln!("failed to persist latest results: {error}");
    }
    history_events::append_entry_safe(
        app,
        history_events::check_all_entry(
            action,
            error_count == 0,
            format!(
                "已检查 {} 项，发现 {} 项更新，{} 项错误",
                results.len(),
                update_count,
                error_count
            ),
        ),
    );
    Ok(results)
}

fn check_all_impl(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_items_impl(
        app,
        "check-all",
        "check-all-skip",
        "已跳过：上一轮全量检查仍在运行",
        |item| item.enabled && is_manual_item(item),
    )
}

fn check_auto_items_impl(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_items_impl(
        app,
        "auto-check",
        "auto-check-skip",
        "已跳过：上一轮自动检查仍在运行",
        |item| is_auto_cli_item(item) || is_auto_app_item(item),
    )
}

fn check_auto_cli_items_impl(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_items_impl(
        app,
        "auto-check-cli",
        "auto-check-cli-skip",
        "已跳过：上一轮 CLI 自动检查仍在运行",
        is_auto_cli_item,
    )
}

fn check_auto_app_items_impl(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_items_impl(
        app,
        "auto-check-app",
        "auto-check-app-skip",
        "已跳过：上一轮 App 自动检查仍在运行",
        is_auto_app_item,
    )
}

fn check_runtime_items_impl(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_items_impl(
        app,
        "check-runtime",
        "check-runtime-skip",
        "已跳过：上一轮运行时检查仍在运行",
        is_runtime_item,
    )
}

fn check_everything_impl(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    let mut results = Vec::new();
    results.extend(check_all_impl(app)?);
    results.extend(check_runtime_items_impl(app)?);
    results.extend(check_auto_cli_items_impl(app)?);
    results.extend(check_auto_app_items_impl(app)?);
    Ok(results)
}

fn run_item_update_impl(app: &AppHandle, item_id: &str) -> Result<UpdateResult, String> {
    let config = config_store::load_or_init_config(app)?;
    let timeout_seconds = default_timeout_seconds(&config);
    let item = find_item(&config, item_id).ok_or_else(|| format!("item not found: {item_id}"))?;
    let output = shell_runner::run_shell_command(&item.update_command, timeout_seconds)?;
    history_events::append_entry_safe(
        app,
        history_events::command_entry(
            "run-item-update",
            &item.id,
            &output,
            format!("更新 {}（退出码 {}）", item.name, output.exit_code),
        ),
    );
    Ok(UpdateResult {
        item_id: item_id.to_string(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        output,
    })
}

fn run_ad_hoc_command_impl(app: &AppHandle, command: &str) -> Result<CommandOutput, String> {
    let config = config_store::load_or_init_config(app)?;
    let timeout_seconds = default_timeout_seconds(&config);
    let output = shell_runner::run_shell_command(command, timeout_seconds)?;
    history_events::append_entry_safe(
        app,
        history_events::command_entry(
            "run-shared-command",
            "shared",
            &output,
            format!("共享命令执行完成（退出码 {}）", output.exit_code),
        ),
    );
    Ok(output)
}

fn get_active_node_version_impl() -> String {
    let command = "if command -v node >/dev/null 2>&1; then node --version | sed -E 's/^v//'; else echo ''; fi";
    match shell_runner::run_shell_command(command, 20) {
        Ok(output) if output.exit_code == 0 => output.stdout.trim().to_string(),
        _ => String::new(),
    }
}

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    config_store::load_or_init_config(&app)
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config_store::save_config(&app, &config)
}

#[tauri::command]
pub fn load_latest_results(app: AppHandle) -> Result<LatestResultState, String> {
    result_store::load_state(&app)
}

#[tauri::command]
pub async fn check_item(app: AppHandle, item_id: String) -> Result<CheckResult, String> {
    tauri::async_runtime::spawn_blocking(move || check_item_impl(&app, &item_id))
        .await
        .map_err(|error| format!("check_item task failed: {error}"))?
}

#[tauri::command]
pub async fn check_all(app: AppHandle) -> Result<Vec<CheckResult>, String> {
    tauri::async_runtime::spawn_blocking(move || check_all_impl(&app))
        .await
        .map_err(|error| format!("check_all task failed: {error}"))?
}

#[tauri::command]
pub async fn check_auto_items(app: AppHandle) -> Result<Vec<CheckResult>, String> {
    tauri::async_runtime::spawn_blocking(move || check_auto_items_impl(&app))
        .await
        .map_err(|error| format!("check_auto_items task failed: {error}"))?
}

#[tauri::command]
pub async fn check_auto_cli_items(app: AppHandle) -> Result<Vec<CheckResult>, String> {
    tauri::async_runtime::spawn_blocking(move || check_auto_cli_items_impl(&app))
        .await
        .map_err(|error| format!("check_auto_cli_items task failed: {error}"))?
}

#[tauri::command]
pub async fn check_auto_app_items(app: AppHandle) -> Result<Vec<CheckResult>, String> {
    tauri::async_runtime::spawn_blocking(move || check_auto_app_items_impl(&app))
        .await
        .map_err(|error| format!("check_auto_app_items task failed: {error}"))?
}

#[tauri::command]
pub async fn check_runtime_items(app: AppHandle) -> Result<Vec<CheckResult>, String> {
    tauri::async_runtime::spawn_blocking(move || check_runtime_items_impl(&app))
        .await
        .map_err(|error| format!("check_runtime_items task failed: {error}"))?
}

#[tauri::command]
pub async fn run_item_update(app: AppHandle, item_id: String) -> Result<UpdateResult, String> {
    tauri::async_runtime::spawn_blocking(move || run_item_update_impl(&app, &item_id))
        .await
        .map_err(|error| format!("run_item_update task failed: {error}"))?
}

#[tauri::command]
pub async fn run_ad_hoc_command(app: AppHandle, command: String) -> Result<CommandOutput, String> {
    tauri::async_runtime::spawn_blocking(move || run_ad_hoc_command_impl(&app, &command))
        .await
        .map_err(|error| format!("run_ad_hoc_command task failed: {error}"))?
}

#[tauri::command]
pub async fn get_active_node_version() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(get_active_node_version_impl)
        .await
        .map_err(|error| format!("get_active_node_version task failed: {error}"))
}

#[tauri::command]
pub fn load_history(
    app: AppHandle,
    limit: Option<u32>,
) -> Result<Vec<ExecutionHistoryEntry>, String> {
    let requested = limit.unwrap_or(50).clamp(1, 200) as usize;
    history_store::load_entries(&app, requested)
}

fn detect_installed_items_impl(app: &AppHandle) -> Result<HashMap<String, bool>, String> {
    let config = config_store::load_or_init_config(app)?;
    Ok(detect_service::detect_all(&config.items))
}

#[tauri::command]
pub async fn detect_installed_items(app: AppHandle) -> Result<HashMap<String, bool>, String> {
    tauri::async_runtime::spawn_blocking(move || detect_installed_items_impl(&app))
        .await
        .map_err(|error| format!("detect_installed_items task failed: {error}"))?
}

pub fn check_manual_items_for_menu(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_all_impl(app)
}

pub fn check_single_item_for_menu(app: &AppHandle, item_id: &str) -> Result<CheckResult, String> {
    check_item_impl(app, item_id)
}

pub fn check_cli_items_for_menu(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_auto_cli_items_impl(app)
}

pub fn check_app_items_for_menu(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_auto_app_items_impl(app)
}

pub fn check_everything_for_menu(app: &AppHandle) -> Result<Vec<CheckResult>, String> {
    check_everything_impl(app)
}

pub fn run_item_update_for_menu(app: &AppHandle, item_id: &str) -> Result<UpdateResult, String> {
    run_item_update_impl(app, item_id)
}

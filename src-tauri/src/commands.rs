use tauri::AppHandle;

use crate::model::{
    AppConfig, CheckResult, CommandOutput, ExecutionHistoryEntry, SoftwareItem, UpdateResult,
};
use crate::services::{
    check_all_guard, check_service, config_store, history_events, history_store, shell_runner,
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

fn check_item_impl(app: &AppHandle, item_id: &str) -> Result<CheckResult, String> {
    let config = config_store::load_or_init_config(app)?;
    let timeout_seconds = default_timeout_seconds(&config);
    let item = find_item(&config, item_id).ok_or_else(|| format!("item not found: {item_id}"))?;
    let mut execute = |command: &str| shell_runner::run_shell_command(command, timeout_seconds);
    let result = check_service::check_single_item(item, &mut execute);
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
        |item| item.enabled && !is_manual_item(item),
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

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    config_store::load_or_init_config(&app)
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config_store::save_config(&app, &config)
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
pub fn load_history(
    app: AppHandle,
    limit: Option<u32>,
) -> Result<Vec<ExecutionHistoryEntry>, String> {
    let requested = limit.unwrap_or(50).clamp(1, 200) as usize;
    history_store::load_entries(&app, requested)
}

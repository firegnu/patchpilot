#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod model;
mod software_catalog;
mod services;

use std::process::Command as ProcessCommand;
use std::sync::Mutex;

use chrono::{DateTime, Local};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use model::{AppConfig, LatestResultState};
use services::{config_store, history_store, result_store};

const TRAY_ID: &str = "patchpilot-tray";

#[derive(Debug, Clone, Default)]
struct TrayRuntimeState {
    last_notice: Option<String>,
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn parse_local_time(raw: &str) -> String {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Local).format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| raw.to_string())
}

fn latest_checked_time(config: &AppConfig, latest: &LatestResultState) -> Option<String> {
    config
        .items
        .iter()
        .filter(|item| item.enabled)
        .filter_map(|item| latest.items.get(&item.id))
        .map(|snapshot| snapshot.checked_at.clone())
        .max()
}

fn collect_error_count(config: &AppConfig, latest: &LatestResultState) -> usize {
    config
        .items
        .iter()
        .filter(|item| item.enabled)
        .filter_map(|item| latest.items.get(&item.id))
        .filter(|snapshot| snapshot.error.is_some())
        .count()
}

fn with_state<R>(app: &AppHandle, f: impl FnOnce(&TrayRuntimeState) -> R) -> Option<R> {
    let state = app.try_state::<Mutex<TrayRuntimeState>>()?;
    let guard = state.lock().ok()?;
    Some(f(&guard))
}

fn with_state_mut<R>(app: &AppHandle, f: impl FnOnce(&mut TrayRuntimeState) -> R) -> Option<R> {
    let state = app.try_state::<Mutex<TrayRuntimeState>>()?;
    let mut guard = state.lock().ok()?;
    Some(f(&mut guard))
}

fn set_notice(app: &AppHandle, text: impl Into<String>) {
    let _ = with_state_mut(app, |state| {
        state.last_notice = Some(text.into());
    });
}

fn state_snapshot(app: &AppHandle) -> TrayRuntimeState {
    with_state(app, Clone::clone).unwrap_or_default()
}

fn menu_item(
    app: &AppHandle,
    id: &str,
    label: impl Into<String>,
    enabled: bool,
) -> Result<MenuItem<tauri::Wry>, String> {
    let text = label.into();
    MenuItem::with_id(app, id, &text, enabled, None::<&str>).map_err(|error| error.to_string())
}

fn append_separator(menu: &Menu<tauri::Wry>, app: &AppHandle) -> Result<(), String> {
    let separator = PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?;
    menu.append(&separator).map_err(|error| error.to_string())
}

fn build_interval_submenu(
    app: &AppHandle,
    current_interval: u64,
    enabled: bool,
) -> Result<Submenu<tauri::Wry>, String> {
    let submenu = Submenu::with_id(app, "menu.interval", "检查频率", true)
        .map_err(|error| error.to_string())?;
    for (minutes, label) in [(240_u64, "4h"), (480_u64, "8h"), (720_u64, "12h")] {
        let title = if current_interval == minutes {
            format!("✓ {label}")
        } else {
            label.to_string()
        };
        let item = menu_item(app, &format!("menu.interval.{minutes}"), title, enabled)?;
        submenu.append(&item).map_err(|error| error.to_string())?;
    }
    Ok(submenu)
}

fn build_theme_submenu(
    app: &AppHandle,
    theme_mode: &str,
    enabled: bool,
) -> Result<Submenu<tauri::Wry>, String> {
    let submenu =
        Submenu::with_id(app, "menu.theme", "主题", true).map_err(|error| error.to_string())?;
    for (mode, label) in [
        ("system", "跟随系统"),
        ("light", "浅色"),
        ("dark", "深色"),
    ] {
        let title = if theme_mode == mode {
            format!("✓ {label}")
        } else {
            label.to_string()
        };
        let item = menu_item(app, &format!("menu.theme.{mode}"), title, enabled)?;
        submenu.append(&item).map_err(|error| error.to_string())?;
    }
    Ok(submenu)
}

fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let config = config_store::load_or_init_config(app)?;
    let latest = result_store::load_state(app).unwrap_or_default();
    let state = state_snapshot(app);
    let error_count = collect_error_count(&config, &latest);
    let checked_at = latest_checked_time(&config, &latest)
        .as_deref()
        .map(parse_local_time)
        .unwrap_or_else(|| "尚未检查".to_string());

    let mut status_line = format!("状态：上次检查 {checked_at} | 错误 {error_count}");
    if let Some(text) = &state.last_notice {
        status_line = format!("{status_line} | {text}");
    }

    let menu = Menu::new(app).map_err(|error| error.to_string())?;
    let status = menu_item(app, "menu.status", status_line, false)?;
    menu.append(&status).map_err(|error| error.to_string())?;

    let open = menu_item(app, "menu.open_window", "打开主窗口", true)?;
    menu.append(&open).map_err(|error| error.to_string())?;
    append_separator(&menu, app)?;

    let auto_label = if config.auto_check_enabled {
        "自动检查：开启（点击暂停）"
    } else {
        "自动检查：暂停（点击开启）"
    };
    let auto_toggle = menu_item(app, "menu.auto.toggle", auto_label, true)?;
    menu.append(&auto_toggle).map_err(|error| error.to_string())?;
    let interval = build_interval_submenu(app, config.check_interval_minutes, true)?;
    menu.append(&interval).map_err(|error| error.to_string())?;

    append_separator(&menu, app)?;
    let theme = build_theme_submenu(app, &config.theme_mode, true)?;
    menu.append(&theme).map_err(|error| error.to_string())?;
    let open_config = menu_item(app, "menu.open.config", "打开配置文件", true)?;
    menu.append(&open_config).map_err(|error| error.to_string())?;
    let open_logs = menu_item(app, "menu.open.logs", "打开日志目录", true)?;
    menu.append(&open_logs).map_err(|error| error.to_string())?;

    append_separator(&menu, app)?;
    let about = menu_item(app, "menu.about", "关于 PatchPilot", true)?;
    menu.append(&about).map_err(|error| error.to_string())?;
    let quit = menu_item(app, "menu.quit", "退出", true)?;
    menu.append(&quit).map_err(|error| error.to_string())?;

    Ok(menu)
}

fn build_fallback_menu(app: &AppHandle, status: &str) -> Result<Menu<tauri::Wry>, String> {
    let menu = Menu::new(app).map_err(|error| error.to_string())?;
    let status_item = menu_item(app, "menu.status", status, false)?;
    menu.append(&status_item).map_err(|error| error.to_string())?;
    let open = menu_item(app, "menu.open_window", "打开主窗口", true)?;
    menu.append(&open).map_err(|error| error.to_string())?;
    append_separator(&menu, app)?;
    let quit = menu_item(app, "menu.quit", "退出", true)?;
    menu.append(&quit).map_err(|error| error.to_string())?;
    Ok(menu)
}

fn refresh_tray_menu(app: &AppHandle) {
    let menu = match build_tray_menu(app) {
        Ok(menu) => menu,
        Err(error) => {
            eprintln!("failed to build tray menu: {error}");
            set_notice(app, format!("菜单构建失败：{error}"));
            match build_fallback_menu(app, "菜单加载失败，请打开主窗口排查") {
                Ok(menu) => menu,
                Err(_) => return,
            }
        }
    };

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_menu(Some(menu));
    }
}

fn emit_config_event(app: &AppHandle) {
    let _ = app.emit("patchpilot://config-updated", ());
}

fn emit_theme_mode_event(app: &AppHandle, mode: &str) {
    let _ = app.emit("patchpilot://theme-mode-updated", mode.to_string());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("patchpilot://theme-mode-updated", mode.to_string());
        let script = match mode {
            "light" => {
                "document.documentElement.setAttribute('data-theme','light');document.documentElement.style.colorScheme='light';"
            }
            "dark" => {
                "document.documentElement.setAttribute('data-theme','dark');document.documentElement.style.colorScheme='dark';"
            }
            _ => {
                "const d=document.documentElement;const dark=window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)').matches;const t=dark?'dark':'light';d.setAttribute('data-theme',t);d.style.colorScheme=t;"
            }
        };
        let _ = window.eval(script);
    }
}

fn with_config_mutation(
    app: &AppHandle,
    mutate: impl FnOnce(&mut AppConfig),
) -> Result<AppConfig, String> {
    let mut config = config_store::load_or_init_config(app)?;
    mutate(&mut config);
    config_store::save_config(app, &config)?;
    Ok(config)
}

fn open_with_system(target: &str) -> Result<(), String> {
    let status = ProcessCommand::new("open")
        .arg(target)
        .status()
        .map_err(|error| format!("failed to open {target}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("failed to open {target}: exit code {:?}", status.code()))
    }
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "menu.open_window" => show_main_window(app),
        "menu.auto.toggle" => {
            match with_config_mutation(app, |config| {
                config.auto_check_enabled = !config.auto_check_enabled;
            }) {
                Ok(config) => {
                    set_notice(
                        app,
                        if config.auto_check_enabled {
                            "自动检查已开启"
                        } else {
                            "自动检查已暂停"
                        },
                    );
                    emit_config_event(app);
                }
                Err(error) => set_notice(app, format!("修改自动检查失败：{error}")),
            }
            refresh_tray_menu(app);
        }
        "menu.interval.240" | "menu.interval.480" | "menu.interval.720" => {
            let next = id
                .strip_prefix("menu.interval.")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(480);
            match with_config_mutation(app, |config| {
                config.check_interval_minutes = next;
            }) {
                Ok(_) => {
                    set_notice(app, format!("检查频率已设置为 {}h", next / 60));
                    emit_config_event(app);
                }
                Err(error) => set_notice(app, format!("设置检查频率失败：{error}")),
            }
            refresh_tray_menu(app);
        }
        "menu.theme.system" | "menu.theme.light" | "menu.theme.dark" => {
            let mode = id.strip_prefix("menu.theme.").unwrap_or("system");
            match with_config_mutation(app, |config| {
                config.theme_mode = mode.to_string();
            }) {
                Ok(_) => {
                    set_notice(app, format!("主题已切换为 {mode}"));
                    emit_config_event(app);
                    emit_theme_mode_event(app, mode);
                }
                Err(error) => set_notice(app, format!("切换主题失败：{error}")),
            }
            refresh_tray_menu(app);
        }
        "menu.open.config" => {
            match config_store::resolve_config_path(app)
                .and_then(|path| open_with_system(&path.to_string_lossy()))
            {
                Ok(_) => set_notice(app, "已打开配置文件"),
                Err(error) => set_notice(app, format!("打开配置文件失败：{error}")),
            }
            refresh_tray_menu(app);
        }
        "menu.open.logs" => {
            let logs_dir = history_store::history_path(app)
                .ok()
                .and_then(|path| path.parent().map(|parent| parent.to_path_buf()));
            match logs_dir {
                Some(path) => match open_with_system(&path.to_string_lossy()) {
                    Ok(_) => set_notice(app, "已打开日志目录"),
                    Err(error) => set_notice(app, format!("打开日志目录失败：{error}")),
                },
                None => set_notice(app, "日志目录不存在"),
            }
            refresh_tray_menu(app);
        }
        "menu.about" => {
            match open_with_system("https://github.com/firegnu/patchpilot") {
                Ok(_) => set_notice(app, "已打开关于页面"),
                Err(error) => set_notice(app, format!("打开关于页面失败：{error}")),
            }
            refresh_tray_menu(app);
        }
        "menu.quit" => app.exit(0),
        _ => {}
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            app.manage(Mutex::new(TrayRuntimeState::default()));
            let app_handle = app.handle().clone();
            let tray_icon =
                Image::from_bytes(include_bytes!("../icons/tray-template.png")).map_err(|error| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("failed to load tray template icon: {error}"),
                    )
                })?;
            let menu = match build_tray_menu(&app_handle) {
                Ok(menu) => menu,
                Err(error) => {
                    eprintln!("failed to build initial tray menu: {error}");
                    match build_fallback_menu(&app_handle, "菜单初始化失败，请打开主窗口排查") {
                        Ok(menu) => menu,
                        Err(inner_error) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                inner_error,
                            )
                            .into())
                        }
                    }
                }
            };

            let mut builder = TrayIconBuilder::with_id(TRAY_ID)
                .tooltip("PatchPilot")
                .show_menu_on_left_click(true)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    let id = event.id().as_ref().to_string();
                    handle_menu_event(app, &id);
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button_state, .. } = event {
                        if button_state == MouseButtonState::Up {
                            refresh_tray_menu(tray.app_handle());
                        }
                    }
                });
            builder = builder.icon(tray_icon);
            builder.build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::load_latest_results,
            commands::check_item,
            commands::check_all,
            commands::check_auto_items,
            commands::check_auto_cli_items,
            commands::check_auto_app_items,
            commands::check_runtime_items,
            commands::run_item_update,
            commands::run_ad_hoc_command,
            commands::get_active_node_version,
            commands::load_history,
            commands::detect_installed_items
        ])
        .run(tauri::generate_context!())
        .expect("error while running patchpilot");
}

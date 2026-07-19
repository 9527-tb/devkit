//! DevKit Tauri 库入口。
//!
//! - `setup`：托盘、开机自启、更新插件
//! - `on_window_event`：关闭到托盘
//! - `invoke_handler`：前端 IPC 命令注册表（实现见 `commands` 模块）

mod platform;
mod core;
mod providers;
mod toolchains;
mod settings;
mod deploy;
mod tools;
mod commands;
mod deps;
mod health;
mod models;
mod process;
mod runtime;
mod workspace_config;
mod tray;

use process::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 注册内置 Provider / 工具箱子系统
    providers::register_builtin();
    tools::init_tools_subsystem();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&'static str>>,
        ))
        .manage(AppState::default())
        .setup(|app| {
            #[cfg(desktop)]
            {
                if let Err(e) = app
                    .handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())
                {
                    eprintln!("[DevKit] updater plugin failed: {e}");
                }
            }
            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("[DevKit] tray setup failed: {e}");
            }
            // 按已保存配置同步开机自启
            if let Ok(settings) = runtime::load_settings(app.handle()) {
                let _ = runtime::sync_launch_at_login(app.handle(), settings.general.launch_at_login);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "main" {
                    return;
                }
                api.prevent_close();
                if runtime::close_to_tray_enabled(window.app_handle()) {
                    // 默认：最小化到托盘
                    let _ = window.hide();
                } else {
                    // 直接退出模式：仍走统一退出确认（可能有运行中实例）
                    tray::request_quit(window.app_handle());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_projects,
            commands::run_action,
            commands::stop_project,
            commands::stop_instance,
            commands::stop_all_processes,
            commands::count_running_processes,
            commands::exit_app,
            commands::request_app_quit,
            commands::running_processes,
            commands::process_metrics,
            commands::project_logs,
            commands::clear_logs,
            commands::refresh_dependencies,
            commands::validate_runtime_path,
            commands::detect_runtimes,
            commands::detect_node_package_managers,
            commands::save_runtime_settings,
            commands::load_runtime_settings,
            commands::runtime_settings_exists,
            commands::initialize_runtime_settings,
            commands::list_providers,
            commands::get_capabilities,
            commands::workspace_health_check,
            commands::open_in_editor,
            commands::open_in_terminal,
            commands::detect_external_tools,
            commands::workspace_git_status,
            commands::load_action_prefs,
            commands::save_action_prefs,
            commands::check_outdated_deps,
            commands::load_workspace_config,
            commands::save_workspace_config,
            commands::deploy_ssh_upload,
            // 工具箱（各工具独立命令）
            tools::commands::list_tools,
            tools::ports::commands::tool_ports_lookup,
            tools::ports::commands::tool_ports_kill,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // macOS：点击 Dock 图标时恢复主窗口（关闭到托盘后窗口已 hide）
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = &event {
                tray::show_main_window(app_handle);
            }
            // Cmd+Q / 系统退出：交给前端确认；exit_app(0) 带 code 时放行
            if let tauri::RunEvent::ExitRequested { api, code, .. } = &event {
                if code.is_none() {
                    api.prevent_exit();
                    tray::request_quit(app_handle);
                }
            }
        });
}

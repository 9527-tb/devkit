//! Tauri IPC 薄封装层。
//!
//! 约定：本模块只做参数转发与类型映射，业务逻辑在 `core` / `providers` /
//! `process` / `settings` 等子模块。前端通过 `@tauri-apps/api` 调用此处注册的 command。
//!
//! TODO(rs-commands-thin): 拆为 projects/process/settings/capabilities 子模块 — DESIGN R2
//! TODO(rs-capabilities): get_capabilities 完整 Capability 合并 — DESIGN §8.1

use crate::core::caps;
use crate::core::registry::ProviderRegistry;
use crate::core::scan_engine;
use crate::deploy::targets::ssh::{SshUploadRequest, SshUploadResult};
use crate::models::{
    ActionPrefs, Dependency, GitStatus, HealthReport, OutdatedDependency, ProcessView, Project,
};
use crate::workspace_config::WorkspaceConfig;
use crate::process::{self, AppState};
use crate::providers::maven;
use crate::runtime;
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, State};

// ── 工作区扫描 ──────────────────────────────────────────────

#[tauri::command]
pub fn scan_projects(root: String) -> Result<Vec<Project>, String> {
    // 经 scan_engine 转发，R2 完全迁入后删除对 scan 的直接依赖
    crate::core::scan_engine::scan_workspace(root)
}

// ── 进程 / 日志 ─────────────────────────────────────────────

#[tauri::command]
pub async fn run_action(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    action: String,
    kind: String,
) -> Result<ProcessView, String> {
    // 解析路径 / 端口探测等可能较慢，必须离开 UI 线程，否则整窗卡死
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        process::run_action(app, &state, path, action, kind)
    })
    .await
    .map_err(|e| format!("执行中断: {e}"))?
}

#[tauri::command]
pub async fn run_pipeline(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    kind: String,
    steps: Vec<process::PipelineStep>,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        process::run_pipeline(app, &state, path, kind, steps)
    })
    .await
    .map_err(|e| format!("流水线执行中断: {e}"))?
}

#[tauri::command]
pub async fn stop_project(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    kind: String,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        process::stop_project(&app, &state, path, kind)
    })
    .await
    .map_err(|e| format!("停止中断: {e}"))?
}

#[tauri::command]
pub async fn stop_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    pid: u32,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        process::stop_instance(&app, &state, path, pid)
    })
    .await
    .map_err(|e| format!("停止中断: {e}"))?
}

/// 停止全部托管实例。
#[tauri::command]
pub async fn stop_all_processes(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || process::stop_all(&app, &state))
        .await
        .map_err(|e| format!("停止中断: {e}"))?
}

/// 当前运行中实例数（PID 去重）。
#[tauri::command]
pub fn count_running_processes(state: State<AppState>) -> u32 {
    process::count_running(state.inner())
}

/// 确认后真正退出应用。
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

/// 从前端主动发起退出确认流程（与托盘退出一致）。
#[tauri::command]
pub fn request_app_quit(app: AppHandle) {
    crate::tray::request_quit(&app);
}

#[tauri::command]
pub async fn running_processes(
    state: State<'_, AppState>,
) -> Result<HashMap<String, Vec<ProcessView>>, String> {
    // 同步 command 会占 UI 线程；进程枚举必须放到 blocking 池
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || process::running_processes(&state))
        .await
        .map_err(|e| format!("读取进程中断: {e}"))
}

#[tauri::command]
pub fn process_metrics(
    state: State<AppState>,
    pid: u32,
) -> Result<crate::platform::ProcessMetrics, String> {
    process::process_metrics(state.inner(), pid)
}

#[tauri::command]
pub fn project_logs(state: State<AppState>, path: String, kind: String) -> Vec<String> {
    process::project_logs(state.inner(), path, kind)
}

#[tauri::command]
pub fn clear_logs(state: State<AppState>, path: String, kind: String) -> Result<(), String> {
    process::clear_logs(state.inner(), path, kind)
}

// ── 工作区健康 / 外部工具 ───────────────────────────────────

#[tauri::command]
pub fn workspace_health_check(
    app: AppHandle,
    state: State<AppState>,
    root: String,
    projects: Vec<Project>,
) -> Result<HealthReport, String> {
    crate::health::check_workspace(&app, state.inner(), root, projects)
}

#[tauri::command]
pub fn open_in_editor(app: AppHandle, path: String) -> Result<(), String> {
    let settings = runtime::load_settings(&app)?;
    let editor = settings.general.editor_command.trim();
    crate::platform::open_in_editor(
        &path,
        if editor.is_empty() {
            None
        } else {
            Some(editor)
        },
    )
}

#[tauri::command]
pub fn open_in_terminal(app: AppHandle, path: String) -> Result<(), String> {
    let settings = runtime::load_settings(&app)?;
    let term = settings.general.terminal_app.trim();
    crate::platform::open_in_terminal(
        &path,
        if term.is_empty() {
            None
        } else {
            Some(term)
        },
    )
}

#[tauri::command]
pub fn detect_external_tools() -> crate::platform::external_tools::DetectedExternalTools {
    crate::platform::detect_external_tools()
}

#[tauri::command]
pub fn workspace_git_status(root: String) -> GitStatus {
    crate::platform::workspace_git_status(&root)
}

// ── 用户偏好 / 工作区配置 ───────────────────────────────────

#[tauri::command]
pub fn load_action_prefs() -> Result<ActionPrefs, String> {
    crate::settings::action_prefs::load_action_prefs()
}

#[tauri::command]
pub fn save_action_prefs(prefs: ActionPrefs) -> Result<(), String> {
    crate::settings::action_prefs::save_action_prefs(&prefs)
}

#[tauri::command]
pub fn load_workspace_config(root: String) -> Result<Option<WorkspaceConfig>, String> {
    crate::workspace_config::load_workspace_config(root)
}

#[tauri::command]
pub fn save_workspace_config(root: String, config: WorkspaceConfig) -> Result<(), String> {
    crate::workspace_config::save_workspace_config(root, config)
}

#[tauri::command]
pub fn deploy_ssh_upload(request: SshUploadRequest) -> SshUploadResult {
    crate::deploy::targets::ssh::ssh_upload(request)
}

// ── 依赖 / 运行时设置 / Provider ────────────────────────────

#[tauri::command]
pub fn check_outdated_deps(
    app: AppHandle,
    path: String,
    kind: String,
) -> Result<Vec<OutdatedDependency>, String> {
    let kind = kind.trim().to_ascii_lowercase();
    if kind != "node" {
        return Err(format!("暂不支持 {kind} 的 outdated 检查（当前仅 Node）"));
    }
    let settings = runtime::load_settings(&app)?;
    let pm = runtime::normalize_node_package_manager(&settings.node.package_manager);
    crate::deps::node_outdated(Path::new(&path), pm)
}

#[tauri::command]
pub fn refresh_dependencies(path: String, kind: String) -> Result<Vec<Dependency>, String> {
    let project = scan_engine::project_at(Path::new(&path), &kind).ok_or("无法识别项目")?;
    if project.kind == "maven" {
        maven::deps::refresh_tree(Path::new(&path))
    } else {
        Ok(project.dependencies)
    }
}

#[tauri::command]
pub fn load_runtime_settings(app: AppHandle) -> Result<crate::runtime::RuntimeSettings, String> {
    crate::runtime::load_settings(&app)
}

#[tauri::command]
pub fn runtime_settings_exists(app: AppHandle) -> Result<bool, String> {
    crate::runtime::settings_file_exists(&app)
}

#[tauri::command]
pub fn initialize_runtime_settings(
    app: AppHandle,
) -> Result<crate::runtime::RuntimeSettings, String> {
    crate::runtime::initialize_runtime_settings(&app)
}

#[tauri::command]
pub fn save_runtime_settings(
    app: AppHandle,
    settings: crate::runtime::RuntimeSettings,
) -> Result<(), String> {
    crate::runtime::save_settings(&app, &settings)
}

#[tauri::command]
pub fn detect_runtimes() -> crate::runtime::DetectedRuntimes {
    crate::runtime::detect_all()
}

/// 探测系统已安装的 npm / pnpm / yarn。
#[tauri::command]
pub fn detect_node_package_managers() -> Vec<String> {
    crate::runtime::detect_node_package_managers()
}

#[tauri::command]
pub fn validate_runtime_path(kind: String, path: String) -> Result<crate::runtime::RuntimeEntry, String> {
    match kind.as_str() {
        "jdk" | "java" => crate::runtime::validate_manual_jdk(&path),
        "node" => crate::runtime::validate_manual_node(&path),
        _ => Err("未知运行时类型".into()),
    }
}

/// 列出已注册 Provider kind（前端侧栏/设置动态段用）。
#[tauri::command]
pub fn list_providers() -> Vec<String> {
    ProviderRegistry::new().list_kinds()
}

/// 按项目 kind 返回能力列表（面板 / Action 注册）。
#[tauri::command]
pub fn get_capabilities(kind: String) -> Vec<String> {
    caps::capabilities_for_kind(&kind)
}

# DevKit 重构功能点 TODO 总表

> 依据重构开发计划与 [DESIGN.md](./DESIGN.md)（§12 / §16 / §17）。  
> 源码内 `// TODO(feature-id): …` / `DONE(feature-id)` 须与本表 **一一对应**。  
> 状态：`pending` | `in_progress` | `done`

---

## 图例

| 前缀 | 含义 |
|------|------|
| `chore-` | 规范 / 清理 |
| `fe-` | 前端功能 |
| `ux-` | 对齐原型 UX |
| `rs-` | Rust / IPC |
| `plat-` | 跨平台 |
| `deploy-` | 部署规划 |

---

## 基础 / 清理

| id | 阶段 | 说明 | 主要文件 | 状态 |
|----|------|------|----------|------|
| `chore-comment-standard` | R0 | 中文文件头 + TODO 规范落地 | 全仓新增/迁出模块 | done |
| `chore-dead-code` | R2 | 删除 `command_for` 与旧 `scan.rs` | `core/scan_engine` + providers | done |
| `chore-remove-monolith` | R1–R4 | App.vue 壳化；Workbench/Settings 拆出 | `src/App.vue` ≤150 行 | done |

---

## 前端（fe-）

| id | 阶段 | 说明 | 主要文件 | 状态 |
|----|------|------|----------|------|
| `fe-app-shell` | R0–R3 | App 仅 ConfigProvider + view 切换 | `src/App.vue`、`src/api/tauri.js` | done |
| `fe-settings-store` | R1 | settings store | `src/stores/settings.js` | done |
| `fe-settings-view` | R1 | 设置页布局 + 导航 | `SettingsView` / `SettingsNav` | done |
| `fe-settings-general` | R1 | 基本配置面板 | `SettingsGeneral.vue` | done |
| `fe-settings-java` | R1 | Java/JDK/Maven 设置 | `SettingsToolchain.vue` | done |
| `fe-settings-node` | R1 | Node 设置 | `SettingsToolchain.vue` | done |
| `fe-settings-toolchain-dynamic` | R5 | `list_providers` 动态导航段 | `SettingsNav.vue` | done |
| `fe-init-modal` | R1 | 首次初始化弹窗 | `features/init/*` | done |
| `fe-theme-apply` | R0–R1 | 主题应用 | `themes/` + settings store | done |
| `fe-topbar-history` | R3 | Topbar Compact + 历史 | `Topbar.vue` | done |
| `fe-sidebar` | R3 | 侧栏分组 | `Sidebar.vue` | done |
| `fe-workbench-view` | R3 | Workbench 布局 | `WorkbenchView.vue` | done |
| `fe-panel-registry` | R4 | panelId → 组件 | `panels/registry.js`、`PanelHost.vue` | done |
| `fe-panel-logs` | R4 | 日志面板 | `LogsPanel.vue` | done |
| `fe-panel-deps` | R4 | 依赖面板 | `DepsPanel.vue` | done |
| `fe-i18n-complete` | R7 | locales 拆分 | `src/i18n/locales/*` | done |

---

## UX 对齐原型（ux-）

| id | 阶段 | 说明 | 主要文件 | 状态 |
|----|------|------|----------|------|
| `ux-project-tabs` | R4 | 多项目标签 + 溢出 + 跳转 | `ProjectTabs.vue` | done |
| `ux-close-confirm` | R4 | 关闭运行中确认 | `CloseTabConfirm.vue` | done |
| `ux-action-overflow` | R4 | Action 前 3 + ⋯ | `ActionBar.vue` | done |
| `ux-project-header` | R4 | 项目头 / runtime pill | `ProjectHeader.vue` | done |
| `ux-sidebar-filter` | R4 | 侧栏筛选 | `Sidebar.vue` | done |
| `ux-monitor` | R5 | Monitor 面板（Host 可先 mock） | `MonitorPanel.vue` | done |

---

## Rust / IPC（rs-）

| id | 阶段 | 说明 | 主要文件 | 状态 |
|----|------|------|----------|------|
| `rs-scan-engine` | R2 | Scan Engine | `core/scan_engine.rs` | done |
| `rs-registry` | R2 | ProviderRegistry | `core/registry.rs` | done |
| `rs-provider-node` | R2 | Node Provider | `providers/node/*` | done |
| `rs-provider-maven` | R2 | Maven Provider | `providers/maven/*` | done |
| `rs-provider-cargo` | R6 | Cargo Provider | `providers/cargo/*` | done |
| `rs-commands-thin` | R2 | commands 薄层 | `commands/mod.rs` | done |
| `rs-instance-key` | R3 | projectKey 进程主键 + 双读 | `process/`、`project_ref` | done |
| `rs-capabilities` | R4 | `get_capabilities` IPC | `commands` + `caps` | done |
| `rs-monitor-host` | R5 | Host 采样（前端 mock 先行） | `MonitorPanel.vue` | done |
| `rs-tests-core` | R7 | scan / projectKey 单测 | `scan_engine` / `project_ref` tests | done |
| `rs-init-progress` | R1 | init-progress 订阅 | `api/settings.js`、InitModal | done |

---

## 跨平台（plat-）

| id | 阶段 | 说明 | 主要文件 | 状态 |
|----|------|------|----------|------|
| `plat-windows` | R5 | Win 路径 / taskkill / netstat | `platform/*` | done |
| `plat-linux` | R5 | kill 进程组 / lsof | `platform/*` | done |

---

## 部署预留（deploy-）

| id | 阶段 | 说明 | 主要文件 | 状态 |
|----|------|------|----------|------|
| `deploy-engine-skel` | R8 | Deploy Engine + SecretRef 骨架 | `src-tauri/.../deploy/` | done |
| `deploy-panel-stub` | R8 | `panel.deploy` 占位 | `DeployPanel.vue` | done |
| `deploy-target-ssh` | R9 | SSH Target 占位 | `deploy/targets/ssh` | done |
| `deploy-target-jenkins` | R10 | Jenkins Adapter 占位 | `deploy/targets/jenkins` | done |

---

## 阶段验收摘要

| 阶段 | 结果 |
|------|------|
| R0 | 目录空壳、themes/i18n/logFormat 迁出、REFACTOR_TODOS |
| R1 | Settings/Init 拆出；App 壳 |
| R2 | Scan Engine + Node/Maven Provider；删 `scan.rs`/`command_for` |
| R3 | Workbench/Topbar/Sidebar；projectKey |
| R4 | Tabs/ActionBar/CloseConfirm/Panels |
| R5 | Monitor；Win/Linux platform；动态设置导航 |
| R6 | Cargo Provider |
| R7 | i18n locales；核心单测；README |
| R8+ | deploy 骨架（无真实远程副作用） |

---

## 维护约定

1. 新增功能点：先在本表加行，再在源码写 `TODO(id)`。  
2. 完成实现：源码 TODO 改为 `DONE` 或删除，并将本表状态改为 `done`。  
3. 阶段门禁：全局搜索 `TODO(fe-` / `TODO(rs-` 等，禁止未登记 id。

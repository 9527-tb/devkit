# DevKit

Monorepo 项目工作台：扫描 Node / Maven / Cargo 项目，运行动作、流式日志、依赖树与设置管理。

技术栈：**Tauri 2** + **Vue 3** + **antdv-next**。

## 开发

```bash
pnpm install
pnpm tauri dev
```

仅前端预览（无后端 IPC）：

```bash
pnpm dev
```

## 架构要点

- 前端：`App.vue` 壳 + `views/` + `features/` + `stores/`（轻量，无 Pinia）
- 后端：`core/` 扫描引擎与注册表、`providers/{node,maven,cargo}`、`process/`、`settings/`、`platform/`、`deploy/`（骨架）
- 身份键：`projectKey = path::kind`（同目录多 Kind 不串日志）
- UX 真源：[prototype/devkit-ui.html](prototype/devkit-ui.html)
- 设计：[docs/DESIGN.md](docs/DESIGN.md)
- 重构功能点：[docs/REFACTOR_TODOS.md](docs/REFACTOR_TODOS.md)

## 测试

```bash
cd src-tauri && cargo test
```

## 配置

用户设置写入：

- macOS / Linux: `~/.devkit/settings.json`
- Windows: `%APPDATA%\devkit\settings.json`

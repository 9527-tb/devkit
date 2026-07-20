# DevKit 开发效率增强设计

> 版本：2026-07-19  
> 状态：规划稿（未排期实现）  
> 范围：在现有工作台能力之上，提升「扫仓 → 跑起来 → 看清失败 → 少切窗口」的效率  
> 关联：[DESIGN.md](./DESIGN.md)（架构基线）、[README.md](../README.md)（产品说明）  
> **明确不做（本期讨论已排除）**：自动检测并领养「外部启动」进程；本文不展开该能力。

---

## 1. 文档目的

回答三件事：

1. **解决什么问题**——开发者日常卡在哪  
2. **在什么地方改**——前端 / Rust / 配置 / UX 落点  
3. **如何调整**——交互、数据、IPC、分期建议  

本文是产品与实现的对照清单，不是替代 [DESIGN.md](./DESIGN.md) 的架构总纲。新增 Kind / Deploy 适配器仍遵循 DESIGN 的 Provider / Adapter 原则。

---

## 2. 现状与缺口（简表）

| 已有 | 缺口 |
|------|------|
| 扫描 Node / Maven / Cargo / Gradle | 启动前环境/端口是否健康不透明 |
| 动作条执行 + 流式日志 | 失败难定位；日志弱过滤 |
| 托管进程监控 / 停止 | 无「组合流水线」、无最近命令 |
| 端口工具（查/杀） | 与「即将启动的项目」未联动 |
| 工具链设置（Java / Node） | 无一键「在编辑器/终端打开」 |
| 依赖树面板 | 无 outdated / 冲突只读提示 |
| 部署骨架 | 无可交付的最小部署路径 |

**产品定位约束**：继续做「本地工作台」，不做第二套 IDE / 完整 Git 客户端 / 全功能调试器。

---

## 3. 设计原则（本增强集）

| 原则 | 说明 |
|------|------|
| 失败可行动 | 任何红色结果都应能「复制摘要 / 跳转线索 / 建议下一步」 |
| 少切窗口 | 能在 DevKit 内完成的诊断，不逼用户先开 Activity Monitor |
| 能力声明扩展 | 新面板 / 新工具走 registry，避免写死 Tab |
| 只读优先 | 依赖扫描、Git 状态先只读；写操作需二次确认 |
| 可关闭 | 通知、自动体检等默认开启但设置里可关 |
| 分期交付 | P0 立刻减痛；P1 集成；P2 协作与交付 |

---

## 4. 分期总览

| 期 | 主题 | 项 |
|----|------|-----|
| **P0** | 跑得起来、败得明白 | ① 失败可行动日志 ② 环境与端口体检 ③ 日志基础体验 ④ 最近/置顶动作 |
| **P1** | 少切应用 | ⑤ 在编辑器打开 ⑥ 在终端打开 ⑦ HTTP 探活 ⑧ 组合动作流 |
| **P2** | 协作与深化 | ⑨ Git 轻量信息 ⑩ 依赖 outdated ⑪ 多项目日志 ⑫ 工作区配置分享 ⑬ 最小部署 ⑭ 完成通知 |

下文按项展开。编号与上表一致。

---

## 5. 功能详设

### ① 失败可行动的日志（Actionable Failures）

#### 解决什么问题

长日志刷完后，用户不知道「到底错在哪、下一步干嘛」：端口占用、JDK 不对、编译错误被淹没。现在只能人肉翻 `LogsPanel`。

#### 目标 / 非目标

- **目标**：一次运行结束后（或失败时）给出结构化摘要；支持复制；对可识别模式给出短建议。  
- **非目标**：不做完整「问题诊断 AI」；不解析所有语言的堆栈为可点击源码（可后续加）。

#### 交互落点

| 位置 | 调整 |
|------|------|
| [`LogsPanel.vue`](../src/features/workbench/panels/LogsPanel.vue) | 顶部或底部增加「本次运行摘要」条：成功/失败、退出码、耗时、关键错误 1～3 行 |
| 摘要条操作 | 「复制摘要」「仅显示错误」（联动过滤） |
| 设置（可选） | 基本配置：是否自动展开失败摘要 |

#### 后端调整

| 位置 | 调整 |
|------|------|
| [`src-tauri/src/process/mod.rs`](../src-tauri/src/process/mod.rs) | 进程结束时除现有收尸外，汇总 `RunSummary { exitCode, durationMs, errorLines[], hints[] }` |
| 新 IPC | `get_run_summary(projectKey, pid?)` 或随 `process-exited` 事件推送 |
| 新模块建议 | `src-tauri/src/process/summary.rs`：从环形缓冲区抽错误行；规则表匹配 hint |

#### 规则示例（可配置、可扩展）

| 匹配 | 建议文案 |
|------|----------|
| `Address already in use` / `EADDRINUSE` / `端口被占用` | 端口可能被占用 → 打开「工具 → 端口管理」 |
| `Unsupported class file` / `invalid target release` | JDK 版本可能不匹配 → 打开「设置 → 工具链 → Java」 |
| `Could not resolve dependencies` / `npm ERR!` | 依赖未就绪 → 建议先执行 install |
| `command not found: mvn/cargo/...` | 工具链未配置或未在 PATH |

#### 前端调整

| 位置 | 调整 |
|------|------|
| [`useWorkbench.js`](../src/features/workbench/useWorkbench.js) | 监听退出事件，写入当前 Tab 的 `lastRunSummary` |
| [`stores/instances.js`](../src/stores/instances.js) 或新 store | 按 `projectKey` 存最近一次摘要 |
| i18n | `runSummary*`、`hintPortInUse` 等 |

#### 验收

- 故意端口冲突启动后，摘要出现且含「端口」类建议。  
- 一键复制内容含项目名、动作、退出码、关键行。

---

### ② 环境与端口体检（Workspace Health）

#### 解决什么问题

工具链配错、端口已被占时，用户往往点了 Run 才失败。希望在选中工作区或项目时提前暴露风险。

#### 目标 / 非目标

- **目标**：工作区级 + 当前项目级健康检查；红/黄提示可点进设置或端口工具。  
- **非目标**：不自动杀进程（与已排除的「外部进程领养」区分；仅提示「某端口被占用」）。

#### 交互落点

| 位置 | 调整 |
|------|------|
| [`Topbar.vue`](../src/features/workbench/Topbar.vue) 或工作区路径旁 | 「环境」小图标 / Tag：正常 / n 项警告 |
| 点击展开 | Drawer 或 Popover：检查项列表 |
| [`ProjectHeader.vue`](../src/features/workbench/ProjectHeader.vue) | 当前项目若声明常用端口被占，显示警告条 |
| [`tools/ports`](../src/features/tools/ports/) | 从体检「去处理」深链：带上 port 查询参数 |

#### 检查项（首期）

| 项 | 逻辑 |
|----|------|
| Java 工具链 | 若工作区含 maven/gradle：`java.jdks` 非空且 `java -version` 可跑 |
| Node 工具链 | 含 node 项目：nodes 非空；默认 pm 在 PATH |
| Cargo | 含 cargo：`cargo -V` |
| Maven Home | 含 maven：`mavenHome` 或 PATH 有 `mvn` |
| 端口（弱） | 若能从**上次成功运行**或 cmdline 记录到 port，则查是否 LISTEN；无记录则跳过 |

#### 后端调整

| 位置 | 调整 |
|------|------|
| 新模块 | `src-tauri/src/health/`：`check_workspace(root, projects[]) -> HealthReport` |
| 复用 | 现有 toolchain 探测、[`tools/ports/service.rs`](../src-tauri/src/tools/ports/service.rs) 的 lookup |
| IPC | `workspace_health_check` |

#### 前端调整

| 位置 | 调整 |
|------|------|
| 新 | `src/features/workbench/HealthPopover.vue` + `useWorkspaceHealth.js` |
| 触发 | 扫描完成后、切换工作区后、手动刷新 |
| 设置 | `general.healthCheckOnScan`（默认 true） |

#### 验收

- 清空 Maven Home 后扫描含 Maven 的仓，体检出现警告并可跳转设置。  
- 已知端口占用时，对应项目头有提示并可打开端口工具。

---

### ③ 日志基础体验增强

#### 解决什么问题

日志面板功能偏「只追加」：难搜、难滤错误、长任务刷屏干扰。

#### 交互落点

| 位置 | 调整 |
|------|------|
| [`LogsPanel.vue`](../src/features/workbench/panels/LogsPanel.vue) | 工具条：搜索框、级别过滤（All / Error）、「跟随底部」开关、导出 |
| 样式 | 错误行高亮（复用/扩展 [`logFormat.js`](../src/shared/logFormat.js)） |

#### 后端调整

- **首期可纯前端**：对已拉到的日志行过滤即可。  
- 若日志量巨大：后续加 `query_logs(projectKey, { needle, level })` 避免全量进内存。

#### 前端调整

| 位置 | 调整 |
|------|------|
| `logFormat.js` | `classifyLine(line) -> info\|warn\|error` |
| LogsPanel | 本地 filter + `download` 导出 `.log` |
| 设置 | 已有 `logWrap`；新增 `logFollow` 默认 true |

#### 验收

- 输入关键字只显示匹配行；切换 Error 只显示错误类行。  
- 导出文件含当前过滤结果或全量（需在 UI 标明）。

---

### ④ 最近动作与置顶动作

#### 解决什么问题

动作条按钮多、scripts 长时，每次找「上次那个命令」成本高。

#### 交互落点

| 位置 | 调整 |
|------|------|
| [`ActionBar.vue`](../src/features/workbench/ActionBar.vue) | 左侧或 ··· 菜单增加「最近」分组；星标置顶 |
| 动作按钮 | 右键或 ···：「置顶 / 取消置顶」 |

#### 数据

| 存储 | 内容 |
|------|------|
| 用户级 `~/.devkit/settings.json` 或独立 `action-prefs.json` | `recentByProjectKey: { [key]: actionId[] }`、`pinnedByProjectKey` |
| 建议 | 放 `settings.general` 易膨胀；优先 `~/.devkit/action-prefs.json` |

#### 后端调整

| 位置 | 调整 |
|------|------|
| 可选 IPC | `load_action_prefs` / `save_action_prefs`；或前端直读写（Tauri fs 插件） |
| `run_action` 成功发起后 | 前端更新 recent 列表（不必等进程结束） |

#### 前端调整

| 位置 | 调整 |
|------|------|
| `useWorkbench.js` / ActionBar | 合并：pinned → recent → 默认 scripts 顺序，再按 `actionButtonCount` 截断 |
| i18n | `actionPin`、`actionRecent` |

#### 验收

- 连续执行同一 script 后，该动作进入「最近」并靠前。  
- 置顶后重启应用仍在。

---

### ⑤ 在编辑器中打开

#### 解决什么问题

DevKit 负责跑，改代码仍在 Cursor / VS Code / IDEA；缺少「从当前项目一键跳过去」。

#### 交互落点

| 位置 | 调整 |
|------|------|
| [`ProjectHeader.vue`](../src/features/workbench/ProjectHeader.vue) 或 ActionBar 旁 | 按钮「在编辑器中打开」 |
| 设置 → 基本配置或新「外部工具」 | 默认编辑器：`cursor` / `code` / `idea` / 自定义命令模板 |

#### 如何调整

| 层 | 做法 |
|----|------|
| 前端 | 调 opener 或新 command `open_in_editor(path)` |
| 后端 | `std::process::Command` 执行配置的 CLI，参数为项目 `path`；macOS 可回退 `open -a` |
| 插件 | 已有 `@tauri-apps/plugin-opener` 可打开路径；带参数的 CLI 更适合 Rust 侧 |

#### 验收

- 配置 `cursor` 后，点击可打开当前模块目录。  
- CLI 不存在时 toast 明确错误，并链到设置。

---

### ⑥ 在终端中打开

#### 解决什么问题

复杂命令、临时 env、交互式工具仍需终端；希望 cwd 已是项目目录。

#### 交互落点

| 位置 | 调整 |
|------|------|
| 与「编辑器」并列 | 「在终端打开」 |
| 设置 | 终端模板：macOS `Terminal` / `iTerm` / `warp`；Windows `wt` / `cmd`；Linux `x-terminal-emulator` 等 |

#### 后端调整

| 位置 | 调整 |
|------|------|
| `platform/terminal.rs`（新） | 按 OS 拼启动命令并 `current_dir(project.path)` |
| IPC | `open_in_terminal(path)` |

#### 验收

- 各平台至少一种默认终端可用；失败有可读错误。

---

### ⑦ HTTP 探活（Run Health Probe）

#### 解决什么问题

进程在跑 ≠ 服务可用（还在编译、还没听端口、路径 404）。联调时需要快速确认「活了没」。

#### 交互落点

| 位置 | 调整 |
|------|------|
| [`MonitorPanel.vue`](../src/features/workbench/panels/MonitorPanel.vue) | 「探活」按钮 + 最近结果（状态码 / 耗时 / 错误） |
| 项目设置（可选） | 每项目默认 URL 模板，如 `http://127.0.0.1:{port}/actuator/health` |

#### 如何调整

| 层 | 做法 |
|----|------|
| 端口来源 | 优先托管实例的 `ProcessView.port` / metrics.ports；无则让用户填 |
| 后端 | `probe_http(url, timeoutMs) -> { ok, status, ms, error }`（reqwest） |
| 前端 | 展示结果；失败不自动杀进程 |

#### 验收

- Spring Boot 默认端口起来后，探活返回 200 或可配置路径结果。  
- 服务未就绪时显示连接失败而非崩溃。

---

### ⑧ 组合动作流（Action Pipeline）

#### 解决什么问题

真实工作流常是「install → build → run」或「clean → package」。现在只能手点多次，中间失败还要自己停。

#### 交互落点

| 位置 | 调整 |
|------|------|
| ActionBar ··· 或项目头 | 「运行流水线」 |
| 设置 / 项目级 | 预置模板 + 自定义步骤列表（动作 id 数组） |
| 日志 | 同一 Logs 面板连续输出，步骤分隔线 |

#### 数据模型（建议）

```text
Pipeline {
  id, name,
  projectKind?: string,       // 可选：仅某 Kind 可用
  steps: [{ action: string, stopOnError: bool }]
}
```

- 用户级默认模板 + 可选工作区覆盖（见 ⑫）。

#### 后端调整

| 位置 | 调整 |
|------|------|
| `process` | `run_pipeline(projectKey, steps)`：串行调用现有 resolve_command / spawn；上一步非 0 且 `stopOnError` 则中止 |
| 事件 | `pipeline-step`（started/finished）供 UI 进度 |

#### 前端调整

| 位置 | 调整 |
|------|------|
| 新 | `PipelineDialog.vue`：选模板、展示步骤状态 |
| ActionBar | 入口按钮 |

#### 验收

- 三步流水线第二步失败时不启动第三步，摘要标明停在哪一步。

---

### ⑨ Git 轻量信息

#### 解决什么问题

跑构建前想知道「在哪个分支、是否有未提交改动」，不必为这切到 IDE。

#### 目标 / 非目标

- **目标**：只读展示 branch、dirty、ahead/behind（可选）。  
- **非目标**：commit / push / 解决冲突。

#### 交互落点

| 位置 | 调整 |
|------|------|
| Topbar 工作区旁或 Sidebar 底 | `main*` 一类短标签；悬停看详情 |
| 点击 | 可选：打开系统 Git GUI / 复制分支名 |

#### 后端调整

| 位置 | 调整 |
|------|------|
| `src-tauri/src/tools/git/` 或 `platform/git.rs` | `git -C root rev-parse --abbrev-ref HEAD`；`status --porcelain` |
| IPC | `workspace_git_status(root)` |
| 依赖 | 要求本机有 `git`；无则隐藏入口 |

#### 验收

- 非 git 目录不报错，仅隐藏。  
- dirty 时标签有视觉区分。

---

### ⑩ 依赖 outdated / 冲突只读提示

#### 解决什么问题

依赖树能看结构，但不知道「过时了吗、有没有冲突」。

#### 交互落点

| 位置 | 调整 |
|------|------|
| [`DepsPanel.vue`](../src/features/workbench/panels/DepsPanel.vue) | Tab 或按钮「检查更新」；只读表格：name / current / latest |
| 风险 | 明确标注「只读，不会改锁文件」 |

#### 后端调整（按 Kind）

| Kind | 命令思路 |
|------|----------|
| node | `npm outdated --json` / `pnpm outdated`（用设置中的 pm） |
| cargo | `cargo outdated`（若未安装则提示安装） |
| maven | 可选 `versions:display-dependency-updates`（较重，可 P2 后置） |
| gradle | 后置 |

#### 验收

- Node 项目能列出至少一批 outdated；失败时给出命令级错误而非白屏。

---

### ⑪ 多项目日志分栏

#### 解决什么问题

前后端联调时来回切 Tab 看日志效率低。

#### 交互落点

| 位置 | 调整 |
|------|------|
| 工作台主区 | 「分栏日志」模式：最多 2～3 路，绑定不同 `projectKey` |
| 入口 | Tab 栏工具区或 View 菜单式按钮 |

#### 如何调整

| 层 | 做法 |
|----|------|
| 前端 | 复用 LogsPanel 多实例；`useWorkbench` 增加 `splitLogKeys[]` |
| 后端 | 无需改；现有按 key 推送日志事件即可 |
| 限制 | 同时分栏数 ≤ 3，避免性能与布局崩溃 |

#### 验收

- 两个运行中项目可左右同时刷日志；关闭分栏回单面板。

---

### ⑫ 工作区配置可分享（`.devkit/workspace.json`）

#### 解决什么问题

团队每人过滤规则、推荐动作、探活 URL 不一致；用户级 `~/.devkit/settings.json` 无法进仓库。

#### 目标 / 非目标

- **目标**：仓库内可选配置，覆盖「工作区相关」偏好。  
- **非目标**：不把本机 JDK 绝对路径写进仓库。

#### 建议文件

```text
<workspace>/.devkit/workspace.json
```

```json
{
  "version": 1,
  "projectFilter": { },
  "runPlans": [],
  "pipelines": [],
  "probes": { "maven": "http://127.0.0.1:{port}/actuator/health" },
  "recommended": { "nodePackageManager": "pnpm" }
}
```

多根会话存在本机 `localStorage`（`devkit.workspace.session`）；上限由 `settings.general.maxWorkspaceRoots`（默认 10）与 `maxParallelSpawns`（默认 10）控制。RunPlan：个人计划在 `~/.devkit/settings.json` 的 `runPlans`；仓库计划在各根 `.devkit/workspace.json` 的 `runPlans`（读时兼容旧 `pipelines`）；同 `id` 时工作区覆盖个人。行内存 `relPath`+`kind`，不写死绝对路径。

#### 合并规则

| 项 | 优先级 |
|----|--------|
| 本机工具链路径 | 仅用户 settings |
| projectFilter / runPlans / probes | workspace 文件 < 用户覆盖（或相反，需在设置中选「优先工作区」）；同 id 的 runPlan 工作区优先 |
| 推荐 pm | 提示用户，不强制改本机默认 |

#### 调整位置

| 位置 | 调整 |
|------|------|
| 扫描完成后 | 读 `.devkit/workspace.json` |
| 设置页 | 「从工作区加载的项」只读说明 + 「写回工作区」导出（可选） |
| DESIGN | 与「设置文件即配置真源」并列：**工作区文件 = 可分享覆盖层** |

#### 验收

- 两人克隆同一仓库后，过滤与流水线模板一致；JDK 路径仍各用各的。

---

### ⑬ 最小可用部署（Deploy MVP）

#### 解决什么问题

[`deploy/`](../src-tauri/src/deploy/) 骨架无法交付价值；用户需要的是「产物 + 传到某台机器」而非完整 K8s。

#### MVP 范围

1. 选定本地产物路径（或接在 `package`/`build` 成功后）。  
2. 配置 SSH 目标（host、user、path；密钥走安全存储）。  
3. 一键：`scp`/`rsync` 上传 + 可选远程命令。  

#### 交互落点

| 位置 | 调整 |
|------|------|
| 工具页或项目面板「部署」Tab | 表单 + 执行日志 |
| 设置 | SSH 目标列表（secret 分离） |

#### 调整位置

| 位置 | 调整 |
|------|------|
| `deploy` 模块 | 实现第一个 `SshUploadAdapter` |
| 前端 | `features/deploy/` 最小面板  
| 与 DESIGN §1.2 | 对齐 Adapter 注册，但不一次做完 Docker/K8s |

#### 验收

- 能把指定目录上传到测试机并执行一条远程命令；失败日志可见。

---

### ⑭ 长时间任务完成通知

#### 解决什么问题

build 要几分钟，窗口在托盘或被挡住时不知道结束。

#### 交互落点

| 位置 | 调整 |
|------|------|
| 进程结束 | 系统通知：成功/失败 + 项目名 + 动作 |
| 设置 → 应用 | 「任务完成时通知」开关（默认开） |

#### 如何调整

| 层 | 做法 |
|----|------|
| Tauri | `tauri-plugin-notification` 或 OS API |
| 触发点 | 与 ① 同一退出钩子 |
| 策略 | 窗口 focused 且在前台时可省略；托盘或失焦时发送 |

#### 验收

- 切到其他应用期间完成 build，能收到系统通知；设置关闭后不再发送。

---

## 6. 跨功能改动地图（按目录）

| 目录 / 文件 | 主要相关项 |
|-------------|------------|
| `src/features/workbench/panels/LogsPanel.vue` | ① ③ ⑪ |
| `src/features/workbench/ActionBar.vue` | ④ ⑧ |
| `src/features/workbench/ProjectHeader.vue` / `Topbar.vue` | ② ⑤ ⑥ ⑨ |
| `src/features/workbench/panels/MonitorPanel.vue` | ⑦ |
| `src/features/workbench/panels/DepsPanel.vue` | ⑩ |
| `src/features/workbench/useWorkbench.js` | ① ② ④ ⑧ ⑪ ⑭ |
| `src/features/tools/` | ② 深链端口；⑬ 可挂工具或独立 |
| `src/features/settings/` | ② ⑤ ⑥ ⑫ ⑭ 开关与外部工具 |
| `src/stores/` | 摘要、prefs、health 缓存 |
| `src/i18n/locales/*` | 全部文案 |
| `src-tauri/src/process/` | ① ⑧ ⑭ |
| `src-tauri/src/health/`（新） | ② |
| `src-tauri/src/platform/` | ⑤ ⑥ ⑨ 终端/编辑器/git |
| `src-tauri/src/tools/ports/` | ② 联动 |
| `src-tauri/src/deploy/` | ⑬ |
| `src-tauri/src/commands/mod.rs` | 注册新 IPC |
| `~/.devkit/*` | prefs、可选与 settings 拆分 |
| `<repo>/.devkit/workspace.json` | ⑫ |

---

## 7. 推荐落地顺序（执行清单）

```text
迭代 A（1～2 周体量）
  ① 失败摘要 + hint
  ③ 日志搜索/错误过滤/跟随
  ⑭ 完成通知（与退出钩子一起）

迭代 B
  ② 环境体检（先工具链，端口后置）
  ④ 最近/置顶动作
  ⑤⑥ 打开编辑器/终端

迭代 C
  ⑦ 探活
  ⑧ 流水线
  ⑨ Git 只读

迭代 D
  ⑩ outdated
  ⑪ 分栏日志
  ⑫ workspace.json
  ⑬ Deploy MVP
```

---

## 8. 风险与约束

| 风险 | 缓解 |
|------|------|
| 日志摘要误报 | hint 仅建议不自动执行；可关闭 |
| 体检太慢 | 异步、缓存、仅扫描后跑一次 |
| 流水线杀错进程 | 仍只管理 DevKit 拉起的 Child；步骤失败即停 |
| 工作区配置泄露本机路径 | schema 禁止提交绝对工具链路径；文档写明 |
| 通知打扰 | 设置开关 + 前台抑制 |
| 范围膨胀成 IDE | 每期对照 §2 定位；不做调试器/完整 Git |

---

## 9. 与「外部进程检测」的边界

已决议：**暂不实现**「检测非 DevKit 启动的项目进程并提供杀死」。

相关诉求（端口冲突）在本文中的替代路径：

- ② 体检提示端口占用 → 跳转现有**端口管理**手动处理  
- ① 启动失败摘要识别 EADDRINUSE → 引导端口工具  

若将来重开外部进程能力，应单独开设计附录（误杀、权限、cwd 误匹配），不塞进 P0。

---

## 10. 成功标准（产品层）

做完 P0～P1 后，应能观察到：

1. 启动失败时，用户平均「定位原因」步骤减少（摘要 + 建议）。  
2. 从 DevKit 跳到编辑器/终端无需手抄路径。  
3. 工具链未配置、端口冲突在 Run 之前被看见。  
4. 重复操作的项目，常用动作点击次数下降（最近/置顶/流水线）。

---

## 11. 修订记录

| 日期 | 说明 |
|------|------|
| 2026-07-19 | 初稿：基于效率讨论整理 14 项；排除外部进程领养 |
| 2026-07-19 | 迭代 B–D 落地：②体检 ④动作偏好 ⑤⑥外部打开 ⑦探活 ⑧流水线 ⑨Git ⑩outdated ⑪分栏日志 ⑫workspace.json ⑬SSH 部署 MVP（前端 + Rust IPC） |

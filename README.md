# DevKit

**Monorepo 项目工作台** —— 一个窗口，管好你目录里所有的 Node、Maven、Cargo、Gradle 项目。

选好工作区根目录，DevKit 自动扫出子项目；点一下就能安装依赖、构建、运行、看日志。不用再开一堆终端、记一堆命令、在 IDE 之间来回跳。

支持 **macOS · Windows · Linux**。界面有中文、繁中、English、日本語。

---

## 为什么需要 DevKit？

大仓库、多模块、多技术栈时，日常往往是这样的：

- 终端标签越开越多，分不清哪个是 `bootRun`、哪个是 `pnpm dev`
- 同目录既有前端又有后端，日志和进程搅在一起
- 换机器就要重新配 JDK、Maven、Node 路径
- 只想「清一下占用 8080 的进程」，却要翻系统监视器

DevKit 把这些收进一个轻量桌面应用：扫得清、跑得稳、日志分得开、设置跟人走。

---

## 核心能力

### 一键扫描，整仓尽收眼底

选择任意工作区根目录，自动识别其中的：

| 项目类型 | 自动识别 | 你能直接做的事 |
|----------|----------|----------------|
| **Node** | `package.json` | 安装依赖；一键跑 scripts（npm / pnpm / yarn） |
| **Maven** | `pom.xml` | clean / install / package…；Spring Boot 可直接 `spring-boot:run` |
| **Cargo** | `Cargo.toml` | build / run / test / check / clean |
| **Gradle** | Gradle 工程文件 | clean / build / test；Spring Boot 可 `bootRun`；优先 `gradlew` |

侧栏按类型筛选，多标签同时打开多个项目——像浏览器开页签一样切换。

### 动作条 + 实时日志

常用命令摆在显眼位置（显示数量可调）。点运行即开进程，日志流式刷在窗口里，支持自动换行。

同目录下 Node 与 Maven 等不同类型互不串台：每个项目独立身份，日志、进程各管各的。

### 依赖与运行监控

- **依赖面板**：看清项目依赖结构，排查版本问题时少猜一步  
- **监控面板**：跟踪正在跑的实例，心里有数、随时可停  

### 端口工具：谁占用了这个端口？

内置端口管理：按端口号查找占用进程，需要时一键结束。联调、热重启、端口冲突时特别省事。

### 工具链与首次引导

第一次打开可扫描本机 **JDK / Node / Maven**，写入个人设置。换电脑也能快速把环境对齐。  
各类型还可配置侧栏过滤（例如只要带 Spring Boot 的模块），让列表更干净。

### 用着顺手的桌面体验

- **六套主题色**：青绿、海洋、森林、青石、琥珀、玫红，整窗对齐换肤  
- **四种语言**：简体中文 / 繁體中文 / English / 日本語  
- **关窗进托盘**：关掉主窗口也不丢任务；托盘一键唤回  
- **开机自启**（可选）：机器起来，工作台就在  
- **应用内更新**：有新版本时在设置里检查并安装，不用自己翻 Release 页  

最近打开的工作区会记住，下次接着干。

---

## 适合谁用？

- 同时维护前端 + 多个后端模块的全栈 / 后端同学  
- 手里有 Cargo 服务、又有 Node 工具链的混合仓库维护者  
- 厌倦「开五六个终端标签记命令」的开发者  
- 想在团队里统一「点一下就能跑」的本地工作流  

---

## 开始使用

### 直接安装

到 [GitHub Releases](https://github.com/9527-tb/devkit/releases) 下载对应系统的安装包：

- **macOS**：`.dmg` / `.app`（Apple Silicon 与 Intel 均有构建）  
- **Windows**：安装程序（NSIS / MSI）  
- **Linux**：deb / AppImage / rpm  

> macOS 若提示「无法验证开发者」，在「系统设置 → 隐私与安全性」中允许，或对应用执行：  
> `xattr -cr /Applications/DevKit.app`

### 从源码运行

```bash
pnpm install
pnpm tauri dev
```

个人设置保存在：

- macOS / Linux：`~/.devkit/settings.json`  
- Windows：`%APPDATA%\devkit\settings.json`

---

## 一句话总结

**DevKit = 面向真实 Monorepo 的本地工作台**：扫项目、跑动作、看日志、查依赖、管端口、配环境——少切窗口，多写代码。

欢迎试用、提 Issue，也欢迎一起把更多技术栈接进来。

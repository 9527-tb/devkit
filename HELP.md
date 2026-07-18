# DevKit 操作手册

面向维护者的实操说明。产品介绍见 [README.md](README.md)。

---

## 发布新版本（以 v0.0.3 为例）

Release 由推送 **`v*`** 标签触发，工作流：[`.github/workflows/release.yml`](.github/workflows/release.yml)。

### 前置条件

1. 要进本版的代码已提交并推送到发版分支（通常为 `master`）
2. 仓库 Secrets 已配置：
   - `TAURI_SIGNING_PRIVATE_KEY`（必填）：本地 `~/.tauri/devkit.key` 全文
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（可选）：私钥有密码时再设
3. 勿使用 draft Release：客户端依赖  
   `https://github.com/9527-tb/devkit/releases/latest/download/latest.json`

### 步骤

#### 1. 确认工作区干净（或先提交全部改动）

```bash
git status
git push origin master
```

#### 2. 同步版本号

将 `package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 写成同一版本（不要带 `v` 前缀）：

```bash
pnpm version:sync 0.0.3
```

提交并推送：

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
# Cargo.lock 若有变更一并加入
git add src-tauri/Cargo.lock
git commit -m "chore: bump version to 0.0.3"
git push origin master
```

#### 3. 打标签并推送（触发 CI）

```bash
git tag v0.0.3
git push origin v0.0.3
```

标签名必须带 **`v`**，且与版本号对应（`0.0.3` → `v0.0.3`）。  
**先推代码，再推 tag**，避免 CI 打到旧 commit。

#### 4. 等待并核对

1. GitHub Actions → **Release** 全部成功（含 `fix-updater-json`）
2. [Releases](https://github.com/9527-tb/devkit/releases) 出现 **DevKit v0.0.3**（正式发布，非 draft）
3. 确认 `latest.json` 使用浏览器下载地址（非 `api.github.com`）：

```bash
curl -sL https://github.com/9527-tb/devkit/releases/latest/download/latest.json
```

`platforms.*.url` 应为：

```text
https://github.com/9527-tb/devkit/releases/download/v0.0.3/...
```

而不是：

```text
https://api.github.com/repos/.../releases/assets/<id>
```

4. 用已安装旧版打开设置 → 检查更新 / 立即更新，验证应用内升级

### CI 会做什么

| 阶段 | 说明 |
|------|------|
| `publish-tauri` | 多平台构建，上传安装包与 updater 产物，创建正式 Release |
| `fix-updater-json` | 将 `latest.json` 中的 API 资源 URL 改写为 `browser_download_url`，并以文件名 **`latest.json`** 覆盖上传（旧 bug 曾误传成 `devkit-latest-<tag>.json`，导致修复“成功”但客户端仍 403） |

### 常见问题

| 现象 | 处理 |
|------|------|
| Actions 失败：缺少签名密钥 | 检查 Secret `TAURI_SIGNING_PRIVATE_KEY` |
| 有 Release 但客户端找不到更新 | 确认不是 draft；确认 tag 已推送且为 Latest |
| 立即更新 403 Forbidden | `latest.json` 里若是 `api.github.com/.../assets/<id>` 会 403。先确认 Actions 里 `fix-updater-json` 成功，再手动：`node scripts/fix-latest-json.mjs vX.Y.Z`（必须覆盖上传名为 `latest.json`，不要传成别的文件名） |
| 版本号不一致 | 重新执行 `pnpm version:sync x.y.z` 并提交后再打新 tag |

---

## 日常构建（不发版）

PR / 推送到 `main`、`master`、`develop` 会跑 **Build** 工作流（[`.github/workflows/build.yml`](.github/workflows/build.yml)）：产出各平台安装包，**不**生成 updater 签名产物，也不需要签名 Secret。

本地当前平台打包：

```bash
pnpm tauri:build
```

---

## 更换应用图标

### 系统图标（Dock / 安装包 / 托盘）

准备 ≥1024×1024 的 PNG，然后：

```bash
pnpm tauri icon path/to/logo.png
```

会覆盖 `src-tauri/icons/`。需重新打包或发版后才会在安装包中生效。

### 界面左上角品牌图

使用 [`public/logo.png`](public/logo.png)。替换该文件后重新运行 / 打包即可。

---

## 配置文件位置

| 平台 | 路径 |
|------|------|
| macOS / Linux | `~/.devkit/settings.json` |
| Windows | `%APPDATA%\devkit\settings.json` |

---

## macOS 安装提示「已损坏 / 无法验证」

CI 未做 Apple Developer ID 签名。本机信任时可：

```bash
xattr -cr /Applications/DevKit.app
```

或在「系统设置 → 隐私与安全性」中允许打开。  
（Tauri updater 签名与 Apple 代码签名不是同一套。）

---

## 相关脚本

| 命令 / 脚本 | 用途 |
|-------------|------|
| `pnpm version:sync [x.y.z]` | 查询或写入三处版本号 |
| `pnpm tauri icon <png>` | 生成全套系统图标 |
| `node scripts/fix-latest-json.mjs vX.Y.Z` | 手动修复某次 Release 的 `latest.json` |
| `scripts/tauri-build.mjs` | 本地多平台打包入口 |

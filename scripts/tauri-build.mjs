#!/usr/bin/env node
/**
 * 跨平台打包入口。
 * Tauri 桌面安装包需在目标 OS 上构建；本机只能打当前系统包。
 * Windows / Linux 通过 GitHub Actions（.github/workflows/build.yml）产出。
 *
 * 用法:
 *   node scripts/tauri-build.mjs mac|mac-intel|windows|linux|current|all|ci
 */

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");
const platform = (process.argv[2] || "current").toLowerCase();
const host = process.platform; // darwin | win32 | linux

const TARGETS = {
  mac: {
    host: "darwin",
    label: "macOS (Apple Silicon)",
    args: ["build", "--target", "aarch64-apple-darwin"],
  },
  "mac-intel": {
    host: "darwin",
    label: "macOS (Intel)",
    args: ["build", "--target", "x86_64-apple-darwin"],
  },
  windows: {
    host: "win32",
    label: "Windows (x64)",
    args: ["build"],
    rustTarget: "x86_64-pc-windows-msvc",
  },
  linux: {
    host: "linux",
    label: "Linux (x64)",
    args: ["build"],
    rustTarget: "x86_64-unknown-linux-gnu",
  },
  current: {
    host,
    label: `current host (${host})`,
    args: ["build"],
  },
};

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    cwd: root,
    stdio: "inherit",
    shell: process.platform === "win32",
    ...opts,
  });
  return r.status ?? 1;
}

function runTauri(args) {
  const local = resolve(root, "node_modules/.bin/tauri");
  const bin = existsSync(local) ? local : "pnpm";
  const finalArgs = existsSync(local) ? args : ["tauri", ...args];
  const cmd = existsSync(local) ? local : "pnpm";
  console.log(`\n> ${cmd} ${finalArgs.join(" ")}\n`);
  return run(cmd, finalArgs);
}

function printCrossHint(name) {
  const t = TARGETS[name];
  console.error(`
[devkit] 无法在 ${host} 上本地打包 ${t.label}。
Tauri 安装包必须在目标操作系统上构建。

可选方式：
  1) GitHub Actions（推荐）
     - 推送仓库后执行:  pnpm tauri:build:ci
     - 或在 GitHub → Actions →「Build Windows / macOS / Linux」→ Run workflow
     - 产物: Actions Artifacts（devkit-windows-x64 / devkit-linux-x64 / …）

  2) 在对应系统机器上执行:
     Windows:  pnpm tauri:build:windows
     Linux:    pnpm tauri:build:linux
`);
}

function triggerCi() {
  const hasGit = existsSync(resolve(root, ".git"));
  if (!hasGit) {
    console.error(`
[devkit] 当前目录还不是 git 仓库，无法触发 GitHub Actions。

请先：
  git init
  git add .
  git commit -m "chore: init"
  # 创建 GitHub 仓库后
  git remote add origin <your-repo-url>
  git push -u origin main

然后执行:  pnpm tauri:build:ci
或在 GitHub Actions 页面手动运行 workflow「Build Windows / macOS / Linux」。
`);
    return 1;
  }

  const gh = spawnSync("gh", ["--version"], { encoding: "utf8" });
  if (gh.status !== 0) {
    console.error(`
[devkit] 未检测到 GitHub CLI (gh)。

安装后登录，再执行 pnpm tauri:build:ci：
  brew install gh && gh auth login

或打开 GitHub 网页 → Actions →「Build Windows / macOS / Linux」→ Run workflow
`);
    return 1;
  }

  console.log("[devkit] 触发 GitHub Actions: Build Windows / macOS / Linux …\n");
  return run("gh", ["workflow", "run", "build.yml"]);
}

function buildOne(name) {
  const t = TARGETS[name];
  if (!t) {
    console.error(`未知平台: ${name}`);
    return 1;
  }
  if (t.host !== host) {
    printCrossHint(name);
    return 1;
  }
  console.log(`[devkit] 打包 ${t.label} …`);
  return runTauri(t.args);
}

function buildAll() {
  console.log(`[devkit] 本机 (${host}) 打包 + 尝试触发 CI 打 Windows/Linux/macOS\n`);
  const local = buildOne("current");
  if (local !== 0) return local;
  console.log("\n[devkit] 本机打包完成。接着触发 CI 产出其余平台…\n");
  return triggerCi();
}

const code = (() => {
  if (platform === "ci") return triggerCi();
  if (platform === "all") return buildAll();
  if (platform in TARGETS) return buildOne(platform);
  console.error(`用法: node scripts/tauri-build.mjs <mac|mac-intel|windows|linux|current|all|ci>`);
  return 1;
})();

process.exit(code);

#!/usr/bin/env node
/**
 * 将 latest.json 中的 api.github.com/assets/<id> 替换为 browser_download_url。
 * tauri-action@v1 默认写 API URL；公开仓库客户端无 token 会 403（且受 API 限流）。
 *
 * 用法:
 *   node scripts/fix-latest-json.mjs v0.0.2
 *   TAG=v0.0.2 node scripts/fix-latest-json.mjs
 *
 * 需要 gh 已登录（或 CI 中 GITHUB_TOKEN），且对本仓库有 contents:write。
 */
import { writeFileSync, readFileSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const tag = (process.argv[2] || process.env.TAG || "").replace(/^refs\/tags\//, "");
if (!tag) {
  console.error("Usage: node scripts/fix-latest-json.mjs <tag>");
  process.exit(1);
}

const repo =
  process.env.GITHUB_REPOSITORY ||
  execFileSync("gh", ["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"], {
    encoding: "utf8",
  }).trim();

const release = JSON.parse(
  execFileSync("gh", ["api", `repos/${repo}/releases/tags/${tag}`], { encoding: "utf8" }),
);

/** @type {Map<string, string>} */
const idToBrowser = new Map();
for (const asset of release.assets || []) {
  if (asset.id != null && asset.browser_download_url) {
    idToBrowser.set(String(asset.id), asset.browser_download_url);
  }
}

if (!(release.assets || []).some((a) => a.name === "latest.json")) {
  console.error(`No latest.json on release ${tag}`);
  process.exit(1);
}

const downloaded = join(tmpdir(), `devkit-latest-download-${tag}.json`);
execFileSync(
  "gh",
  [
    "release",
    "download",
    tag,
    "--repo",
    repo,
    "--pattern",
    "latest.json",
    "--output",
    downloaded,
    "--clobber",
  ],
  { stdio: "inherit" },
);

const latest = JSON.parse(readFileSync(downloaded, "utf8"));

let changed = 0;
for (const info of Object.values(latest.platforms || {})) {
  if (!info || typeof info.url !== "string") continue;
  const m = info.url.match(/\/releases\/assets\/(\d+)\s*$/);
  if (!m) continue;
  const browser = idToBrowser.get(m[1]);
  if (!browser) {
    console.warn(`No browser_download_url for asset ${m[1]}`);
    continue;
  }
  if (info.url !== browser) {
    info.url = browser;
    changed += 1;
  }
}

if (!changed) {
  console.log(`[fix-latest-json] ${tag}: already using browser download URLs`);
  try {
    unlinkSync(downloaded);
  } catch {
    /* ignore */
  }
  process.exit(0);
}

const tmp = join(tmpdir(), `devkit-latest-${tag}.json`);
writeFileSync(tmp, `${JSON.stringify(latest, null, 2)}\n`);
execFileSync("gh", ["release", "upload", tag, tmp, "--repo", repo, "--clobber"], {
  stdio: "inherit",
});
try {
  unlinkSync(tmp);
  unlinkSync(downloaded);
} catch {
  /* ignore */
}

console.log(`[fix-latest-json] ${tag}: rewrote ${changed} platform URL(s)`);

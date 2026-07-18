#!/usr/bin/env node
/**
 * 将 latest.json 中的 api.github.com/assets/<id> 替换为 browser_download_url，
 * 并以文件名 latest.json 覆盖上传（公开仓库客户端无 token 会 403）。
 *
 * 用法:
 *   node scripts/fix-latest-json.mjs v0.0.3
 *
 * 需要 gh 已登录（或 CI 中 GH_TOKEN / GITHUB_TOKEN），且对本仓库有 contents:write。
 */
import { writeFileSync, readFileSync, unlinkSync, mkdirSync, rmSync } from "node:fs";
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

function ghJson(args) {
  return JSON.parse(execFileSync("gh", args, { encoding: "utf8" }));
}

function countApiUrls(latest) {
  return Object.values(latest.platforms || {}).filter(
    (info) => info && typeof info.url === "string" && info.url.includes("api.github.com"),
  ).length;
}

function rewrite(latest, idToBrowser) {
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
  return changed;
}

const release = ghJson(["api", `repos/${repo}/releases/tags/${tag}`]);

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

const workDir = join(tmpdir(), `devkit-fix-latest-${tag}-${Date.now()}`);
mkdirSync(workDir, { recursive: true });
const downloaded = join(workDir, "downloaded.json");
const uploadPath = join(workDir, "latest.json"); // 必须叫 latest.json，否则会上传成错误文件名

try {
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
  const before = countApiUrls(latest);
  const changed = rewrite(latest, idToBrowser);
  const after = countApiUrls(latest);

  if (after > 0) {
    console.error(
      `[fix-latest-json] ${tag}: still has ${after} api.github.com URL(s) after rewrite (changed=${changed})`,
    );
    process.exit(1);
  }

  if (!changed && before === 0) {
    console.log(`[fix-latest-json] ${tag}: already using browser download URLs`);
    process.exit(0);
  }

  // 标记变化，便于打穿 GitHub CDN 对 latest.json 的缓存
  latest.notes = `${(latest.notes || "").replace(/\n*<!-- bust \d+ -->\n*$/, "").trimEnd()}\n\n<!-- bust ${Date.now()} -->\n`;

  writeFileSync(uploadPath, `${JSON.stringify(latest, null, 2)}\n`);

  // 先删后传：--clobber 有时仍命中旧 CDN；删除可换新 asset id
  try {
    execFileSync(
      "gh",
      ["release", "delete-asset", tag, "latest.json", "--repo", repo, "--yes"],
      { stdio: "inherit" },
    );
  } catch {
    /* ignore if missing */
  }
  execFileSync("gh", ["release", "upload", tag, uploadPath, "--repo", repo], {
    stdio: "inherit",
  });

  // 校验：经 gh 拉取（权威）必须已无 API URL
  const verifyPath = join(workDir, "verify.json");
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
      verifyPath,
      "--clobber",
    ],
    { stdio: "inherit" },
  );
  const verified = JSON.parse(readFileSync(verifyPath, "utf8"));
  const remaining = countApiUrls(verified);
  if (remaining > 0) {
    console.error(
      `[fix-latest-json] ${tag}: upload completed but latest.json still has ${remaining} API URL(s)`,
    );
    process.exit(1);
  }

  console.log(`[fix-latest-json] ${tag}: rewrote ${changed} platform URL(s) → latest.json`);
} finally {
  try {
    rmSync(workDir, { recursive: true, force: true });
  } catch {
    /* ignore */
  }
}

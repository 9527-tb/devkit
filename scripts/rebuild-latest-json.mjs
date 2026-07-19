#!/usr/bin/env node
/**
 * 根据 Release 资产完整重建 latest.json（含 darwin），URL 一律用 browser_download_url。
 * 同时写入仓库 updater/latest.json，供 jsDelivr 镜像端点使用。
 *
 * 用法: node scripts/rebuild-latest-json.mjs v0.0.4
 *
 * 环境变量:
 *   DEVKIT_UPDATE_DOWNLOAD_MIRROR=0  — 不改写下载 URL（默认经 ghfast 加速，便于国内安装）
 */
import { writeFileSync, readFileSync, mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

const tag = (process.argv[2] || "").replace(/^refs\/tags\//, "");
if (!tag) {
  console.error("Usage: node scripts/rebuild-latest-json.mjs <tag>");
  process.exit(1);
}

const useDownloadMirror = process.env.DEVKIT_UPDATE_DOWNLOAD_MIRROR !== "0";
const DOWNLOAD_MIRROR_PREFIX = "https://ghfast.top/";

const repo =
  process.env.GITHUB_REPOSITORY ||
  execFileSync("gh", ["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"], {
    encoding: "utf8",
  }).trim();

function ghJson(args) {
  return JSON.parse(execFileSync("gh", args, { encoding: "utf8" }));
}

function mirrorUrl(url) {
  if (!useDownloadMirror) return url;
  if (url.startsWith(DOWNLOAD_MIRROR_PREFIX)) return url;
  return `${DOWNLOAD_MIRROR_PREFIX}${url}`;
}

/** @param {string} name @param {Map<string, any>} byName */
function platformEntries(name, byName) {
  const asset = byName.get(name);
  const sigAsset = byName.get(`${name}.sig`);
  if (!asset?.browser_download_url || !sigAsset) return null;

  const sigPath = join(tmpdir(), `devkit-sig-${sigAsset.id}.sig`);
  execFileSync(
    "gh",
    [
      "release",
      "download",
      tag,
      "--repo",
      repo,
      "--pattern",
      sigAsset.name,
      "--output",
      sigPath,
      "--clobber",
    ],
    { stdio: "pipe" },
  );
  const signature = readFileSync(sigPath, "utf8").trim();
  return { url: mirrorUrl(asset.browser_download_url), signature };
}

const release = ghJson(["api", `repos/${repo}/releases/tags/${tag}`]);
const byName = new Map((release.assets || []).map((a) => [a.name, a]));

const version = tag.replace(/^v/, "");
/** @type {Record<string, {url:string, signature:string}>} */
const platforms = {};

const mapping = [
  // macOS updater 使用 .app.tar.gz
  [`DevKit_${version}_aarch64.app.tar.gz`, ["darwin-aarch64", "darwin-aarch64-app"]],
  [`DevKit_${version}_x64.app.tar.gz`, ["darwin-x86_64", "darwin-x86_64-app"]],
  [`DevKit_${version}_x64-setup.exe`, ["windows-x86_64", "windows-x86_64-nsis"]],
  [`DevKit_${version}_x64_en-US.msi`, ["windows-x86_64-msi"]],
  [`DevKit_${version}_amd64.AppImage`, ["linux-x86_64", "linux-x86_64-appimage"]],
  [`DevKit_${version}_amd64.deb`, ["linux-x86_64-deb"]],
  [`DevKit-${version}-1.x86_64.rpm`, ["linux-x86_64-rpm"]],
];

for (const [file, keys] of mapping) {
  const entry = platformEntries(file, byName);
  if (!entry) {
    console.warn(`[rebuild-latest-json] skip missing ${file}`);
    continue;
  }
  for (const key of keys) {
    platforms[key] = entry;
  }
  console.log(`[rebuild-latest-json] + ${keys.join(", ")} ← ${file}`);
}

if (!platforms["darwin-aarch64"] && !platforms["darwin-x86_64"]) {
  console.error("[rebuild-latest-json] no darwin platforms — macOS updater will fail");
  process.exit(1);
}
if (!Object.keys(platforms).length) {
  console.error("[rebuild-latest-json] no platforms found");
  process.exit(1);
}

const latest = {
  version,
  notes:
    (release.body || `## DevKit ${tag}`).trim() +
    `\n\n<!-- bust ${Date.now()} -->\n`,
  pub_date: new Date().toISOString(),
  platforms,
};

const jsonText = `${JSON.stringify(latest, null, 2)}\n`;

const updaterDir = join(root, "updater");
mkdirSync(updaterDir, { recursive: true });
const repoPath = join(updaterDir, "latest.json");
writeFileSync(repoPath, jsonText);
console.log(`[rebuild-latest-json] wrote ${repoPath}`);

const workDir = join(tmpdir(), `devkit-rebuild-latest-${tag}-${Date.now()}`);
mkdirSync(workDir, { recursive: true });
const uploadPath = join(workDir, "latest.json");
writeFileSync(uploadPath, jsonText);

try {
  try {
    execFileSync(
      "gh",
      ["release", "delete-asset", tag, "latest.json", "--repo", repo, "--yes"],
      { stdio: "inherit" },
    );
  } catch {
    /* ignore */
  }
  execFileSync("gh", ["release", "upload", tag, uploadPath, "--repo", repo], {
    stdio: "inherit",
  });
  console.log(
    `[rebuild-latest-json] ${tag}: uploaded latest.json with ${Object.keys(platforms).length} platform keys` +
      (useDownloadMirror ? " (download URLs via ghfast)" : ""),
  );
} finally {
  try {
    rmSync(workDir, { recursive: true, force: true });
  } catch {
    /* ignore */
  }
}

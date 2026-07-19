#!/usr/bin/env node
/**
 * 同步 package.json / tauri.conf.json / Cargo.toml 版本号。
 *
 * 用法:
 *   node scripts/sync-version.mjs              # 打印当前版本
 *   node scripts/sync-version.mjs 0.0.5        # 写入指定版本
 *   node scripts/sync-version.mjs patch        # 递增 patch 并写入
 *   node scripts/sync-version.mjs minor|major
 */
import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pkgPath = resolve(root, "package.json");
const tauriPath = resolve(root, "src-tauri/tauri.conf.json");
const cargoPath = resolve(root, "src-tauri/Cargo.toml");

function readPkgVersion() {
  return JSON.parse(readFileSync(pkgPath, "utf8")).version;
}

function writeVersion(version) {
  if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(version)) {
    throw new Error(`Invalid semver: ${version}`);
  }

  const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
  pkg.version = version;
  writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);

  const tauri = JSON.parse(readFileSync(tauriPath, "utf8"));
  tauri.version = version;
  writeFileSync(tauriPath, `${JSON.stringify(tauri, null, 2)}\n`);

  let cargo = readFileSync(cargoPath, "utf8");
  cargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  writeFileSync(cargoPath, cargo);

  console.log(`[sync-version] set version → ${version}`);
}

function bumpVersion(current, kind) {
  const base = current.split("-")[0];
  const parts = base.split(".").map((n) => Number(n));
  if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) {
    throw new Error(`Cannot bump invalid version: ${current}`);
  }
  const [major, minor, patch] = parts;
  if (kind === "major") return `${major + 1}.0.0`;
  if (kind === "minor") return `${major}.${minor + 1}.0`;
  if (kind === "patch") return `${major}.${minor + patch + 1}`;
  throw new Error(`Unknown bump kind: ${kind}`);
}

const arg = process.argv[2];
if (!arg) {
  console.log(readPkgVersion());
  process.exit(0);
}

if (arg === "patch" || arg === "minor" || arg === "major") {
  writeVersion(bumpVersion(readPkgVersion(), arg));
} else {
  writeVersion(arg.replace(/^v/, ""));
}

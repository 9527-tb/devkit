#!/usr/bin/env node
/**
 * 同步 package.json / tauri.conf.json / Cargo.toml 版本号。
 *
 * 用法:
 *   node scripts/sync-version.mjs           # 打印当前版本
 *   node scripts/sync-version.mjs 0.0.2    # 写入三处版本
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

const arg = process.argv[2];
if (!arg) {
  console.log(readPkgVersion());
  process.exit(0);
}
writeVersion(arg.replace(/^v/, ""));

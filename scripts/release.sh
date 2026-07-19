#!/usr/bin/env bash
# DevKit 一键发版：
#   1. 递增版本号（patch / minor / major 或指定 x.y.z）
#   2. 提交并推送当前开发分支
#   3. 合并到 master 并推送
#   4. 在 master 上打 v* 标签并推送 → 触发 GitHub Actions Release
#
# 用法:
#   ./scripts/release.sh                 # 默认 patch +1
#   ./scripts/release.sh minor
#   ./scripts/release.sh 0.0.5
#   ./scripts/release.sh patch --yes     # 跳过确认
#   ./scripts/release.sh patch --dry-run
#
# 环境变量:
#   RELEASE_SOURCE_BRANCH  开发分支（默认：当前分支）
#   RELEASE_TARGET_BRANCH  发版分支（默认：master）
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BUMP="patch"
ASSUME_YES=0
DRY_RUN=0

for arg in "$@"; do
  case "$arg" in
    --yes | -y) ASSUME_YES=1 ;;
    --dry-run) DRY_RUN=1 ;;
    patch | minor | major) BUMP="$arg" ;;
    *)
      if [[ "$arg" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[[:alnum:].]+)?$ ]]; then
        BUMP="$arg"
      elif [[ "$arg" != --* ]]; then
        echo "Unknown argument: $arg" >&2
        exit 1
      fi
      ;;
  esac
done

SOURCE_BRANCH="${RELEASE_SOURCE_BRANCH:-$(git branch --show-current)}"
TARGET_BRANCH="${RELEASE_TARGET_BRANCH:-master}"
ORIG_BRANCH="$SOURCE_BRANCH"

for cmd in git node gh; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
done

if ! gh auth status >/dev/null 2>&1; then
  echo "GitHub CLI not authenticated. Run: gh auth login" >&2
  exit 1
fi

if [[ "$SOURCE_BRANCH" == "$TARGET_BRANCH" ]]; then
  echo "Refusing to release: source branch equals target ($TARGET_BRANCH)." >&2
  echo "Checkout your dev branch (e.g. develop) and run again." >&2
  exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree is not clean. Commit or stash changes first." >&2
  git status -sb
  exit 1
fi

CURRENT="$(node scripts/sync-version.mjs)"
TAG="v${CURRENT}"
if [[ "$BUMP" =~ ^(patch|minor|major)$ ]]; then
  NEXT="$(node -e "
    const fs=require('fs');
    const v=JSON.parse(fs.readFileSync('package.json','utf8')).version;
    const [a,b,c]=v.split('-')[0].split('.').map(Number);
    const k=process.argv[1];
    const n=k==='major'?[a+1,0,0]:k==='minor'?[a,b+1,0]:[a,b,c+1];
    console.log(n.join('.'));
  " "$BUMP")"
else
  NEXT="${BUMP#v}"
fi
TAG_NEXT="v${NEXT}"

if git rev-parse "$TAG_NEXT" >/dev/null 2>&1; then
  echo "Tag already exists locally: $TAG_NEXT" >&2
  exit 1
fi
if gh release view "$TAG_NEXT" --repo "$(gh repo view --json nameWithOwner -q .nameWithOwner)" >/dev/null 2>&1; then
  echo "GitHub release already exists: $TAG_NEXT" >&2
  exit 1
fi

echo "DevKit release plan"
echo "  Current version : $CURRENT"
echo "  New version     : $NEXT ($BUMP)"
echo "  Source branch   : $SOURCE_BRANCH"
echo "  Target branch   : $TARGET_BRANCH"
echo "  Tag             : $TAG_NEXT"
echo "  CI workflow     : .github/workflows/release.yml (on tag push)"
echo

if [[ "$DRY_RUN" == "1" ]]; then
  echo "[dry-run] node scripts/sync-version.mjs $BUMP"
  echo "[dry-run] git commit version files on $SOURCE_BRANCH"
  echo "[dry-run] git push origin $SOURCE_BRANCH"
  echo "[dry-run] git checkout $TARGET_BRANCH && git pull"
  echo "[dry-run] git merge origin/$SOURCE_BRANCH"
  echo "[dry-run] git push origin $TARGET_BRANCH"
  echo "[dry-run] git tag -a $TAG_NEXT && git push origin $TAG_NEXT"
  echo "[dry-run] git checkout $SOURCE_BRANCH && git merge $TARGET_BRANCH"
  exit 0
fi

if [[ "$ASSUME_YES" != "1" ]]; then
  read -r -p "Continue? [y/N] " ans
  if [[ "${ans,,}" != "y" && "${ans,,}" != "yes" ]]; then
    echo "Aborted."
    exit 0
  fi
fi

cleanup() {
  local code=$?
  if [[ -n "${SWITCHED:-}" ]]; then
    git checkout "$ORIG_BRANCH" >/dev/null 2>&1 || true
  fi
  exit "$code"
}
trap cleanup EXIT

echo "→ Sync version to $NEXT"
node scripts/sync-version.mjs "$NEXT"

git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
if ! git diff --cached --quiet; then
  git commit -m "chore: release $TAG_NEXT"
fi

# Cargo.lock 可能随版本变更；若存在未提交的 lock 变更一并带上
if ! git diff --quiet src-tauri/Cargo.lock 2>/dev/null; then
  git add src-tauri/Cargo.lock
  git commit -m "chore: sync Cargo.lock for $TAG_NEXT" || true
fi

echo "→ Push $SOURCE_BRANCH"
git push origin "$SOURCE_BRANCH"

echo "→ Merge into $TARGET_BRANCH"
git fetch origin "$TARGET_BRANCH" "$SOURCE_BRANCH"
git checkout "$TARGET_BRANCH"
SWITCHED=1
git pull --ff-only origin "$TARGET_BRANCH"
git merge --no-ff "origin/$SOURCE_BRANCH" -m "Merge branch '$SOURCE_BRANCH' for release $TAG_NEXT"

echo "→ Push $TARGET_BRANCH"
git push origin "$TARGET_BRANCH"

echo "→ Tag $TAG_NEXT"
git tag -a "$TAG_NEXT" -m "Release $TAG_NEXT"
git push origin "$TAG_NEXT"

echo "→ Sync $SOURCE_BRANCH with $TARGET_BRANCH"
git checkout "$SOURCE_BRANCH"
git merge --no-ff "$TARGET_BRANCH" -m "Merge branch '$TARGET_BRANCH' after release $TAG_NEXT"
git push origin "$SOURCE_BRANCH"

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
RUN_URL="$(gh run list --repo "$REPO" --workflow=Release --limit 1 --json url -q '.[0].url' 2>/dev/null || true)"

echo
echo "Done. Release $TAG_NEXT triggered."
echo "  Tag     : https://github.com/$REPO/releases/tag/$TAG_NEXT"
if [[ -n "$RUN_URL" && "$RUN_URL" != "null" ]]; then
  echo "  Actions : $RUN_URL"
fi
echo
echo "Wait for CI (publish-tauri + fix-updater-json), then verify:"
echo "  curl -sL https://github.com/$REPO/releases/latest/download/latest.json | head"
echo "Installed app can check update after latest.json is published."

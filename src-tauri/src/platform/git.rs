//! Git 只读状态。
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ⑨

use crate::models::GitStatus;
use std::path::Path;
use std::process::Command;

/// 读取工作区根目录的分支与 dirty 状态；非 git 仓返回 available=false。
pub fn workspace_git_status(root: &str) -> GitStatus {
    let root = Path::new(root);
    if !root.is_dir() {
        return GitStatus {
            available: false,
            branch: None,
            dirty: false,
            ahead: None,
            behind: None,
        };
    }

    let Some(git) = crate::platform::find_executable("git") else {
        return GitStatus {
            available: false,
            branch: None,
            dirty: false,
            ahead: None,
            behind: None,
        };
    };

    let inside = Command::new(&git)
        .args(["-C", &root.to_string_lossy(), "rev-parse", "--is-inside-work-tree"])
        .output();
    let ok = matches!(inside, Ok(ref o) if o.status.success());
    if !ok {
        return GitStatus {
            available: false,
            branch: None,
            dirty: false,
            ahead: None,
            behind: None,
        };
    }

    let branch = Command::new(&git)
        .args(["-C", &root.to_string_lossy(), "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    let dirty = Command::new(&git)
        .args(["-C", &root.to_string_lossy(), "status", "--porcelain"])
        .output()
        .ok()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false);

    let (ahead, behind) = ahead_behind(&git, root);

    GitStatus {
        available: true,
        branch,
        dirty,
        ahead,
        behind,
    }
}

fn ahead_behind(git: &Path, root: &Path) -> (Option<i32>, Option<i32>) {
    let output = Command::new(git)
        .args([
            "-C",
            &root.to_string_lossy(),
            "rev-list",
            "--left-right",
            "--count",
            "@{upstream}...HEAD",
        ])
        .output();
    let Ok(out) = output else {
        return (None, None);
    };
    if !out.status.success() {
        return (None, None);
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.split_whitespace();
    let behind = parts.next().and_then(|s| s.parse().ok());
    let ahead = parts.next().and_then(|s| s.parse().ok());
    (ahead, behind)
}

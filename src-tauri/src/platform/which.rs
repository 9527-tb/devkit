//! 可执行文件查找（which / where）。
//!
//! macOS GUI 应用的 PATH 通常不含 Homebrew / nvm，需主动补全后再查找。

use std::env;
use std::path::{Path, PathBuf};

#[cfg(unix)]
fn path_sep() -> char {
    ':'
}

#[cfg(windows)]
fn path_sep() -> char {
    ';'
}

/// 为 GUI / 精简环境补全常用工具路径。
pub fn enriched_path() -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut push = |p: String| {
        if p.is_empty() || parts.iter().any(|x| x == &p) {
            return;
        }
        if Path::new(&p).is_dir() {
            parts.push(p);
        }
    };

    if let Ok(current) = env::var("PATH") {
        for p in current.split(path_sep()) {
            push(p.to_string());
        }
    }

    #[cfg(unix)]
    {
        if let Ok(home) = env::var("HOME") {
            let home = PathBuf::from(home);
            // nvm 当前版本
            let nvm_versions = home.join(".nvm/versions/node");
            if nvm_versions.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_versions) {
                    let mut versions: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                    versions.sort_by_key(|e| std::cmp::Reverse(e.file_name()));
                    for e in versions.into_iter().take(3) {
                        push(e.path().join("bin").to_string_lossy().into_owned());
                    }
                }
            }
            push(home.join(".local/bin").to_string_lossy().into_owned());
            push(home.join(".yarn/bin").to_string_lossy().into_owned());
            push(home.join(".fnm/current/bin").to_string_lossy().into_owned());
            push(
                home.join("Library/pnpm")
                    .to_string_lossy()
                    .into_owned(),
            );
            push(home.join(".volta/bin").to_string_lossy().into_owned());
            push(home.join(".asdf/shims").to_string_lossy().into_owned());
        }
        push("/opt/homebrew/bin".into());
        push("/opt/homebrew/sbin".into());
        push("/usr/local/bin".into());
        push("/usr/local/sbin".into());
        push("/usr/bin".into());
        push("/bin".into());
        push("/usr/sbin".into());
        push("/sbin".into());
    }

    #[cfg(windows)]
    {
        if let Ok(home) = env::var("USERPROFILE") {
            let home = PathBuf::from(home);
            push(home.join("AppData\\Roaming\\npm").to_string_lossy().into_owned());
            push(home.join("AppData\\Local\\pnpm").to_string_lossy().into_owned());
            push(home.join(".yarn\\bin").to_string_lossy().into_owned());
        }
        if let Ok(pf) = env::var("ProgramFiles") {
            push(format!(r"{pf}\nodejs"));
        }
    }

    parts.join(&path_sep().to_string())
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        true
    }
}

#[cfg(windows)]
fn candidate_names(name: &str) -> Vec<String> {
    if name.contains('.') {
        return vec![name.to_string()];
    }
    let mut out = vec![
        format!("{name}.cmd"),
        format!("{name}.exe"),
        format!("{name}.bat"),
        name.to_string(),
    ];
    if let Ok(pathext) = env::var("PATHEXT") {
        for ext in pathext.split(';') {
            let ext = ext.trim();
            if ext.is_empty() {
                continue;
            }
            let cand = if ext.starts_with('.') {
                format!("{name}{ext}")
            } else {
                format!("{name}.{ext}")
            };
            if !out.iter().any(|x| x.eq_ignore_ascii_case(&cand)) {
                out.push(cand);
            }
        }
    }
    out
}

#[cfg(unix)]
fn candidate_names(name: &str) -> Vec<String> {
    vec![name.to_string()]
}

/// 在给定 PATH 字符串中查找可执行文件。
pub fn find_in_path(name: &str, path_env: &str) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }
    let as_path = Path::new(name);
    if as_path.is_absolute() && is_executable(as_path) {
        return Some(as_path.to_path_buf());
    }

    let names = candidate_names(name);
    for dir in path_env.split(path_sep()) {
        if dir.is_empty() {
            continue;
        }
        for n in &names {
            let candidate = Path::new(dir).join(n);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

/// 在补全后的 PATH 中查找可执行文件，返回绝对路径。
pub fn find_executable(name: &str) -> Option<PathBuf> {
    find_in_path(name, &enriched_path())
}

/// 优先在 extra_bins 中查找，再回退到补全 PATH。
pub fn find_executable_with_bins(name: &str, extra_bins: &[PathBuf]) -> Option<PathBuf> {
    let names = candidate_names(name);
    for bin in extra_bins {
        for n in &names {
            let candidate = bin.join(n);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    find_executable(name)
}

//! 用户配置目录与路径规范化。
//!
//! DONE(plat-windows): Windows 使用 %APPDATA%\devkit — DESIGN §5.4 / §16.4

use std::path::PathBuf;


/// 返回 DevKit 用户配置目录（确保已创建）。
pub fn config_dir() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let base = std::env::var("APPDATA").map_err(|_| "无法获取 APPDATA".to_string())?;
        let dir = PathBuf::from(base).join("devkit");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        return Ok(dir);
    }
    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").map_err(|_| "无法获取 HOME 目录".to_string())?;
        let dir = PathBuf::from(home).join(".devkit");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }
}

/// 设置文件完整路径。
pub fn settings_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("settings.json"))
}

/// 规范化路径字符串（统一分隔符语义，便于 projectKey）。
#[allow(dead_code)]
pub fn normalize_path_str(path: &str) -> String {
    let p = PathBuf::from(path);
    match p.canonicalize() {
        Ok(c) => c.to_string_lossy().into_owned(),
        Err(_) => path.replace('\\', "/"),
    }
}

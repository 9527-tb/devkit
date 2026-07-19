//! 最近 / 置顶动作偏好（独立于 settings.json）。
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ④

use crate::models::ActionPrefs;
use crate::platform::paths::config_dir;
use std::fs;

fn prefs_path() -> Result<std::path::PathBuf, String> {
    Ok(config_dir()?.join("action-prefs.json"))
}

pub fn load_action_prefs() -> Result<ActionPrefs, String> {
    let path = prefs_path()?;
    if !path.exists() {
        return Ok(ActionPrefs::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("解析 action-prefs.json 失败: {e}"))
}

pub fn save_action_prefs(prefs: &ActionPrefs) -> Result<(), String> {
    let path = prefs_path()?;
    let text = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

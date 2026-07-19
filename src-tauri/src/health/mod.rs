//! 工作区环境与端口体检。
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ②
//!
//! 端口检查为弱提示：仅对**具体端口**做占用探测。
//! `server.port=0` / 随机端口项目启动前端口未定，跳过占用检查。

use crate::models::{HealthItem, HealthReport, Project};
use crate::platform;
use crate::process::{self, AppState};
use crate::runtime::{self, RuntimeSettings};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
use tauri::AppHandle;

/// 对工作区项目做工具链与弱端口检查。
pub fn check_workspace(
    app: &AppHandle,
    state: &AppState,
    _root: String,
    projects: Vec<Project>,
) -> Result<HealthReport, String> {
    let settings = runtime::load_settings(app)?;
    let mut items = Vec::new();

    let kinds: HashSet<&str> = projects.iter().map(|p| p.kind.as_str()).collect();
    let needs_java = kinds.contains("maven") || kinds.contains("gradle");
    let needs_node = kinds.contains("node");
    let needs_cargo = kinds.contains("cargo");
    let needs_maven = kinds.contains("maven");

    if needs_java {
        items.push(check_java(&settings));
    }
    if needs_maven {
        items.push(check_maven_home(&settings));
    }
    if needs_node {
        items.extend(check_node(&settings));
    }
    if needs_cargo {
        items.push(check_cargo());
    }

    // 弱端口：仅对「上次运行解析到的具体端口」检查；随机端口（0）项目跳过
    if let Ok(ports) = state.last_ports.lock() {
        for project in &projects {
            if project_uses_dynamic_port(project) {
                continue;
            }
            let key = crate::core::project_ref::make_project_key(&project.path, &project.kind);
            let Some(port) = ports.get(&key) else {
                continue;
            };
            let Some(port_num) = process::concrete_listen_port(port) else {
                continue;
            };
            if port_is_listening(port_num) {
                items.push(HealthItem {
                    id: format!("portOccupied:{port_num}"),
                    level: "warn".into(),
                    message: format!(
                        "项目 {} 常用端口 {} 当前已被占用（不会自动结束进程）。",
                        project.name, port_num
                    ),
                    action: Some("tools.ports".into()),
                    project_key: Some(key),
                    port: Some(port_num.to_string()),
                });
            }
        }
    }

    let ok = items.iter().all(|i| i.level == "ok");
    // 只返回非 ok，或全部通过时保留一条 ok 摘要
    let items = if ok {
        vec![HealthItem {
            id: "allOk".into(),
            level: "ok".into(),
            message: "环境检查通过。".into(),
            action: None,
            project_key: None,
            port: None,
        }]
    } else {
        items
            .into_iter()
            .filter(|i| i.level != "ok")
            .collect()
    };

    Ok(HealthReport { ok, items })
}

fn check_java(settings: &RuntimeSettings) -> HealthItem {
    if settings.java.jdks.is_empty() {
        return HealthItem {
            id: "java.jdks".into(),
            level: "warn".into(),
            message: "工作区含 Java 项目，但尚未配置 JDK。".into(),
            action: Some("settings.toolchain-jdk".into()),
            project_key: None,
            port: None,
        };
    }
    let entry = &settings.java.jdks[0];
    let java = Path::new(&entry.path).join("bin").join(if cfg!(windows) {
        "java.exe"
    } else {
        "java"
    });
    let prog = if java.is_file() {
        java
    } else {
        Path::new("java").to_path_buf()
    };
    match Command::new(&prog).arg("-version").output() {
        Ok(out) if out.status.success() || !out.stderr.is_empty() || !out.stdout.is_empty() => {
            HealthItem {
                id: "java.jdks".into(),
                level: "ok".into(),
                message: format!("JDK 可用：{}", entry.label),
                action: None,
                project_key: None,
                port: None,
            }
        }
        _ => HealthItem {
            id: "java.jdks".into(),
            level: "error".into(),
            message: "已配置 JDK，但无法执行 java -version。".into(),
            action: Some("settings.toolchain-jdk".into()),
            project_key: None,
            port: None,
        },
    }
}

fn check_maven_home(settings: &RuntimeSettings) -> HealthItem {
    let home = settings.java.maven_home.trim();
    let mvn_ok = if !home.is_empty() {
        let bin = Path::new(home).join("bin").join(if cfg!(windows) {
            "mvn.cmd"
        } else {
            "mvn"
        });
        bin.is_file() || platform::find_executable("mvn").is_some()
    } else {
        platform::find_executable("mvn").is_some()
    };
    if mvn_ok {
        HealthItem {
            id: "maven.home".into(),
            level: "ok".into(),
            message: "Maven 可用。".into(),
            action: None,
            project_key: None,
            port: None,
        }
    } else {
        HealthItem {
            id: "maven.home".into(),
            level: "warn".into(),
            message: "未找到 mvn：请配置 Maven Home 或将其加入 PATH。".into(),
            action: Some("settings.provider-maven".into()),
            project_key: None,
            port: None,
        }
    }
}

fn check_node(settings: &RuntimeSettings) -> Vec<HealthItem> {
    let mut items = Vec::new();
    if settings.node.nodes.is_empty() {
        items.push(HealthItem {
            id: "node.nodes".into(),
            level: "warn".into(),
            message: "工作区含 Node 项目，但尚未配置 Node。".into(),
            action: Some("settings.toolchain-node".into()),
            project_key: None,
            port: None,
        });
    } else {
        items.push(HealthItem {
            id: "node.nodes".into(),
            level: "ok".into(),
            message: format!("已配置 {} 个 Node。", settings.node.nodes.len()),
            action: None,
            project_key: None,
            port: None,
        });
    }
    let pm = runtime::normalize_node_package_manager(&settings.node.package_manager);
    if platform::find_executable(pm).is_some() {
        items.push(HealthItem {
            id: "node.pm".into(),
            level: "ok".into(),
            message: format!("包管理器 {pm} 在 PATH 中。"),
            action: None,
            project_key: None,
            port: None,
        });
    } else {
        items.push(HealthItem {
            id: "node.pm".into(),
            level: "warn".into(),
            message: format!("默认包管理器 {pm} 未在 PATH 中找到。"),
            action: Some("settings.toolchain-node".into()),
            project_key: None,
            port: None,
        });
    }
    items
}

fn check_cargo() -> HealthItem {
    match Command::new("cargo").arg("-V").output() {
        Ok(out) if out.status.success() => HealthItem {
            id: "cargo".into(),
            level: "ok".into(),
            message: String::from_utf8_lossy(&out.stdout).trim().to_string(),
            action: None,
            project_key: None,
            port: None,
        },
        _ => HealthItem {
            id: "cargo".into(),
            level: "warn".into(),
            message: "未找到 cargo，Cargo 项目将无法构建。".into(),
            action: Some("settings.provider-cargo".into()),
            project_key: None,
            port: None,
        },
    }
}

fn port_is_listening(port: u16) -> bool {
    if port == 0 {
        return false;
    }
    crate::tools::ports::service::lookup_port(&port.to_string(), false)
        .map(|rows| !rows.is_empty())
        .unwrap_or(false)
}

/// 配置为随机端口（如 Spring `server.port=0`）时，启动前无法做占用预检。
fn project_uses_dynamic_port(project: &Project) -> bool {
    let dir = Path::new(&project.path);
    const RELATIVE: &[&str] = &[
        "src/main/resources/application.properties",
        "src/main/resources/application.yml",
        "src/main/resources/application.yaml",
        "src/main/resources/application-dev.properties",
        "src/main/resources/application-dev.yml",
        "application.properties",
        "application.yml",
        "application.yaml",
        ".env",
        ".env.local",
        ".env.development",
    ];
    for rel in RELATIVE {
        let path = dir.join(rel);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if text_declares_dynamic_port(&text) {
            return true;
        }
    }
    false
}

fn text_declares_dynamic_port(text: &str) -> bool {
    static PROP: OnceLock<Regex> = OnceLock::new();
    static YAML_SERVER_PORT: OnceLock<Regex> = OnceLock::new();
    static ENV_PORT: OnceLock<Regex> = OnceLock::new();

    let prop = PROP.get_or_init(|| {
        Regex::new(r"(?i)server\.port\s*[=:]\s*0\b").expect("port prop regex")
    });
    if prop.is_match(text) {
        return true;
    }

    // server:\n  port: 0
    let yaml = YAML_SERVER_PORT.get_or_init(|| {
        Regex::new(r"(?im)^server:\s*(?:\n[ \t]+[^\n]+)*?\n[ \t]+port:\s*0\b")
            .expect("yaml server.port regex")
    });
    if yaml.is_match(text) {
        return true;
    }

    let env = ENV_PORT.get_or_init(|| Regex::new(r"(?im)^PORT\s*=\s*0\s*$").expect("env port"));
    env.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_spring_port_zero_properties() {
        assert!(text_declares_dynamic_port("server.port=0\n"));
        assert!(text_declares_dynamic_port("server.port = 0\n"));
        assert!(!text_declares_dynamic_port("server.port=8080\n"));
    }

    #[test]
    fn detects_spring_port_zero_yaml() {
        assert!(text_declares_dynamic_port("server:\n  port: 0\n"));
        assert!(!text_declares_dynamic_port("server:\n  port: 8080\n"));
    }

    #[test]
    fn detects_env_port_zero() {
        assert!(text_declares_dynamic_port("PORT=0\n"));
        assert!(!text_declares_dynamic_port("PORT=3000\n"));
    }
}

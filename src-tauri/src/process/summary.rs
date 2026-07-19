//! 运行摘要：从日志抽取错误行并匹配可行动 hint。
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ①

use crate::models::{RunHint, RunSummary};
use regex::Regex;
use std::sync::OnceLock;

/// 从日志行构建一次运行的摘要。
pub fn build_summary(
    project_key: String,
    path: String,
    kind: String,
    pid: u32,
    action: String,
    exit_code: Option<i32>,
    duration_ms: u64,
    port: Option<String>,
    log_lines: &[String],
) -> RunSummary {
    let success = exit_code == Some(0);
    let error_lines = extract_error_lines(log_lines, 8);
    let hints = match_hints(log_lines);

    RunSummary {
        project_key,
        path,
        kind,
        pid,
        action,
        exit_code,
        duration_ms,
        success,
        port,
        error_lines,
        hints,
    }
}

fn strip_ansi_simple(input: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"\x1B\[[0-9;?]*[ -/]*[@-~]|\x1B[@-Z\\-_]|\r").expect("ansi")
    });
    re.replace_all(input, "").into_owned()
}

/// 是否像错误行（与前端 classifyLine 思路对齐，略保守）。
pub fn looks_like_error(line: &str) -> bool {
    let s = strip_ansi_simple(line);
    let lower = s.to_ascii_lowercase();
    if lower.contains("[devkit]") && lower.contains("exit 0") {
        return false;
    }
    const KEYS: &[&str] = &[
        "error",
        "exception",
        "fatal",
        "failed",
        "failure",
        "eaddrinuse",
        "address already in use",
        "端口被占用",
        "npm err!",
        "pnpm err",
        "yarn error",
        "could not resolve",
        "command not found",
        "unsupported class file",
        "invalid target release",
        "compilation failure",
        "build failed",
        "error:",
        "err!",
    ];
    KEYS.iter().any(|k| lower.contains(k))
}

fn extract_error_lines(lines: &[String], max: usize) -> Vec<String> {
    let mut out = Vec::new();
    // 优先本轮分隔线之后的行
    let start = lines
        .iter()
        .rposition(|l| l.contains("────────") || l.contains("[DevKit] 执行命令:"))
        .unwrap_or(0);
    let slice = &lines[start..];
    for line in slice.iter().rev() {
        let plain = strip_ansi_simple(line);
        let trimmed = plain.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("[DevKit]") && !looks_like_error(trimmed) {
            continue;
        }
        if looks_like_error(trimmed) {
            out.push(trimmed.to_string());
            if out.len() >= max {
                break;
            }
        }
    }
    out.reverse();
    // 失败但没抓到错误行时，取尾部非空行
    if out.is_empty() {
        for line in slice.iter().rev() {
            let plain = strip_ansi_simple(line);
            let trimmed = plain.trim();
            if trimmed.is_empty() || trimmed.starts_with('─') {
                continue;
            }
            if trimmed.starts_with("[DevKit] 执行命令:") || trimmed.starts_with("[DevKit] JAVA_HOME=") {
                continue;
            }
            out.push(trimmed.to_string());
            if out.len() >= 3 {
                break;
            }
        }
        out.reverse();
    }
    out
}

struct HintRule {
    id: &'static str,
    needles: &'static [&'static str],
    message: &'static str,
    action: &'static str,
}

fn hint_rules() -> &'static [HintRule] {
    &[
        HintRule {
            id: "portInUse",
            needles: &[
                "address already in use",
                "eaddrinuse",
                "端口被占用",
                "bind: address already in use",
            ],
            message: "端口可能被占用，可在「工具 → 端口管理」中查看并结束占用进程。",
            action: "tools.ports",
        },
        HintRule {
            id: "jdkMismatch",
            needles: &[
                "unsupported class file",
                "invalid target release",
                "release version",
                "wrong version of",
                "class file version",
            ],
            message: "JDK 版本可能不匹配，请检查「设置 → 工具链 → Java」。",
            action: "settings.toolchain-jdk",
        },
        HintRule {
            id: "depsMissing",
            needles: &[
                "could not resolve dependencies",
                "npm err!",
                "pnpm err",
                "yarn error",
                "unable to resolve",
                "module not found",
                "cannot find module",
            ],
            message: "依赖可能未就绪，建议先执行 install / 刷新依赖。",
            action: "action.install",
        },
        HintRule {
            id: "toolchainMissing",
            needles: &[
                "command not found: mvn",
                "command not found: cargo",
                "command not found: npm",
                "command not found: node",
                "command not found: gradle",
                "未找到可执行文件",
                "no such file or directory",
            ],
            message: "工具链未配置或不在 PATH，请到「设置」中配置对应运行时。",
            action: "settings.general",
        },
    ]
}

fn match_hints(lines: &[String]) -> Vec<RunHint> {
    let mut blob = String::new();
    let start = lines.len().saturating_sub(400);
    for line in &lines[start..] {
        blob.push_str(&strip_ansi_simple(line).to_ascii_lowercase());
        blob.push('\n');
    }
    let mut out = Vec::new();
    for rule in hint_rules() {
        if rule.needles.iter().any(|n| blob.contains(n)) {
            out.push(RunHint {
                id: rule.id.into(),
                message: rule.message.into(),
                action: Some(rule.action.into()),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_port_hint() {
        let lines = vec![
            "Error: listen EADDRINUSE: address already in use :::3000".into(),
        ];
        let s = build_summary(
            "p::node".into(),
            "/p".into(),
            "node".into(),
            1,
            "start".into(),
            Some(1),
            10,
            None,
            &lines,
        );
        assert!(!s.success);
        assert!(s.hints.iter().any(|h| h.id == "portInUse"));
    }
}

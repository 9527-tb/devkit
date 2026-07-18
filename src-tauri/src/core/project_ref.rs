//! 项目与实例身份键。
//!
//! DONE(rs-instance-key): projectKey = path::kind — DESIGN §16.1

/// 生成稳定项目键：`{normalizedPath}::{kind}`。
pub fn make_project_key(path: &str, kind: &str) -> String {
    let norm = path.replace('\\', "/").trim_end_matches('/').to_string();
    format!("{norm}::{kind}")
}

/// 解析 projectKey，失败返回 None。
pub fn parse_project_key(key: &str) -> Option<(String, String)> {
    let (path, kind) = key.rsplit_once("::")?;
    if path.is_empty() || kind.is_empty() {
        return None;
    }
    Some((path.to_string(), kind.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_key_roundtrip() {
        let key = make_project_key("/tmp/demo", "node");
        assert_eq!(key, "/tmp/demo::node");
        let (p, k) = parse_project_key(&key).unwrap();
        assert_eq!(p, "/tmp/demo");
        assert_eq!(k, "node");
    }

    #[test]
    fn project_key_normalizes_slash() {
        let key = make_project_key(r"C:\work\app\", "maven");
        assert!(key.ends_with("::maven"));
        assert!(!key.contains('\\'));
    }
}

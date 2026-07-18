use crate::models::Dependency;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::fs;

pub fn node_dependency_tree(json: &Value) -> Vec<Dependency> {
    let mut out = vec![];
    for key in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let Some(obj) = json.get(key).and_then(|v| v.as_object()) else {
            continue;
        };
        if obj.is_empty() {
            continue;
        }
        let mut children = vec![];
        for (name, version) in obj {
            children.push(Dependency {
                key: format!("{key}:{name}"),
                name: name.clone(),
                version: version.as_str().unwrap_or("workspace:*").into(),
                scope: key.into(),
                children: None,
            });
        }
        children.sort_by(|a, b| a.name.cmp(&b.name));
        out.push(Dependency {
            key: key.into(),
            name: key.into(),
            // 分组计数文案由前端 i18n 渲染（勿写死语言）
            version: String::new(),
            scope: "group".into(),
            children: Some(children),
        });
    }
    out
}

pub fn parse_pom_tree(path: &Path) -> (Vec<Dependency>, bool) {
    let Ok(xml) = fs::read_to_string(path) else {
        return (vec![], false);
    };
    let spring_boot = xml.contains("spring-boot");
    let mut by_scope: HashMap<String, Vec<Dependency>> = HashMap::new();

    for block in xml.split("<dependency>").skip(1) {
        let Some(end) = block.find("</dependency>") else {
            continue;
        };
        let b = &block[..end];
        let value = |tag: &str| {
            b.split(&format!("<{tag}>"))
                .nth(1)
                .and_then(|s| s.split(&format!("</{tag}>")).next())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let group = value("groupId");
        let artifact = value("artifactId");
        if artifact.is_empty() {
            continue;
        }
        let mut scope = value("scope");
        if scope.is_empty() {
            scope = "compile".into();
        }
        let name = format!("{group}:{artifact}");
        by_scope.entry(scope.clone()).or_default().push(Dependency {
            key: format!("{scope}:{name}"),
            name,
            version: value("version"),
            scope: scope.clone(),
            children: None,
        });
    }

    let order = ["compile", "provided", "runtime", "test", "system", "import"];
    let mut out = vec![];
    for scope in order {
        let Some(mut children) = by_scope.remove(scope) else {
            continue;
        };
        children.sort_by(|a, b| a.name.cmp(&b.name));
        out.push(Dependency {
            key: scope.into(),
            name: scope.into(),
            version: String::new(),
            scope: "group".into(),
            children: Some(children),
        });
    }
    let mut rest: Vec<_> = by_scope.into_iter().collect();
    rest.sort_by(|a, b| a.0.cmp(&b.0));
    for (scope, mut children) in rest {
        children.sort_by(|a, b| a.name.cmp(&b.name));
        out.push(Dependency {
            key: scope.clone(),
            name: scope,
            version: String::new(),
            scope: "group".into(),
            children: Some(children),
        });
    }
    (out, spring_boot)
}

/// Parse `mvn dependency:tree` text into a children tree under a synthetic root group.
pub fn parse_mvn_tree_output(text: &str) -> Vec<Dependency> {
    let mut roots: Vec<Dependency> = vec![];
    let mut stack: Vec<(usize, Dependency)> = vec![];

    for raw in text.lines() {
        let line = raw.trim_end();
        if !line.contains("--") && !line.contains("+-") && !line.contains("\\-") {
            continue;
        }
        let marker = if line.contains("+-") {
            "+-"
        } else if line.contains("\\-") {
            "\\-"
        } else {
            continue;
        };
        let Some(idx) = line.find(marker) else {
            continue;
        };
        let depth = idx / 3;
        let body = line[idx + marker.len()..].trim();
        let body = body.trim_start_matches('-').trim();
        if body.is_empty() {
            continue;
        }
        let parts: Vec<&str> = body.split(':').collect();
        if parts.len() < 2 {
            continue;
        }
        let name = format!("{}:{}", parts[0], parts[1]);
        let version = parts.get(3).or_else(|| parts.get(2)).unwrap_or(&"").to_string();
        let scope = parts
            .get(4)
            .or_else(|| parts.get(3))
            .unwrap_or(&"compile")
            .to_string();
        let node = Dependency {
            key: format!("{depth}:{name}:{version}:{scope}"),
            name,
            version,
            scope,
            children: Some(vec![]),
        };

        while stack.last().map(|(d, _)| *d >= depth).unwrap_or(false) {
            let (d, finished) = stack.pop().unwrap();
            if let Some((_, parent)) = stack.last_mut() {
                parent.children.get_or_insert_with(Vec::new).push(finished);
            } else if d == 0 {
                roots.push(finished);
            } else {
                roots.push(finished);
            }
        }
        stack.push((depth, node));
    }

    while let Some((d, finished)) = stack.pop() {
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.get_or_insert_with(Vec::new).push(finished);
        } else if d == 0 {
            roots.push(finished);
        } else {
            roots.push(finished);
        }
    }

    // Clean empty children to None for leaf nodes
    fn clean(nodes: &mut [Dependency]) {
        for n in nodes.iter_mut() {
            if let Some(children) = n.children.as_mut() {
                if children.is_empty() {
                    n.children = None;
                } else {
                    clean(children);
                }
            }
        }
    }
    clean(&mut roots);

    if roots.is_empty() {
        return vec![];
    }
    vec![Dependency {
        key: "mvn-tree".into(),
        name: "dependency:tree".into(),
        version: String::new(),
        scope: "group".into(),
        children: Some(roots),
    }]
}

pub fn maven_dependency_tree(project_path: &Path) -> Result<Vec<Dependency>, String> {
    let output = Command::new("sh")
        .args([
            "-lc",
            "mvn -q dependency:tree -DoutputType=text 2>/dev/null || mvn dependency:tree -DoutputType=text",
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&output.stdout);
    let tree = parse_mvn_tree_output(&text);
    if tree.is_empty() {
        let pom = project_path.join("pom.xml");
        let (fallback, _) = parse_pom_tree(&pom);
        if fallback.is_empty() {
            return Err("无法解析 Maven 依赖树".into());
        }
        return Ok(fallback);
    }
    Ok(tree)
}

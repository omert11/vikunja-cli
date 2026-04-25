use std::collections::{HashMap, HashSet};

use serde_json::{Map, Number, Value};

use crate::types::{Project, ProjectNode};

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}...")
    }
}

pub fn insert_opt_str(body: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(v) = value {
        body.insert(key.to_string(), Value::String(v));
    }
}

pub fn insert_opt_bool(body: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(v) = value {
        body.insert(key.to_string(), Value::Bool(v));
    }
}

pub fn insert_opt_i64(body: &mut Map<String, Value>, key: &str, value: Option<i64>) {
    if let Some(v) = value {
        body.insert(key.to_string(), Value::Number(v.into()));
    }
}

pub fn insert_opt_f64(body: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(v) = value {
        if let Some(num) = Number::from_f64(v) {
            body.insert(key.to_string(), Value::Number(num));
        }
    }
}

pub fn push_opt<T: ToString>(
    query: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<T>,
) {
    if let Some(v) = value {
        query.push((key, v.to_string()));
    }
}

pub fn children_map(projects: &[Project]) -> HashMap<i64, Vec<i64>> {
    let mut map: HashMap<i64, Vec<i64>> = HashMap::new();
    for p in projects {
        if let Some(parent) = p.parent_project_id.filter(|&id| id != 0) {
            map.entry(parent).or_default().push(p.id);
        }
    }
    map
}

pub fn descendant_ids(map: &HashMap<i64, Vec<i64>>, root: i64, max_depth: usize) -> HashSet<i64> {
    let mut all = HashSet::new();
    all.insert(root);
    let mut stack: Vec<(i64, usize)> = vec![(root, 0)];
    while let Some((current, depth)) = stack.pop() {
        if depth >= max_depth {
            continue;
        }
        if let Some(children) = map.get(&current) {
            for &child in children {
                if all.insert(child) {
                    stack.push((child, depth + 1));
                }
            }
        }
    }
    all
}

/// Replace `project_id = X` with OR-expansion of X and all descendants.
pub fn expand_project_filter(filter: &str, projects: &[Project]) -> String {
    if !filter.contains("project_id") {
        return filter.to_string();
    }
    let Some((start, end, pid)) = find_project_id(filter) else {
        return filter.to_string();
    };
    let map = children_map(projects);
    let ids = descendant_ids(&map, pid, 10);
    if ids.len() <= 1 {
        return filter.to_string();
    }
    let mut sorted: Vec<i64> = ids.into_iter().collect();
    sorted.sort();
    let expansion = sorted
        .iter()
        .map(|id| format!("project_id = {id}"))
        .collect::<Vec<_>>()
        .join(" || ");
    format!("{}({}){}", &filter[..start], expansion, &filter[end..])
}

/// Locate first `project_id\s*=\s*\d+` match without depending on the `regex` crate
/// (saves ~500KB binary growth for a single call site).
fn find_project_id(s: &str) -> Option<(usize, usize, i64)> {
    let key = "project_id";
    let mut idx = 0;
    while let Some(found) = s[idx..].find(key) {
        let abs = idx + found;
        let after = abs + key.len();
        let rest = &s[after..];
        let rest_trim = rest.trim_start();
        if let Some(eq) = rest_trim.strip_prefix('=') {
            let num_part = eq.trim_start();
            let digits: String = num_part
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                if let Ok(pid) = digits.parse::<i64>() {
                    let consumed =
                        (rest_trim.len() - eq.len()) + (eq.len() - num_part.len()) + digits.len();
                    let end = after + (rest.len() - rest_trim.len()) + consumed;
                    return Some((abs, end, pid));
                }
            }
        }
        idx = abs + key.len();
    }
    None
}

pub fn build_tree(projects: &[Project]) -> Vec<ProjectNode> {
    let map = children_map(projects);
    let by_id: HashMap<i64, &Project> = projects.iter().map(|p| (p.id, p)).collect();

    fn build(id: i64, by_id: &HashMap<i64, &Project>, map: &HashMap<i64, Vec<i64>>) -> ProjectNode {
        let title = by_id.get(&id).map(|p| p.title.clone()).unwrap_or_default();
        let children = map
            .get(&id)
            .map(|kids| {
                let mut nodes: Vec<ProjectNode> =
                    kids.iter().map(|&cid| build(cid, by_id, map)).collect();
                nodes.sort_by(|a, b| a.title.cmp(&b.title));
                nodes
            })
            .unwrap_or_default();
        ProjectNode {
            id,
            title,
            children,
        }
    }

    let mut roots: Vec<ProjectNode> = projects
        .iter()
        .filter(|p| p.parent_project_id.unwrap_or(0) == 0)
        .map(|p| build(p.id, &by_id, &map))
        .collect();
    roots.sort_by(|a, b| a.title.cmp(&b.title));
    roots
}

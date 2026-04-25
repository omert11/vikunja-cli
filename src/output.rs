use anyhow::Result;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use serde::Serialize;
use serde_json::Value;

use crate::types::{Label, Project, ProjectNode, Task, ZERO_DATE};
use crate::util::truncate;

pub fn emit_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn emit_value(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Dispatch on `--json` flag: emit JSON or call the human-formatter.
pub fn render<T: Serialize>(value: &T, json: bool, human: impl FnOnce(&T)) -> Result<()> {
    if json {
        emit_json(value)
    } else {
        human(value);
        Ok(())
    }
}

fn join_field<T>(items: &[T], extract: impl Fn(&T) -> &str) -> String {
    items.iter().map(extract).collect::<Vec<_>>().join(", ")
}

pub fn print_task_table(tasks: &[Task]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "ID",
            "Title",
            "Done",
            "Pri",
            "Project",
            "Due",
            "Labels",
            "Assignees",
        ]);
    for t in tasks {
        let due = t
            .due_date
            .as_deref()
            .filter(|d| *d != ZERO_DATE && !d.is_empty())
            .unwrap_or("-");
        let labels = t
            .labels
            .as_deref()
            .map(|ls| join_field(ls, |l| l.title.as_str()))
            .unwrap_or_default();
        let assignees = t
            .assignees
            .as_deref()
            .map(|xs| join_field(xs, |a| a.username.as_str()))
            .unwrap_or_default();
        table.add_row(vec![
            Cell::new(t.id),
            Cell::new(truncate(&t.title, 60)),
            Cell::new(if t.done { "✓" } else { "" }),
            Cell::new(t.priority),
            Cell::new(t.project_id),
            Cell::new(due),
            Cell::new(truncate(&labels, 30)),
            Cell::new(truncate(&assignees, 30)),
        ]);
    }
    println!("{table}");
    println!("{} {}", tasks.len().to_string().bold(), "tasks".dimmed());
}

pub fn print_task_detail(t: &Task) {
    println!("{} {}", "Task".bold().underline(), t.id.to_string().cyan());
    println!("  {} {}", "Title:".bold(), t.title);
    println!(
        "  {} {}",
        "Done:".bold(),
        if t.done {
            "✓".green().to_string()
        } else {
            "✗".yellow().to_string()
        }
    );
    println!("  {} {}", "Priority:".bold(), t.priority);
    println!("  {} {}", "Project:".bold(), t.project_id);
    if let Some(due) = t
        .due_date
        .as_deref()
        .filter(|d| *d != ZERO_DATE && !d.is_empty())
    {
        println!("  {} {}", "Due:".bold(), due);
    }
    if !t.description.is_empty() {
        println!("  {}", "Description:".bold());
        for line in t.description.lines() {
            println!("    {line}");
        }
    }
    if let Some(labels) = t.labels.as_deref().filter(|v| !v.is_empty()) {
        println!(
            "  {} {}",
            "Labels:".bold(),
            join_field(labels, |l| l.title.as_str())
        );
    }
    if let Some(assignees) = t.assignees.as_deref().filter(|v| !v.is_empty()) {
        println!(
            "  {} {}",
            "Assignees:".bold(),
            join_field(assignees, |a| a.username.as_str())
        );
    }
}

pub fn print_project_tree(roots: &[ProjectNode]) {
    fn walk(node: &ProjectNode, depth: usize) {
        let indent = "  ".repeat(depth);
        let id = format!("[{}]", node.id).dimmed();
        println!("{indent}{id} {}", node.title);
        for child in &node.children {
            walk(child, depth + 1);
        }
    }
    for r in roots {
        walk(r, 0);
    }
    let total = count_nodes(roots);
    println!("{} {}", total.to_string().bold(), "projects".dimmed());
}

fn count_nodes(nodes: &[ProjectNode]) -> usize {
    nodes.iter().map(|n| 1 + count_nodes(&n.children)).sum()
}

pub fn print_project_detail(p: &Project) {
    println!(
        "{} {}",
        "Project".bold().underline(),
        p.id.to_string().cyan()
    );
    println!("  {} {}", "Title:".bold(), p.title);
    if !p.description.is_empty() {
        println!("  {} {}", "Description:".bold(), p.description);
    }
    if let Some(parent) = p.parent_project_id.filter(|&id| id != 0) {
        println!("  {} {}", "Parent:".bold(), parent);
    }
    if p.is_archived {
        println!("  {} {}", "Status:".bold(), "archived".yellow());
    }
    if let Some(c) = p.hex_color.as_deref().filter(|s| !s.is_empty()) {
        println!("  {} #{}", "Color:".bold(), c);
    }
}

pub fn print_label_table(labels: &[Label]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["ID", "Title", "Color"]);
    for l in labels {
        let color = l.hex_color.as_deref().unwrap_or("");
        table.add_row(vec![Cell::new(l.id), Cell::new(&l.title), Cell::new(color)]);
    }
    println!("{table}");
    println!("{} {}", labels.len().to_string().bold(), "labels".dimmed());
}

pub fn print_message(msg: &str) {
    println!("{} {}", "→".bold().green(), msg);
}

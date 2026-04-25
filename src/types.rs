use serde::{Deserialize, Serialize};

pub const ZERO_DATE: &str = "0001-01-01T00:00:00Z";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub project_id: i64,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<Label>>,
    #[serde(default)]
    pub assignees: Option<Vec<Assignee>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub hex_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignee {
    pub id: i64,
    #[serde(default)]
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parent_project_id: Option<i64>,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub hex_color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectNode {
    pub id: i64,
    pub title: String,
    pub children: Vec<ProjectNode>,
}

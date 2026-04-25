use anyhow::Result;
use clap::Subcommand;
use futures::future::try_join_all;
use serde_json::{json, Map, Value};

use crate::client::VikunjaClient;
use crate::output;
use crate::types::{Project, Task};
use crate::util::{
    expand_project_filter, insert_opt_bool, insert_opt_f64, insert_opt_i64, insert_opt_str,
    push_opt,
};

#[derive(Subcommand)]
pub enum TaskCmd {
    /// List/filter/search tasks across all projects
    List {
        /// Vikunja filter expression (e.g. "done = false && priority >= 3")
        #[arg(short, long)]
        filter: Option<String>,
        /// Free-text search
        #[arg(short, long)]
        search: Option<String>,
        /// Sort field (id, title, done, due_date, priority, project_id, created, updated)
        #[arg(long)]
        sort_by: Option<String>,
        /// Order (asc, desc)
        #[arg(long)]
        order_by: Option<String>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Per-page count (default: 50)
        #[arg(long)]
        per_page: Option<u32>,
    },
    /// Get a single task by ID
    Get { id: i64 },
    /// Get full details for multiple task IDs
    GetDetail {
        /// Comma-separated task IDs (max 25)
        ids: String,
    },
    /// Create a new task in a project
    Create {
        project_id: i64,
        title: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<i64>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        start_date: Option<String>,
        #[arg(long)]
        end_date: Option<String>,
        #[arg(long)]
        hex_color: Option<String>,
    },
    /// Update an existing task
    Update {
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        done: Option<bool>,
        #[arg(long)]
        priority: Option<i64>,
        #[arg(long)]
        due_date: Option<String>,
        #[arg(long)]
        percent_done: Option<f64>,
        #[arg(long)]
        is_favorite: Option<bool>,
    },
    /// Delete a task
    Delete { id: i64 },
    /// Comment subcommands
    Comment {
        #[command(subcommand)]
        cmd: CommentCmd,
    },
    /// Assign a user to a task
    Assign { task_id: i64, user_id: i64 },
    /// Remove a user from a task
    Unassign { task_id: i64, user_id: i64 },
    /// Set labels on a task (replaces existing)
    Labels {
        task_id: i64,
        /// Comma-separated label IDs (empty to clear)
        #[arg(long)]
        ids: String,
    },
}

#[derive(Subcommand)]
pub enum CommentCmd {
    /// List comments on a task
    List { task_id: i64 },
    /// Add a comment to a task
    Create {
        task_id: i64,
        #[arg(short, long)]
        comment: String,
    },
}

pub async fn run(cmd: TaskCmd, client: &VikunjaClient, json: bool) -> Result<()> {
    match cmd {
        TaskCmd::List {
            filter,
            search,
            sort_by,
            order_by,
            page,
            per_page,
        } => {
            list(
                client, filter, search, sort_by, order_by, page, per_page, json,
            )
            .await
        }
        TaskCmd::Get { id } => get(client, id, json).await,
        TaskCmd::GetDetail { ids } => get_detail(client, &ids, json).await,
        TaskCmd::Create {
            project_id,
            title,
            description,
            priority,
            due_date,
            start_date,
            end_date,
            hex_color,
        } => {
            create(
                client,
                project_id,
                title,
                description,
                priority,
                due_date,
                start_date,
                end_date,
                hex_color,
                json,
            )
            .await
        }
        TaskCmd::Update {
            id,
            title,
            description,
            done,
            priority,
            due_date,
            percent_done,
            is_favorite,
        } => {
            update(
                client,
                id,
                title,
                description,
                done,
                priority,
                due_date,
                percent_done,
                is_favorite,
                json,
            )
            .await
        }
        TaskCmd::Delete { id } => delete(client, id, json).await,
        TaskCmd::Comment { cmd } => match cmd {
            CommentCmd::List { task_id } => comment_list(client, task_id, json).await,
            CommentCmd::Create { task_id, comment } => {
                comment_create(client, task_id, comment, json).await
            }
        },
        TaskCmd::Assign { task_id, user_id } => assign(client, task_id, user_id, json).await,
        TaskCmd::Unassign { task_id, user_id } => unassign(client, task_id, user_id, json).await,
        TaskCmd::Labels { task_id, ids } => set_labels(client, task_id, &ids, json).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn list(
    client: &VikunjaClient,
    filter: Option<String>,
    search: Option<String>,
    sort_by: Option<String>,
    order_by: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
    json: bool,
) -> Result<()> {
    // Vikunja filter expansion: when the filter mentions `project_id`, fetch the
    // full project list so we can OR-expand sub-projects. One-shot CLI, no shared
    // cache to leverage — this round-trip happens per `task list` invocation.
    let expanded_filter = if filter.as_deref().is_some_and(|f| f.contains("project_id")) {
        let projects: Vec<Project> =
            serde_json::from_value(client.get::<()>("/projects", None).await?).unwrap_or_default();
        filter.map(|f| expand_project_filter(&f, &projects))
    } else {
        filter
    };

    let mut query: Vec<(&'static str, String)> = Vec::new();
    push_opt(&mut query, "filter", expanded_filter);
    push_opt(&mut query, "s", search);
    push_opt(&mut query, "sort_by", sort_by);
    push_opt(&mut query, "order_by", order_by);
    push_opt(&mut query, "page", page);
    query.push(("per_page", per_page.unwrap_or(50).to_string()));

    let value = client.get("/tasks", Some(&query)).await?;
    let tasks: Vec<Task> = serde_json::from_value(value).unwrap_or_default();
    output::render(&tasks, json, |t| output::print_task_table(t))
}

async fn get(client: &VikunjaClient, id: i64, json: bool) -> Result<()> {
    let value = client.get::<()>(&format!("/tasks/{id}"), None).await?;
    if json {
        return output::emit_value(&value);
    }
    let task: Task = serde_json::from_value(value)?;
    output::print_task_detail(&task);
    Ok(())
}

async fn get_detail(client: &VikunjaClient, ids: &str, json: bool) -> Result<()> {
    let id_list: Vec<i64> = ids
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if id_list.len() > 25 {
        anyhow::bail!("Maximum 25 task IDs per request");
    }

    let paths: Vec<String> = id_list.iter().map(|id| format!("/tasks/{id}")).collect();
    let values: Vec<Value> = try_join_all(paths.iter().map(|p| client.get::<()>(p, None))).await?;
    let tasks: Vec<Task> = values
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    output::render(&tasks, json, |t| output::print_task_table(t))
}

#[allow(clippy::too_many_arguments)]
async fn create(
    client: &VikunjaClient,
    project_id: i64,
    title: String,
    description: Option<String>,
    priority: Option<i64>,
    due_date: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    hex_color: Option<String>,
    json: bool,
) -> Result<()> {
    let mut body: Map<String, Value> = Map::new();
    body.insert("title".into(), Value::String(title));
    insert_opt_str(&mut body, "description", description);
    insert_opt_i64(&mut body, "priority", priority);
    insert_opt_str(&mut body, "due_date", due_date);
    insert_opt_str(&mut body, "start_date", start_date);
    insert_opt_str(&mut body, "end_date", end_date);
    insert_opt_str(&mut body, "hex_color", hex_color);

    let value = client
        .put(
            &format!("/projects/{project_id}/tasks"),
            Some(&Value::Object(body)),
        )
        .await?;
    if json {
        return output::emit_value(&value);
    }
    let task: Task = serde_json::from_value(value)?;
    output::print_task_detail(&task);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn update(
    client: &VikunjaClient,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    done: Option<bool>,
    priority: Option<i64>,
    due_date: Option<String>,
    percent_done: Option<f64>,
    is_favorite: Option<bool>,
    json: bool,
) -> Result<()> {
    let mut body: Map<String, Value> = Map::new();
    insert_opt_str(&mut body, "title", title);
    insert_opt_str(&mut body, "description", description);
    insert_opt_bool(&mut body, "done", done);
    insert_opt_i64(&mut body, "priority", priority);
    insert_opt_str(&mut body, "due_date", due_date);
    insert_opt_f64(&mut body, "percent_done", percent_done);
    insert_opt_bool(&mut body, "is_favorite", is_favorite);

    if body.is_empty() {
        anyhow::bail!("No fields provided to update");
    }

    let value = client
        .post(&format!("/tasks/{id}"), Some(&Value::Object(body)))
        .await?;
    if json {
        return output::emit_value(&value);
    }
    let task: Task = serde_json::from_value(value)?;
    output::print_task_detail(&task);
    Ok(())
}

async fn delete(client: &VikunjaClient, id: i64, json: bool) -> Result<()> {
    client.delete(&format!("/tasks/{id}")).await?;
    output::render(&json!({"deleted": id}), json, |_| {
        output::print_message(&format!("Task {id} deleted"))
    })
}

async fn comment_list(client: &VikunjaClient, task_id: i64, json: bool) -> Result<()> {
    let value = client
        .get::<()>(&format!("/tasks/{task_id}/comments"), None)
        .await?;
    if json {
        return output::emit_value(&value);
    }
    let Some(arr) = value.as_array() else {
        return output::emit_value(&value);
    };
    if arr.is_empty() {
        output::print_message("No comments");
        return Ok(());
    }
    for c in arr {
        let id = c.get("id").and_then(|v| v.as_i64()).unwrap_or_default();
        let comment = c.get("comment").and_then(|v| v.as_str()).unwrap_or("");
        let author = c
            .get("author")
            .and_then(|a| a.get("username").and_then(|v| v.as_str()))
            .unwrap_or("");
        let created = c.get("created").and_then(|v| v.as_str()).unwrap_or("");
        println!("[{id}] {author} @ {created}");
        for line in comment.lines() {
            println!("  {line}");
        }
    }
    Ok(())
}

async fn comment_create(
    client: &VikunjaClient,
    task_id: i64,
    comment: String,
    json: bool,
) -> Result<()> {
    let value = client
        .put(
            &format!("/tasks/{task_id}/comments"),
            Some(&json!({ "comment": comment })),
        )
        .await?;
    if json {
        return output::emit_value(&value);
    }
    let id = value.get("id").and_then(|v| v.as_i64()).unwrap_or_default();
    output::print_message(&format!("Comment {id} added"));
    Ok(())
}

async fn assign(client: &VikunjaClient, task_id: i64, user_id: i64, json: bool) -> Result<()> {
    let value = client
        .put(
            &format!("/tasks/{task_id}/assignees"),
            Some(&json!({ "user_id": user_id })),
        )
        .await?;
    if json {
        return output::emit_value(&value);
    }
    output::print_message(&format!("User {user_id} assigned to task {task_id}"));
    Ok(())
}

async fn unassign(client: &VikunjaClient, task_id: i64, user_id: i64, json: bool) -> Result<()> {
    client
        .delete(&format!("/tasks/{task_id}/assignees/{user_id}"))
        .await?;
    output::render(
        &json!({"unassigned": user_id, "task_id": task_id}),
        json,
        |_| output::print_message(&format!("User {user_id} removed from task {task_id}")),
    )
}

async fn set_labels(client: &VikunjaClient, task_id: i64, ids: &str, json: bool) -> Result<()> {
    let label_ids: Vec<i64> = ids
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            (!trimmed.is_empty())
                .then(|| trimmed.parse().ok())
                .flatten()
        })
        .collect();
    let value = client
        .post(
            &format!("/tasks/{task_id}/labels/bulk"),
            Some(&json!({ "label_ids": label_ids })),
        )
        .await?;
    if json {
        return output::emit_value(&value);
    }
    output::print_message(&format!(
        "Labels set on task {task_id}: {} label(s)",
        label_ids.len()
    ));
    Ok(())
}

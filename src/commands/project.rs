use anyhow::Result;
use clap::Subcommand;
use serde_json::{Map, Value};

use crate::client::VikunjaClient;
use crate::output;
use crate::types::Project;
use crate::util::{build_tree, insert_opt_bool, insert_opt_i64, insert_opt_str, push_opt};

#[derive(Subcommand)]
pub enum ProjectCmd {
    /// List all projects (hierarchical tree)
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        per_page: Option<u32>,
        /// Search projects by name (client-side, since Vikunja /projects has no `s` param)
        #[arg(short, long)]
        search: Option<String>,
        /// Show only archived
        #[arg(long)]
        archived: Option<bool>,
    },
    /// Get a single project by ID
    Get { id: i64 },
    /// Create a new project
    Create {
        title: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(long)]
        hex_color: Option<String>,
        #[arg(long)]
        parent: Option<i64>,
        #[arg(long)]
        archived: Option<bool>,
    },
    /// Update a project
    Update {
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        hex_color: Option<String>,
        #[arg(long)]
        archived: Option<bool>,
        #[arg(long)]
        favorite: Option<bool>,
    },
}

pub async fn run(cmd: ProjectCmd, client: &VikunjaClient, json: bool) -> Result<()> {
    match cmd {
        ProjectCmd::List {
            page,
            per_page,
            search,
            archived,
        } => list(client, page, per_page, search, archived, json).await,
        ProjectCmd::Get { id } => get(client, id, json).await,
        ProjectCmd::Create {
            title,
            description,
            hex_color,
            parent,
            archived,
        } => {
            create(
                client,
                title,
                description,
                hex_color,
                parent,
                archived,
                json,
            )
            .await
        }
        ProjectCmd::Update {
            id,
            title,
            description,
            hex_color,
            archived,
            favorite,
        } => {
            update(
                client,
                id,
                title,
                description,
                hex_color,
                archived,
                favorite,
                json,
            )
            .await
        }
    }
}

async fn list(
    client: &VikunjaClient,
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    archived: Option<bool>,
    json: bool,
) -> Result<()> {
    let mut query: Vec<(&'static str, String)> = Vec::new();
    push_opt(&mut query, "page", page);
    query.push(("per_page", per_page.unwrap_or(100).to_string()));
    push_opt(&mut query, "is_archived", archived);

    let value = client.get("/projects", Some(&query)).await?;
    let projects: Vec<Project> = serde_json::from_value(value).unwrap_or_default();

    let mut filtered: Vec<Project> = if let Some(q) = search {
        let needle = q.to_lowercase();
        projects
            .into_iter()
            .filter(|p| p.title.to_lowercase().contains(&needle))
            .collect()
    } else {
        projects
    };

    filtered.sort_by_key(|p| p.id);
    let tree = build_tree(&filtered);
    output::render(&tree, json, |t| output::print_project_tree(t))
}

async fn get(client: &VikunjaClient, id: i64, json: bool) -> Result<()> {
    let value = client.get::<()>(&format!("/projects/{id}"), None).await?;
    if json {
        return output::emit_value(&value);
    }
    let p: Project = serde_json::from_value(value)?;
    output::print_project_detail(&p);
    Ok(())
}

async fn create(
    client: &VikunjaClient,
    title: String,
    description: Option<String>,
    hex_color: Option<String>,
    parent: Option<i64>,
    archived: Option<bool>,
    json: bool,
) -> Result<()> {
    let mut body: Map<String, Value> = Map::new();
    body.insert("title".into(), Value::String(title));
    insert_opt_str(&mut body, "description", description);
    insert_opt_str(&mut body, "hex_color", hex_color);
    insert_opt_i64(&mut body, "parent_project_id", parent);
    insert_opt_bool(&mut body, "is_archived", archived);

    let value = client.put("/projects", Some(&Value::Object(body))).await?;
    if json {
        return output::emit_value(&value);
    }
    let p: Project = serde_json::from_value(value)?;
    output::print_project_detail(&p);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn update(
    client: &VikunjaClient,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    hex_color: Option<String>,
    archived: Option<bool>,
    favorite: Option<bool>,
    json: bool,
) -> Result<()> {
    let mut body: Map<String, Value> = Map::new();
    insert_opt_str(&mut body, "title", title);
    insert_opt_str(&mut body, "description", description);
    insert_opt_str(&mut body, "hex_color", hex_color);
    insert_opt_bool(&mut body, "is_archived", archived);
    insert_opt_bool(&mut body, "is_favorite", favorite);

    if body.is_empty() {
        anyhow::bail!("No fields provided to update");
    }

    let value = client
        .post(&format!("/projects/{id}"), Some(&Value::Object(body)))
        .await?;
    if json {
        return output::emit_value(&value);
    }
    let p: Project = serde_json::from_value(value)?;
    output::print_project_detail(&p);
    Ok(())
}

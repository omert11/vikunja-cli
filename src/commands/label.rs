use anyhow::Result;
use clap::Subcommand;

use crate::client::VikunjaClient;
use crate::output;
use crate::types::Label;
use crate::util::push_opt;

#[derive(Subcommand)]
pub enum LabelCmd {
    /// List all labels
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        per_page: Option<u32>,
        /// Search by label title
        #[arg(short, long)]
        search: Option<String>,
    },
}

pub async fn run(cmd: LabelCmd, client: &VikunjaClient, json: bool) -> Result<()> {
    match cmd {
        LabelCmd::List {
            page,
            per_page,
            search,
        } => list(client, page, per_page, search, json).await,
    }
}

async fn list(
    client: &VikunjaClient,
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    json: bool,
) -> Result<()> {
    let mut query: Vec<(&'static str, String)> = Vec::new();
    push_opt(&mut query, "page", page);
    query.push(("per_page", per_page.unwrap_or(100).to_string()));
    push_opt(&mut query, "s", search);

    let value = client.get("/labels", Some(&query)).await?;
    let labels: Vec<Label> = serde_json::from_value(value).unwrap_or_default();
    output::render(&labels, json, |l| output::print_label_table(l))
}

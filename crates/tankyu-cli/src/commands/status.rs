use anyhow::Result;
use tankyu_core::domain::ports::IEntryStore;

use crate::context::AppContext;

pub async fn run(ctx: &AppContext) -> Result<()> {
    let topics = ctx.topic_mgr.list_all().await?;
    let sources = ctx.source_mgr.list_all().await?;
    let entries = ctx.entry_store.list().await?;

    if ctx.output.is_json() {
        println!(
            "{}",
            serde_json::json!({
                "topics": topics.len(),
                "sources": sources.len(),
                "entries": entries.len()
            })
        );
        return Ok(());
    }

    let mut table = comfy_table::Table::new();
    table.set_header(["Metric", "Count"]);
    table.add_row(["Topics", &topics.len().to_string()]);
    table.add_row(["Sources", &sources.len().to_string()]);
    table.add_row(["Entries", &entries.len().to_string()]);
    println!("{table}");
    Ok(())
}

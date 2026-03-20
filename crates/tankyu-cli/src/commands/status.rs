use anyhow::Result;

use crate::context::AppContext;

fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {plural}")
    }
}

pub async fn run(ctx: &AppContext) -> Result<()> {
    let report = ctx.status_uc.run().await?;

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Research Graph Status");
    println!("  {}", pluralize(report.topics, "topic", "topics"));
    println!("  {}", pluralize(report.sources, "source", "sources"));
    println!("  {}", pluralize(report.entries, "entry", "entries"));
    Ok(())
}

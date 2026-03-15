use anyhow::Result;

use crate::context::AppContext;

pub async fn list(ctx: &AppContext) -> Result<()> {
    let topics = ctx.topic_mgr.list_all().await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&topics)?);
        return Ok(());
    }
    let mut table = comfy_table::Table::new();
    table.set_header(["Name", "Tags", "Scans"]);
    for t in &topics {
        table.add_row([&t.name, &t.tags.join(", "), &t.scan_count.to_string()]);
    }
    println!("{table}");
    Ok(())
}

pub async fn inspect(ctx: &AppContext, name: &str) -> Result<()> {
    let t = ctx
        .topic_mgr
        .get_by_name(name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Topic '{name}' not found"))?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&t)?);
        return Ok(());
    }
    println!("Name:        {}", t.name);
    println!("Description: {}", t.description);
    println!("Tags:        {}", t.tags.join(", "));
    println!("Scan count:  {}", t.scan_count);
    println!("Created:     {}", t.created_at.format("%Y-%m-%d"));
    Ok(())
}

pub async fn create(ctx: &AppContext, name: &str, description: &str, tags_csv: &str) -> Result<()> {
    use tankyu_core::features::topic::topic_manager::CreateTopicInput;
    let tags: Vec<String> = tags_csv
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let topic = ctx
        .topic_mgr
        .create(CreateTopicInput {
            name: name.to_string(),
            description: description.to_string(),
            tags: tags.clone(),
        })
        .await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&topic)?);
        return Ok(());
    }
    println!("Created topic: {} ({})", topic.name, topic.id);
    if !tags.is_empty() {
        println!("  Tags: {}", tags.join(", "));
    }
    Ok(())
}

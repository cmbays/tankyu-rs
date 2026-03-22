use anyhow::Result;
use uuid::Uuid;

use tankyu_core::domain::types::{EntryState, EntryType, Signal};

use crate::context::AppContext;

fn parse_state(s: &str) -> Result<EntryState> {
    match s {
        "new" => Ok(EntryState::New),
        "scanned" => Ok(EntryState::Scanned),
        "triaged" => Ok(EntryState::Triaged),
        "read" => Ok(EntryState::Read),
        "archived" => Ok(EntryState::Archived),
        _ => Err(anyhow::anyhow!(
            "Invalid state '{s}'. Valid: new, scanned, triaged, read, archived"
        )),
    }
}

fn parse_signal(s: &str) -> Result<Signal> {
    match s {
        "high" => Ok(Signal::High),
        "medium" => Ok(Signal::Medium),
        "low" => Ok(Signal::Low),
        "noise" => Ok(Signal::Noise),
        _ => Err(anyhow::anyhow!(
            "Invalid signal '{s}'. Valid: high, medium, low, noise"
        )),
    }
}

const fn state_str(s: &EntryState) -> &'static str {
    match s {
        EntryState::New => "new",
        EntryState::Scanned => "scanned",
        EntryState::Triaged => "triaged",
        EntryState::Read => "read",
        EntryState::Archived => "archived",
    }
}

const fn signal_str(s: Option<&Signal>) -> &'static str {
    match s {
        None => "—",
        Some(Signal::High) => "high",
        Some(Signal::Medium) => "medium",
        Some(Signal::Low) => "low",
        Some(Signal::Noise) => "noise",
    }
}

const fn type_str(t: &EntryType) -> &'static str {
    match t {
        EntryType::Tweet => "tweet",
        EntryType::Commit => "commit",
        EntryType::Pr => "pr",
        EntryType::Release => "release",
        EntryType::Article => "article",
        EntryType::Page => "page",
        EntryType::Repo => "repo",
        EntryType::GithubIssue => "github-issue",
        EntryType::SpikeReport => "spike-report",
    }
}

pub async fn list(
    ctx: &AppContext,
    state: Option<&str>,
    signal: Option<&str>,
    source: Option<&str>,
    topic: Option<&str>,
    limit: Option<usize>,
    unclassified: bool,
) -> Result<()> {
    if unclassified && (source.is_some() || topic.is_some()) {
        anyhow::bail!("--unclassified is mutually exclusive with --topic and --source");
    }
    if source.is_some() && topic.is_some() {
        anyhow::bail!("--topic and --source are mutually exclusive");
    }

    // Validate filters before any I/O so invalid args fail fast.
    let state_filter = state.map(parse_state).transpose()?;
    let signal_filter = signal.map(parse_signal).transpose()?;

    let mut entries = if unclassified {
        use tankyu_core::domain::types::{EdgeType, GraphQuery, NodeType};
        let classified_ids: std::collections::HashSet<_> = ctx
            .graph_store
            .query(GraphQuery {
                edge_type: Some(EdgeType::TaggedWith),
                from_type: Some(NodeType::Entry),
                ..Default::default()
            })
            .await?
            .into_iter()
            .map(|e| e.from_id)
            .collect();
        let all = ctx.entry_mgr.list_all().await?;
        all.into_iter()
            .filter(|e| !classified_ids.contains(&e.id))
            .collect::<Vec<_>>()
    } else {
        match (source, topic) {
            (Some(name), None) => {
                let src = ctx
                    .source_mgr
                    .get_by_name(name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Source '{name}' not found"))?;
                ctx.entry_mgr.list_by_source(src.id).await?
            }
            (None, Some(name)) => {
                let t = ctx
                    .topic_mgr
                    .get_by_name(name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Topic '{name}' not found"))?;
                ctx.entry_mgr.list_by_topic(t.id).await?
            }
            (None, None) => ctx.entry_mgr.list_all().await?,
            (Some(_), Some(_)) => unreachable!("mutually exclusive guard above"),
        }
    };

    if let Some(filter) = state_filter {
        entries.retain(|e| e.state == filter);
    }

    if let Some(filter) = signal_filter {
        entries.retain(|e| e.signal.as_ref() == Some(&filter));
    }

    entries.sort_by(|a, b| b.scanned_at.cmp(&a.scanned_at));

    if let Some(n) = limit {
        entries.truncate(n);
    }

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&entries)?);
        return Ok(());
    }

    if entries.is_empty() {
        println!("No entries yet. Entries appear after scanning sources.");
        return Ok(());
    }

    let mut table = comfy_table::Table::new();
    table.set_header(["ID", "Type", "State", "Signal", "Title", "Scanned"]);
    for e in &entries {
        let id_str = e.id.to_string();
        let title: String = if e.title.chars().count() > 60 {
            format!("{}…", e.title.chars().take(59).collect::<String>())
        } else {
            e.title.clone()
        };
        table.add_row([
            &id_str,
            type_str(&e.r#type),
            state_str(&e.state),
            signal_str(e.signal.as_ref()),
            &title,
            &e.scanned_at.format("%Y-%m-%d").to_string(),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub async fn inspect(ctx: &AppContext, id: &str) -> Result<()> {
    let uuid = Uuid::parse_str(id)
        .map_err(|_| anyhow::anyhow!("'{id}' is not a valid entry ID (expected a UUID)"))?;
    let e = ctx
        .entry_mgr
        .get(uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Entry '{id}' not found"))?;

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&e)?);
        return Ok(());
    }

    let source_name = match ctx.source_mgr.get_by_id(e.source_id).await {
        Ok(Some(s)) => s.name,
        Ok(None) => e.source_id.to_string(),
        Err(_) => {
            eprintln!("warning: could not look up source {}", e.source_id);
            e.source_id.to_string()
        }
    };

    let topics = ctx.topic_mgr.list_by_entry(e.id).await?;
    let topic_names: Vec<&str> = topics.iter().map(|t| t.name.as_str()).collect();

    println!("ID:        {}", e.id);
    println!("Type:      {}", type_str(&e.r#type));
    println!("State:     {}", state_str(&e.state));
    println!("Signal:    {}", signal_str(e.signal.as_ref()));
    println!("Title:     {}", e.title);
    println!("URL:       {}", e.url);
    println!("Source:    {source_name}");
    println!("Summary:   {}", e.summary.as_deref().unwrap_or("—"));
    println!("Scanned:   {}", e.scanned_at);
    println!("Created:   {}", e.created_at.format("%Y-%m-%d"));
    if !topic_names.is_empty() {
        println!("Topics:    {}", topic_names.join(", "));
    }
    Ok(())
}

pub async fn update(
    ctx: &AppContext,
    id: &str,
    state: Option<&str>,
    signal: Option<&str>,
) -> Result<()> {
    use tankyu_core::domain::types::EntryUpdate;
    if state.is_none() && signal.is_none() {
        anyhow::bail!("At least one of --state or --signal must be provided");
    }
    let uuid = Uuid::parse_str(id)
        .map_err(|_| anyhow::anyhow!("'{id}' is not a valid entry ID (expected a UUID)"))?;
    let state_val = state.map(parse_state).transpose()?;
    let signal_val = signal.map(parse_signal).transpose()?;
    let entry = ctx
        .entry_mgr
        .update(
            uuid,
            EntryUpdate {
                state: state_val,
                signal: signal_val,
                summary: None,
            },
        )
        .await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&entry)?);
        return Ok(());
    }
    println!("Updated entry: {}", entry.title);
    if let Some(s) = state {
        println!("  State: {s}");
    }
    if let Some(s) = signal {
        println!("  Signal: {s}");
    }
    Ok(())
}

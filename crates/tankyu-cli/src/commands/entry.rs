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
) -> Result<()> {
    if source.is_some() && topic.is_some() {
        anyhow::bail!("--topic and --source are mutually exclusive");
    }

    let mut entries = match (source, topic) {
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
        _ => ctx.entry_mgr.list_all().await?,
    };

    if let Some(s) = state {
        let filter = parse_state(s)?;
        entries.retain(|e| e.state == filter);
    }

    if let Some(s) = signal {
        let filter = parse_signal(s)?;
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

    let mut table = comfy_table::Table::new();
    table.set_header(["ID", "Type", "State", "Signal", "Title", "Scanned"]);
    for e in &entries {
        let id_short = &e.id.to_string()[..8];
        let title: String = if e.title.chars().count() > 60 {
            format!("{}…", e.title.chars().take(59).collect::<String>())
        } else {
            e.title.clone()
        };
        table.add_row([
            id_short,
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
    let uuid = Uuid::parse_str(id).map_err(|_| anyhow::anyhow!("Invalid UUID: {id}"))?;
    let e = ctx
        .entry_mgr
        .get(uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Entry '{id}' not found"))?;

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&e)?);
        return Ok(());
    }

    println!("ID:        {}", e.id);
    println!("Type:      {}", type_str(&e.r#type));
    println!("State:     {}", state_str(&e.state));
    println!("Signal:    {}", signal_str(e.signal.as_ref()));
    println!("Title:     {}", e.title);
    println!("URL:       {}", e.url);
    println!("Source:    {}", e.source_id);
    println!("Summary:   {}", e.summary.as_deref().unwrap_or("—"));
    println!("Scanned:   {}", e.scanned_at);
    println!("Created:   {}", e.created_at.format("%Y-%m-%d"));
    Ok(())
}

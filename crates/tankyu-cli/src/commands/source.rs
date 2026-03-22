use anyhow::Result;
use tankyu_core::domain::types::{SourceRole, SourceState, SourceType};
use tankyu_core::features::source::source_manager::AddSourceInput;

use crate::context::AppContext;

fn parse_source_type(s: &str) -> Result<SourceType> {
    match s {
        "x-account" => Ok(SourceType::XAccount),
        "x-bookmarks" => Ok(SourceType::XBookmarks),
        "github-repo" => Ok(SourceType::GithubRepo),
        "github-releases" => Ok(SourceType::GithubReleases),
        "github-user" => Ok(SourceType::GithubUser),
        "blog" => Ok(SourceType::Blog),
        "rss-feed" => Ok(SourceType::RssFeed),
        "web-page" => Ok(SourceType::WebPage),
        "manual" => Ok(SourceType::Manual),
        "github-issues" => Ok(SourceType::GithubIssues),
        "agent-report" => Ok(SourceType::AgentReport),
        _ => Err(anyhow::anyhow!(
            "Invalid type '{s}'. Valid: x-account, x-bookmarks, github-repo, github-releases, \
             github-user, blog, rss-feed, web-page, manual, github-issues, agent-report"
        )),
    }
}

fn parse_role(s: &str) -> Result<SourceRole> {
    match s {
        "starred" => Ok(SourceRole::Starred),
        "role-model" => Ok(SourceRole::RoleModel),
        "reference" => Ok(SourceRole::Reference),
        _ => Err(anyhow::anyhow!(
            "Invalid role '{s}'. Valid: starred, role-model, reference"
        )),
    }
}

const fn role_str(r: Option<&SourceRole>) -> &'static str {
    match r {
        None => "—",
        Some(SourceRole::Starred) => "starred",
        Some(SourceRole::RoleModel) => "role-model",
        Some(SourceRole::Reference) => "reference",
    }
}

const fn type_str(t: &SourceType) -> &'static str {
    match t {
        SourceType::XAccount => "x-account",
        SourceType::XBookmarks => "x-bookmarks",
        SourceType::GithubRepo => "github-repo",
        SourceType::GithubReleases => "github-releases",
        SourceType::GithubUser => "github-user",
        SourceType::Blog => "blog",
        SourceType::RssFeed => "rss-feed",
        SourceType::WebPage => "web-page",
        SourceType::Manual => "manual",
        SourceType::GithubIssues => "github-issues",
        SourceType::AgentReport => "agent-report",
    }
}

const fn state_str(s: &SourceState) -> &'static str {
    match s {
        SourceState::Active => "active",
        SourceState::Stale => "stale",
        SourceState::Dormant => "dormant",
        SourceState::Pruned => "pruned",
    }
}

pub async fn list(ctx: &AppContext, topic: Option<&str>, role: Option<&str>) -> Result<()> {
    let sources = match (topic, role) {
        (Some(t), _) => {
            let topic = ctx
                .topic_mgr
                .get_by_name(t)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Topic '{t}' not found"))?;
            ctx.source_mgr.list_by_topic(topic.id).await?
        }
        (None, Some(r)) => ctx.source_mgr.list_by_role(parse_role(r)?).await?,
        (None, None) => ctx.source_mgr.list_all().await?,
    };
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&sources)?);
        return Ok(());
    }
    if sources.is_empty() {
        let hint = match (topic, role) {
            (Some(t), _) => {
                format!("No sources for \"{t}\". Add one with: tankyu source add <url> --topic {t}")
            }
            (None, Some(r)) => format!("No {r} sources."),
            (None, None) => "No sources yet. Add one with: tankyu source add <url>".to_string(),
        };
        println!("{hint}");
        return Ok(());
    }
    let mut table = comfy_table::Table::new();
    table.set_header(["Name", "Type", "State", "Role"]);
    for s in &sources {
        table.add_row([
            s.name.as_str(),
            type_str(&s.r#type),
            state_str(&s.state),
            role_str(s.role.as_ref()),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub async fn inspect(ctx: &AppContext, name: &str) -> Result<()> {
    let s = ctx
        .source_mgr
        .get_by_name(name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Source '{name}' not found"))?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&s)?);
        return Ok(());
    }
    println!("Name:         {}", s.name);
    println!("Type:         {}", type_str(&s.r#type));
    println!("State:        {}", state_str(&s.state));
    println!("Role:         {}", role_str(s.role.as_ref()));
    println!("URL:          {}", s.url);
    println!("Check count:  {}", s.check_count);
    println!("Hit count:    {}", s.hit_count);
    println!("Miss count:   {}", s.miss_count);
    let last_checked = s.last_checked_at.map_or_else(
        || "never".into(),
        |t| t.format("%Y-%m-%d %H:%M").to_string(),
    );
    println!("Last checked: {last_checked}");
    println!("Created:      {}", s.created_at.format("%Y-%m-%d"));

    // Show related topics (via Monitors edges pointing to this source).
    let topics = ctx.topic_mgr.list_by_source(s.id).await?;
    if !topics.is_empty() {
        let names: Vec<_> = topics.iter().map(|t| t.name.as_str()).collect();
        println!("Topics:       {}", names.join(", "));
    }
    Ok(())
}

pub async fn add(
    ctx: &AppContext,
    url: &str,
    name: Option<&str>,
    topic: Option<&str>,
    role: Option<&str>,
    source_type: Option<&str>,
) -> Result<()> {
    let role = role.map(parse_role).transpose()?;
    let source_type = source_type.map(parse_source_type).transpose()?;
    let topic_id = if let Some(t) = topic {
        Some(
            ctx.topic_mgr
                .get_by_name(t)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Topic '{t}' not found"))?
                .id,
        )
    } else {
        None
    };
    // Check if this URL is already tracked (for idempotent messaging).
    let was_known = ctx.source_mgr.get_by_url(url).await?.is_some();

    let source = ctx
        .source_mgr
        .add(AddSourceInput {
            url: url.to_string(),
            name: name.map(str::to_string),
            source_type,
            role,
            topic_id,
        })
        .await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&source)?);
        return Ok(());
    }
    if was_known {
        println!(
            "Source {} already exists ({})",
            source.name,
            type_str(&source.r#type)
        );
    } else {
        println!(
            "Added source: {} ({})",
            source.name,
            type_str(&source.r#type)
        );
    }
    println!("  URL: {}", source.url);
    println!("  ID:  {}", source.id);
    if let Some(t) = topic {
        println!("  Linked to topic: {t}");
    }
    Ok(())
}

pub async fn remove(ctx: &AppContext, name: &str) -> Result<()> {
    let source = ctx.source_mgr.remove(name).await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&source)?);
        return Ok(());
    }
    println!("Removed source: {} (marked as pruned)", source.name);
    Ok(())
}

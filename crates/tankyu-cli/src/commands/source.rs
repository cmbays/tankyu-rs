use anyhow::Result;
use tankyu_core::domain::types::{SourceRole, SourceState, SourceType};

use crate::context::AppContext;

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

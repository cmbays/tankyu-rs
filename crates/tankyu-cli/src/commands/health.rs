use anyhow::Result;
use tankyu_core::{
    domain::types::SourceType,
    features::health::{HealthThresholds, HealthWarningKind},
};

use crate::context::AppContext;

const fn source_type_str(t: &SourceType) -> &'static str {
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

pub async fn run(ctx: &AppContext) -> Result<()> {
    let thresholds = HealthThresholds {
        stale_days: ctx.config.stale_days,
        dormant_days: ctx.config.dormant_days,
    };
    let report = ctx.health_mgr.health(thresholds).await?;
    let healthy = report.ok;

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&report)?);
    } else if healthy {
        println!("All sources healthy");
    } else {
        let mut table = comfy_table::Table::new();
        table.set_header(["Kind", "Source", "Type", "Detail"]);
        for w in &report.warnings {
            let kind = match w.kind {
                HealthWarningKind::Stale => "stale",
                HealthWarningKind::Dormant => "dormant",
                HealthWarningKind::Empty => "empty",
            };
            table.add_row([
                kind,
                &w.source_name,
                source_type_str(&w.source_type),
                &w.detail,
            ]);
        }
        println!("{table}");
    }

    if !healthy {
        anyhow::bail!("health check failed: {} warning(s)", report.warnings.len());
    }
    Ok(())
}

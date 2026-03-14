use anyhow::Result;

use crate::context::AppContext;

pub async fn run(ctx: &AppContext) -> Result<()> {
    let mut issues: Vec<String> = Vec::new();

    if !ctx.data_dir.exists() {
        issues.push(format!(
            "Data directory not found: {}",
            ctx.data_dir.display()
        ));
    }
    if ctx.config.version != 1 {
        issues.push(format!(
            "Unexpected config version: {} (expected 1)",
            ctx.config.version
        ));
    }
    let entry_count = ctx.entry_mgr.list_all().await?.len();

    if ctx.output.is_json() {
        println!(
            "{}",
            serde_json::json!({
                "ok": issues.is_empty(),
                "issues": issues,
                "data_dir": ctx.data_dir.to_string_lossy(),
                "entry_count": entry_count
            })
        );
        return Ok(());
    }

    if issues.is_empty() {
        println!("✓ All checks passed");
        println!("  Data dir: {}", ctx.data_dir.display());
        println!("  Entries:  {entry_count}");
    } else {
        for issue in &issues {
            eprintln!("  ✗ {issue}");
        }
        anyhow::bail!("{} issue(s) found", issues.len());
    }
    Ok(())
}

use anyhow::Result;

use crate::context::AppContext;

pub fn show(ctx: &AppContext) -> Result<()> {
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string_pretty(&ctx.config)?);
        return Ok(());
    }
    println!("Version:            {}", ctx.config.version);
    println!("Default scan limit: {}", ctx.config.default_scan_limit);
    println!("Stale days:         {}", ctx.config.stale_days);
    println!("Dormant days:       {}", ctx.config.dormant_days);
    println!("LLM classify:       {}", ctx.config.llm_classify);
    if let Some(reg) = &ctx.config.registry_path {
        println!("Registry path:      {reg}");
    }
    Ok(())
}

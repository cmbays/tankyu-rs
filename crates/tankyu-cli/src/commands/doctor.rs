use std::path::PathBuf;

use anyhow::Result;

use crate::output::OutputMode;

/// Doctor runs independently — it must not require a fully initialized `AppContext`
/// because its job is to diagnose broken setups (missing config, missing DB, etc.).
pub async fn run_standalone(data_dir: PathBuf, output: OutputMode) -> Result<()> {
    let mut issues: Vec<String> = Vec::new();
    let mut db_healthy = false;
    let mut db_warnings: Vec<String> = Vec::new();
    let mut datasets_checked: usize = 0;

    // Check config
    let config_ok = {
        let config_path = tankyu_core::shared::constants::config_path(&data_dir);
        if config_path.exists() {
            match tokio::fs::read(&config_path).await {
                Ok(bytes) => match serde_json::from_slice::<tankyu_core::domain::types::TankyuConfig>(
                    &bytes,
                ) {
                    Ok(cfg) if cfg.version == 1 => true,
                    Ok(cfg) => {
                        issues.push(format!(
                            "Unexpected config version: {} (expected 1)",
                            cfg.version
                        ));
                        false
                    }
                    Err(e) => {
                        issues.push(format!("Config parse error: {e}"));
                        false
                    }
                },
                Err(e) => {
                    issues.push(format!("Config read error: {e}"));
                    false
                }
            }
        } else {
            issues.push("Config: not found".to_string());
            false
        }
    };

    // Check nanograph database
    let db_path = tankyu_core::shared::constants::db_path(&data_dir);
    if db_path.join("schema.ir.json").exists() {
        match tankyu_core::NanographStore::open(&db_path).await {
            Ok(store) => {
                use tankyu_core::IResearchGraph;
                match store.doctor().await {
                    Ok(report) => {
                        db_healthy = report.healthy;
                        db_warnings = report.warnings;
                        datasets_checked = report.datasets_checked;
                        if !report.healthy {
                            for issue in &report.issues {
                                issues.push(format!("Database issue: {issue}"));
                            }
                        }
                    }
                    Err(e) => {
                        issues.push(format!("Database doctor error: {e}"));
                    }
                }
            }
            Err(e) => {
                issues.push(format!("Database open error: {e}"));
            }
        }
    } else {
        issues.push("Database: not initialized".to_string());
    }

    if output.is_json() {
        println!(
            "{}",
            serde_json::json!({
                "ok": issues.is_empty(),
                "issues": issues,
                "warnings": db_warnings,
                "data_dir": data_dir.to_string_lossy(),
                "datasets_checked": datasets_checked
            })
        );
        if !issues.is_empty() {
            anyhow::bail!("{} issue(s) found", issues.len());
        }
        return Ok(());
    }

    // Text output
    if db_healthy {
        println!("  Database: OK");
    } else {
        println!("  Database: not initialized");
    }

    if config_ok {
        println!("  Config: OK");
    } else {
        println!("  Config: not found");
    }

    for w in &db_warnings {
        println!("  ⚠ {w}");
    }

    if issues.is_empty() {
        println!("  Data dir: {}", data_dir.display());
    } else {
        anyhow::bail!("{} issue(s) found", issues.len());
    }
    Ok(())
}

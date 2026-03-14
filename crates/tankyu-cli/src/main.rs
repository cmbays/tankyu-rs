#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod context;
mod error;
mod output;

use cli::{Cli, Commands, ConfigCommands, EntryCommands, SourceCommands, TopicCommands};
use context::AppContext;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = AppContext::new(cli.tankyu_dir, cli.json).await?;
    match cli.command {
        Commands::Status => commands::status::run(&ctx).await,
        Commands::Topic { command } => match command {
            TopicCommands::List => commands::topic::list(&ctx).await,
            TopicCommands::Inspect { name } => commands::topic::inspect(&ctx, &name).await,
        },
        Commands::Source { command } => match command {
            SourceCommands::List { topic, role } => {
                commands::source::list(&ctx, topic.as_deref(), role.as_deref()).await
            }
        },
        Commands::Entry { command } => match command {
            EntryCommands::List {
                state,
                signal,
                source,
                topic,
                limit,
            } => {
                commands::entry::list(
                    &ctx,
                    state.as_deref(),
                    signal.as_deref(),
                    source.as_deref(),
                    topic.as_deref(),
                    limit,
                )
                .await
            }
            EntryCommands::Inspect { id } => commands::entry::inspect(&ctx, &id).await,
        },
        Commands::Config { command } => match command {
            ConfigCommands::Show => commands::config::show(&ctx),
        },
        Commands::Doctor => commands::doctor::run(&ctx).await,
    }
}

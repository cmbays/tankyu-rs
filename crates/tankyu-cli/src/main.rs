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
            TopicCommands::Create {
                name,
                description,
                tags,
            } => commands::topic::create(&ctx, &name, &description, &tags).await,
        },
        Commands::Source { command } => match command {
            SourceCommands::List { topic, role } => {
                commands::source::list(&ctx, topic.as_deref(), role.as_deref()).await
            }
            SourceCommands::Inspect { name } => commands::source::inspect(&ctx, &name).await,
            SourceCommands::Add {
                url,
                name,
                topic,
                role,
                source_type,
            } => {
                commands::source::add(
                    &ctx,
                    &url,
                    name.as_deref(),
                    topic.as_deref(),
                    role.as_deref(),
                    source_type.as_deref(),
                )
                .await
            }
            SourceCommands::Remove { name } => commands::source::remove(&ctx, &name).await,
        },
        Commands::Entry { command } => match command {
            EntryCommands::List {
                state,
                signal,
                source,
                topic,
                limit,
                unclassified,
            } => {
                commands::entry::list(
                    &ctx,
                    state.as_deref(),
                    signal.as_deref(),
                    source.as_deref(),
                    topic.as_deref(),
                    limit,
                    unclassified,
                )
                .await
            }
            EntryCommands::Inspect { id } => commands::entry::inspect(&ctx, &id).await,
            EntryCommands::Update { id, state, signal } => {
                commands::entry::update(&ctx, &id, state.as_deref(), signal.as_deref()).await
            }
        },
        Commands::Config { command } => match command {
            ConfigCommands::Show => commands::config::show(&ctx),
        },
        Commands::Doctor => commands::doctor::run(&ctx).await,
        Commands::Health => commands::health::run(&ctx).await,
    }
}

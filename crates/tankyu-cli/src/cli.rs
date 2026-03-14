use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tankyu", version, about = "Research intelligence graph")]
pub struct Cli {
    #[arg(long, global = true, help = "Override data directory")]
    pub tankyu_dir: Option<PathBuf>,

    #[arg(long, global = true, help = "Output as JSON")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Dashboard with source, topic, and entry counts
    Status,
    /// Topic management
    Topic {
        #[command(subcommand)]
        command: TopicCommands,
    },
    /// Source management
    Source {
        #[command(subcommand)]
        command: SourceCommands,
    },
    /// Entry management
    Entry {
        #[command(subcommand)]
        command: EntryCommands,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Run diagnostics on the data directory
    Doctor,
}

#[derive(Subcommand)]
pub enum TopicCommands {
    /// List all topics
    List,
    /// Inspect a topic by name
    Inspect { name: String },
}

#[derive(Subcommand)]
pub enum SourceCommands {
    /// List sources, optionally filtered by topic or role
    List {
        /// Filter by topic name
        #[arg(long)]
        topic: Option<String>,
        /// Filter by role: starred, role-model, reference
        #[arg(long)]
        role: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum EntryCommands {
    /// List entries, optionally filtered by state, signal, source, or topic
    List {
        /// Filter by state: new, scanned, triaged, read, archived
        #[arg(long)]
        state: Option<String>,
        /// Filter by signal: high, medium, low, noise
        #[arg(long)]
        signal: Option<String>,
        /// Filter by source name
        #[arg(long)]
        source: Option<String>,
        /// Filter by topic name (resolves to sources monitored by that topic)
        #[arg(long)]
        topic: Option<String>,
        /// Limit number of results (applied after all filters)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Inspect a single entry by UUID
    Inspect {
        /// Entry UUID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Print the current configuration
    Show,
}

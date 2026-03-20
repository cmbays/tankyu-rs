use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use tankyu_core::{
    domain::{
        ports::{IEntryStore, IGraphStore, ISourceStore, ITopicStore},
        research_graph::IResearchGraph,
        types::TankyuConfig,
    },
    features::{
        entry::EntryManager, source::SourceManager, status::StatusUseCase, topic::TopicManager,
    },
    infrastructure::{
        graph::JsonGraphStore,
        stores::{EntryStore, SourceStore, TopicStore},
    },
    shared::constants,
    HealthManager, NanographStore,
};

use crate::output::OutputMode;

pub struct AppContext {
    pub topic_mgr: TopicManager,
    pub source_mgr: SourceManager,
    pub entry_mgr: EntryManager,
    pub health_mgr: HealthManager,
    pub graph_store: Arc<dyn IGraphStore>,
    pub status_uc: StatusUseCase,
    pub config: TankyuConfig,
    pub output: OutputMode,
}

impl AppContext {
    pub async fn new(tankyu_dir_arg: Option<PathBuf>, json: bool) -> Result<Self> {
        let base = tankyu_dir_arg.unwrap_or_else(constants::tankyu_dir);

        let config_path = constants::config_path(&base);
        let config_bytes = tokio::fs::read(&config_path)
            .await
            .with_context(|| format!("Cannot read config: {}", config_path.display()))?;
        let config: TankyuConfig = serde_json::from_slice(&config_bytes)
            .with_context(|| format!("Cannot parse config: {}", config_path.display()))?;

        let topic_store: Arc<dyn ITopicStore> =
            Arc::new(TopicStore::new(constants::topics_dir(&base)));
        let source_store: Arc<dyn ISourceStore> =
            Arc::new(SourceStore::new(constants::sources_dir(&base)));
        let graph_store: Arc<dyn IGraphStore> =
            Arc::new(JsonGraphStore::new(constants::edges_path(&base)));
        let entry_store: Arc<dyn IEntryStore> =
            Arc::new(EntryStore::new(constants::entries_dir(&base)));

        let health_source_store = Arc::clone(&source_store);
        let health_entry_store = Arc::clone(&entry_store);

        // Initialize nanograph — auto-creates the DB directory on first run
        let db_path = constants::db_path(&base);
        let graph: Arc<dyn IResearchGraph> = Arc::new(
            NanographStore::open(&db_path)
                .await
                .with_context(|| format!("Cannot open research graph at {}", db_path.display()))?,
        );

        Ok(Self {
            topic_mgr: TopicManager::new(topic_store),
            source_mgr: SourceManager::new(source_store, Arc::clone(&graph_store)),
            entry_mgr: EntryManager::new(entry_store, Arc::clone(&graph_store)),
            health_mgr: HealthManager::new(health_source_store, health_entry_store),
            graph_store: Arc::clone(&graph_store),
            status_uc: StatusUseCase::new(Arc::clone(&graph)),
            config,
            output: OutputMode::detect(json),
        })
    }
}

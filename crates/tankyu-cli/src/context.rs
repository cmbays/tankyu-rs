use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use tankyu_core::{
    domain::{
        ports::{IGraphStore, ISourceStore, ITopicStore},
        types::TankyuConfig,
    },
    features::{source::SourceManager, topic::TopicManager},
    infrastructure::{
        graph::JsonGraphStore,
        stores::{EntryStore, SourceStore, TopicStore},
    },
    shared::constants,
};

use crate::output::OutputMode;

pub struct AppContext {
    pub topic_mgr: TopicManager,
    pub source_mgr: SourceManager,
    pub entry_store: Arc<EntryStore>,
    pub config: TankyuConfig,
    pub output: OutputMode,
    pub data_dir: PathBuf,
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
        let entry_store = Arc::new(EntryStore::new(constants::entries_dir(&base)));

        Ok(Self {
            topic_mgr: TopicManager::new(topic_store),
            source_mgr: SourceManager::new(source_store, graph_store),
            entry_store,
            config,
            output: OutputMode::detect(json),
            data_dir: base,
        })
    }
}

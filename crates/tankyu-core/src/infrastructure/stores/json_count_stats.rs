use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::ports::{IEntryStore, ISourceStore, ITopicStore};
use crate::features::status::CountStats;
use crate::shared::error::TankyuError;

/// [`CountStats`] implementation that counts records from JSON file stores.
pub struct JsonCountStats {
    topics: Arc<dyn ITopicStore>,
    sources: Arc<dyn ISourceStore>,
    entries: Arc<dyn IEntryStore>,
}

impl JsonCountStats {
    #[must_use]
    pub fn new(
        topics: Arc<dyn ITopicStore>,
        sources: Arc<dyn ISourceStore>,
        entries: Arc<dyn IEntryStore>,
    ) -> Self {
        Self {
            topics,
            sources,
            entries,
        }
    }
}

#[async_trait]
impl CountStats for JsonCountStats {
    async fn count_topics(&self) -> Result<usize, TankyuError> {
        self.topics
            .list()
            .await
            .map(|v| v.len())
            .map_err(|e| TankyuError::Store(e.to_string()))
    }

    async fn count_sources(&self) -> Result<usize, TankyuError> {
        self.sources
            .list()
            .await
            .map(|v| v.len())
            .map_err(|e| TankyuError::Store(e.to_string()))
    }

    async fn count_entries(&self) -> Result<usize, TankyuError> {
        self.entries
            .list()
            .await
            .map(|v| v.len())
            .map_err(|e| TankyuError::Store(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ports::{IEntryStore, ISourceStore, ITopicStore},
        types::{
            Entry, EntryState, EntryType, EntryUpdate, Source, SourceState, SourceType,
            SourceUpdate, Topic, TopicUpdate,
        },
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    struct OkTopicStore(usize);
    struct OkSourceStore(usize);
    struct OkEntryStore(usize);
    struct FailingTopicStore;
    struct FailingSourceStore;
    struct FailingEntryStore;

    fn make_topic() -> Topic {
        Topic {
            id: Uuid::new_v4(),
            name: "t".to_string(),
            description: String::new(),
            tags: vec![],
            projects: vec![],
            routing: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_scanned_at: None,
            scan_count: 0,
        }
    }

    fn make_source() -> Source {
        Source {
            id: Uuid::new_v4(),
            r#type: SourceType::GithubRepo,
            role: None,
            name: "s".to_string(),
            url: "https://example.com".to_string(),
            config: None,
            state: SourceState::Active,
            poll_interval_minutes: None,
            discovered_via: None,
            discovery_reason: None,
            last_checked_at: None,
            last_new_content_at: None,
            check_count: 0,
            hit_count: 0,
            miss_count: 0,
            created_at: Utc::now(),
        }
    }

    fn make_entry() -> Entry {
        Entry {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            r#type: EntryType::Article,
            title: "e".to_string(),
            url: "https://example.com/e".to_string(),
            summary: None,
            content_hash: None,
            state: EntryState::New,
            signal: None,
            scanned_at: Utc::now(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    #[async_trait]
    impl ITopicStore for OkTopicStore {
        async fn create(&self, _t: Topic) -> Result<()> { Ok(()) }
        async fn get(&self, _id: Uuid) -> Result<Option<Topic>> { Ok(None) }
        async fn get_by_name(&self, _n: &str) -> Result<Option<Topic>> { Ok(None) }
        async fn list(&self) -> Result<Vec<Topic>> {
            Ok((0..self.0).map(|_| make_topic()).collect())
        }
        async fn update(&self, _id: Uuid, _u: TopicUpdate) -> Result<Topic> { unimplemented!() }
    }

    #[async_trait]
    impl ISourceStore for OkSourceStore {
        async fn create(&self, _s: Source) -> Result<()> { Ok(()) }
        async fn get(&self, _id: Uuid) -> Result<Option<Source>> { Ok(None) }
        async fn get_by_url(&self, _u: &str) -> Result<Option<Source>> { Ok(None) }
        async fn list(&self) -> Result<Vec<Source>> {
            Ok((0..self.0).map(|_| make_source()).collect())
        }
        async fn update(&self, _id: Uuid, _u: SourceUpdate) -> Result<Source> { unimplemented!() }
    }

    #[async_trait]
    impl IEntryStore for OkEntryStore {
        async fn create(&self, _e: Entry) -> Result<()> { Ok(()) }
        async fn get(&self, _id: Uuid) -> Result<Option<Entry>> { Ok(None) }
        async fn get_by_url(&self, _u: &str) -> Result<Option<Entry>> { Ok(None) }
        async fn get_by_content_hash(&self, _h: &str) -> Result<Option<Entry>> { Ok(None) }
        async fn list_by_source(&self, _id: Uuid) -> Result<Vec<Entry>> { Ok(vec![]) }
        async fn list(&self) -> Result<Vec<Entry>> {
            Ok((0..self.0).map(|_| make_entry()).collect())
        }
        async fn update(&self, _id: Uuid, _u: EntryUpdate) -> Result<Entry> { unimplemented!() }
    }

    #[async_trait]
    impl ITopicStore for FailingTopicStore {
        async fn create(&self, _t: Topic) -> Result<()> { Ok(()) }
        async fn get(&self, _id: Uuid) -> Result<Option<Topic>> { Ok(None) }
        async fn get_by_name(&self, _n: &str) -> Result<Option<Topic>> { Ok(None) }
        async fn list(&self) -> Result<Vec<Topic>> {
            Err(anyhow::anyhow!("disk read failed"))
        }
        async fn update(&self, _id: Uuid, _u: TopicUpdate) -> Result<Topic> { unimplemented!() }
    }

    #[async_trait]
    impl ISourceStore for FailingSourceStore {
        async fn create(&self, _s: Source) -> Result<()> { Ok(()) }
        async fn get(&self, _id: Uuid) -> Result<Option<Source>> { Ok(None) }
        async fn get_by_url(&self, _u: &str) -> Result<Option<Source>> { Ok(None) }
        async fn list(&self) -> Result<Vec<Source>> {
            Err(anyhow::anyhow!("disk read failed"))
        }
        async fn update(&self, _id: Uuid, _u: SourceUpdate) -> Result<Source> { unimplemented!() }
    }

    #[async_trait]
    impl IEntryStore for FailingEntryStore {
        async fn create(&self, _e: Entry) -> Result<()> { Ok(()) }
        async fn get(&self, _id: Uuid) -> Result<Option<Entry>> { Ok(None) }
        async fn get_by_url(&self, _u: &str) -> Result<Option<Entry>> { Ok(None) }
        async fn get_by_content_hash(&self, _h: &str) -> Result<Option<Entry>> { Ok(None) }
        async fn list_by_source(&self, _id: Uuid) -> Result<Vec<Entry>> { Ok(vec![]) }
        async fn list(&self) -> Result<Vec<Entry>> {
            Err(anyhow::anyhow!("disk read failed"))
        }
        async fn update(&self, _id: Uuid, _u: EntryUpdate) -> Result<Entry> { unimplemented!() }
    }

    #[tokio::test]
    async fn counts_return_correct_values() {
        let stats = JsonCountStats::new(
            Arc::new(OkTopicStore(3)),
            Arc::new(OkSourceStore(2)),
            Arc::new(OkEntryStore(5)),
        );
        assert_eq!(stats.count_topics().await.unwrap(), 3);
        assert_eq!(stats.count_sources().await.unwrap(), 2);
        assert_eq!(stats.count_entries().await.unwrap(), 5);
    }

    #[tokio::test]
    async fn counts_return_zero_on_empty() {
        let stats = JsonCountStats::new(
            Arc::new(OkTopicStore(0)),
            Arc::new(OkSourceStore(0)),
            Arc::new(OkEntryStore(0)),
        );
        assert_eq!(stats.count_topics().await.unwrap(), 0);
        assert_eq!(stats.count_sources().await.unwrap(), 0);
        assert_eq!(stats.count_entries().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn topic_store_error_propagates_as_tankyu_error() {
        let stats = JsonCountStats::new(
            Arc::new(FailingTopicStore),
            Arc::new(OkSourceStore(0)),
            Arc::new(OkEntryStore(0)),
        );
        let err = stats.count_topics().await.unwrap_err();
        assert!(matches!(err, TankyuError::Store(_)));
    }

    #[tokio::test]
    async fn source_store_error_propagates_as_tankyu_error() {
        let stats = JsonCountStats::new(
            Arc::new(OkTopicStore(0)),
            Arc::new(FailingSourceStore),
            Arc::new(OkEntryStore(0)),
        );
        let err = stats.count_sources().await.unwrap_err();
        assert!(matches!(err, TankyuError::Store(_)));
    }

    #[tokio::test]
    async fn entry_store_error_propagates_as_tankyu_error() {
        let stats = JsonCountStats::new(
            Arc::new(OkTopicStore(0)),
            Arc::new(OkSourceStore(0)),
            Arc::new(FailingEntryStore),
        );
        let err = stats.count_entries().await.unwrap_err();
        assert!(matches!(err, TankyuError::Store(_)));
    }
}

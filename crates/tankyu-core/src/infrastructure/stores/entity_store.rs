use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    domain::{ports::IEntityStore, types::Entity},
    infrastructure::persistence::JsonStore,
    shared::error::TankyuError,
};

/// JSON-backed store for [`Entity`] records.
pub struct EntityStore {
    store: JsonStore<Entity>,
}

impl EntityStore {
    /// Create a new `EntityStore` rooted at `dir`.
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self {
            store: JsonStore::new(dir),
        }
    }
}

#[async_trait]
impl IEntityStore for EntityStore {
    async fn create(&self, entity: Entity) -> Result<()> {
        self.store
            .write(&entity.id.to_string(), &entity)
            .await
            .context("failed to write entity")
    }

    async fn get(&self, id: Uuid) -> Result<Option<Entity>> {
        match self.store.read(&id.to_string()).await {
            Ok(e) => Ok(Some(e)),
            Err(TankyuError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Entity>> {
        let all = self.store.read_all().await?;
        Ok(all.into_iter().find(|e| e.name == name))
    }

    async fn list(&self) -> Result<Vec<Entity>> {
        self.store
            .read_all()
            .await
            .context("failed to list entities")
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::domain::types::EntityType;
    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn make_entity(name: &str) -> Entity {
        Entity {
            id: Uuid::new_v4(),
            r#type: EntityType::Technology,
            name: name.to_string(),
            aliases: vec![],
            url: None,
            description: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_then_get_by_name() {
        let dir = tempdir().unwrap();
        let store = EntityStore::new(dir.path().to_path_buf());
        let entity = make_entity("Rust");
        store.create(entity.clone()).await.unwrap();
        let found = store.get_by_name("Rust").await.unwrap();
        assert_eq!(found.unwrap().id, entity.id);
    }

    #[tokio::test]
    async fn list_returns_all() {
        let dir = tempdir().unwrap();
        let store = EntityStore::new(dir.path().to_path_buf());
        store.create(make_entity("Rust")).await.unwrap();
        store.create(make_entity("Tokio")).await.unwrap();
        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }
}

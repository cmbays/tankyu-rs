use assert_cmd::Command;
use cucumber::World;
use std::path::Path;
use tempfile::TempDir;

fn write_json(path: impl AsRef<Path>, value: &serde_json::Value) {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

/// Shared state carried through all steps of one cucumber scenario.
#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TankyuWorld {
    pub data_dir: TempDir,
    pub last_stdout: String,
    pub last_stderr: String,
    pub last_exit_code: Option<i32>,
}

impl TankyuWorld {
    #[allow(clippy::unused_async)]
    async fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let b = dir.path();
        for sub in &["topics", "sources", "entries", "graph"] {
            std::fs::create_dir_all(b.join(sub)).unwrap();
        }
        write_json(
            b.join("config.json"),
            &serde_json::json!({
                "version": 1, "defaultScanLimit": 20, "staleDays": 7,
                "dormantDays": 30, "llmClassify": false, "localRepoPaths": {}
            }),
        );
        write_json(
            b.join("graph/edges.json"),
            &serde_json::json!({ "version": 1, "edges": [] }),
        );
        Self {
            data_dir: dir,
            last_stdout: String::new(),
            last_stderr: String::new(),
            last_exit_code: None,
        }
    }

    pub fn run_cmd(&mut self, args: &[&str]) {
        let mut c = Command::cargo_bin("tankyu").unwrap();
        c.env("TANKYU_DIR", self.data_dir.path());
        for arg in args {
            c.arg(arg);
        }
        let out = c.output().unwrap();
        self.last_stdout = String::from_utf8_lossy(&out.stdout).to_string();
        self.last_stderr = String::from_utf8_lossy(&out.stderr).to_string();
        self.last_exit_code = out.status.code();
    }

    pub fn write_entry(&self, id: &str, title: &str, state: &str, signal: Option<&str>) {
        write_json(
            self.data_dir.path().join(format!("entries/{id}.json")),
            &serde_json::json!({
                "id": id,
                "sourceId": "22222222-2222-2222-2222-222222222222",
                "type": "article",
                "title": title,
                "url": format!("https://example.com/{id}"),
                "summary": null,
                "contentHash": null,
                "state": state,
                "signal": signal,
                "scannedAt": "2025-01-15T10:00:00Z",
                "metadata": null,
                "createdAt": "2025-01-15T10:00:00Z"
            }),
        );
    }

    pub fn write_source(
        &self,
        id: &str,
        name: &str,
        url: &str,
        state: &str,
        last_checked_days_ago: Option<i64>,
    ) {
        use chrono::{Duration, Utc};
        let last_checked = last_checked_days_ago
            .map(|d| (Utc::now() - Duration::days(d)).to_rfc3339())
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null);
        write_json(
            self.data_dir.path().join(format!("sources/{id}.json")),
            &serde_json::json!({
                "id": id,
                "type": "github-repo",
                "name": name,
                "url": url,
                "state": state,
                "config": null,
                "pollIntervalMinutes": null,
                "discoveredVia": null,
                "discoveryReason": null,
                "lastCheckedAt": last_checked,
                "lastNewContentAt": null,
                "checkCount": 0,
                "hitCount": 0,
                "missCount": 0,
                "createdAt": "2025-01-01T00:00:00Z"
            }),
        );
    }

    pub fn write_topic(&self, id: &str, name: &str) {
        write_json(
            self.data_dir.path().join(format!("topics/{id}.json")),
            &serde_json::json!({
                "id": id,
                "name": name,
                "description": "",
                "tags": [],
                "projects": [],
                "createdAt": "2025-01-01T00:00:00Z",
                "updatedAt": "2025-01-01T00:00:00Z",
                "lastScannedAt": null,
                "scanCount": 0
            }),
        );
    }

    pub fn write_tagged_with_edge(&self, entry_id: &str, topic_id: &str) {
        use uuid::Uuid;
        let edge_id = Uuid::new_v4().to_string();
        let edges_path = self.data_dir.path().join("graph/edges.json");
        let current: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&edges_path).unwrap()).unwrap();
        let mut edges = current["edges"].as_array().cloned().unwrap_or_default();
        edges.push(serde_json::json!({
            "id": edge_id,
            "fromId": entry_id,
            "fromType": "entry",
            "toId": topic_id,
            "toType": "topic",
            "edgeType": "tagged-with",
            "reason": "test classification",
            "createdAt": "2025-01-01T00:00:00Z"
        }));
        write_json(
            &edges_path,
            &serde_json::json!({ "version": 1, "edges": edges }),
        );
    }

    pub fn write_entry_for_source(&self, source_id: &str) {
        let id = uuid::Uuid::new_v4().to_string();
        write_json(
            self.data_dir.path().join(format!("entries/{id}.json")),
            &serde_json::json!({
                "id": id,
                "sourceId": source_id,
                "type": "article",
                "title": "test entry",
                "url": format!("https://example.com/{id}"),
                "summary": null,
                "contentHash": null,
                "state": "new",
                "signal": null,
                "scannedAt": "2025-01-15T10:00:00Z",
                "metadata": null,
                "createdAt": "2025-01-15T10:00:00Z"
            }),
        );
    }
}

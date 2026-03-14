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
}

// ============================================================
// ⏳ Tao Version History — 时间旅行系统
// ============================================================
// 用法:
//   tao history <key>     — 查看版本历史
//   tao rollback <key> <v> — 回滚到指定版本
//   tao diff <key> v1 v2  — 比较两个版本差异
// ============================================================

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 版本记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    pub version: u64,
    pub key: String,
    pub value: String,
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub operation: String, // "put", "update", "delete"
}

/// 版本存储
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionStore {
    pub versions: HashMap<String, Vec<VersionRecord>>,
}

impl VersionStore {
    pub fn load(path: &PathBuf) -> Self {
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(store) = serde_json::from_str(&data) {
                    return store;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn add_version(&mut self, key: &str, value: &str, tags: &[String], op: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = self.versions.entry(key.to_string()).or_default();
        let version = entry.len() as u64 + 1;

        entry.push(VersionRecord {
            version,
            key: key.to_string(),
            value: value.to_string(),
            tags: tags.to_vec(),
            timestamp: now,
            operation: op.to_string(),
        });

        // 最多保留 100 个版本
        if entry.len() > 100 {
            entry.remove(0);
        }
    }

    pub fn get_history(&self, key: &str) -> Vec<&VersionRecord> {
        self.versions.get(key).map(|v| v.iter().collect()).unwrap_or_default()
    }

    pub fn get_version(&self, key: &str, version: u64) -> Option<&VersionRecord> {
        self.versions.get(key)?.iter().find(|v| v.version == version)
    }
}

pub fn cmd_record_version(data_dir: &PathBuf, key: &str, value: &str, tags: &[String], op: &str) {
    let history_path = data_dir.join("history.json");
    let mut store = VersionStore::load(&history_path);
    store.add_version(key, value, tags, op);
    store.save(&history_path);
}

pub fn cmd_history(data_dir: &PathBuf, key: &str) {
    let history_path = data_dir.join("history.json");
    let store = VersionStore::load(&history_path);
    let history = store.get_history(key);

    if history.is_empty() {
        println!("📭 '{}' 无版本历史", key);
        return;
    }

    println!("\n⏳ 版本历史: {}", key);
    println!("{}", "─".repeat(60));
    for record in &history {
        let ts = chrono::DateTime::from_timestamp(record.timestamp as i64, 0)
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".into());
        let op_icon = match record.operation.as_str() {
            "put" => "📝",
            "update" => "✏️",
            "delete" => "🗑️",
            _ => "❓",
        };
        let snippet = if record.value.len() > 50 {
            format!("{}...", &record.value[..50])
        } else {
            record.value.clone()
        };
        println!("  {} v{}  {}  {}  {}", op_icon, record.version, ts, snippet, record.tags.join(","));
    }
    println!("{}", "─".repeat(60));
}

pub fn cmd_rollback(data_dir: &PathBuf, key: &str, version: u64) {
    let history_path = data_dir.join("history.json");
    let mut store = VersionStore::load(&history_path);

    let record = match store.get_version(key, version) {
        Some(r) => r.clone(),
        None => {
            println!("❌ 版本 v{} 不存在", version);
            return;
        }
    };

    // 使用 sled 恢复数据
    let store_path = data_dir.join("store");
    let db = match sled::open(&store_path) {
        Ok(d) => d,
        Err(e) => { println!("❌ 无法打开存储: {}", e); return; }
    };

    let payload = record.value.as_bytes().to_vec();
    let owner = [0u8; 32];
    let mut unit = tao_core::DataUnit::new(payload, key.to_string(), owner);
    unit.yang.tags = record.tags.clone();

    if let Ok(bytes) = bincode::serialize(&unit) {
        if db.insert(&unit.yin.content_hash, bytes).is_ok() {
            let _ = db.flush();
            println!("✅ 已回滚到 v{}: {}", version, key);
        }
    }

    // 记录回滚操作
    store.add_version(key, &record.value, &record.tags, "rollback");
    store.save(&history_path);
}

pub fn cmd_diff(data_dir: &PathBuf, key: &str, v1: u64, v2: u64) {
    let history_path = data_dir.join("history.json");
    let store = VersionStore::load(&history_path);

    let rec1 = match store.get_version(key, v1) {
        Some(r) => r,
        None => { println!("❌ v{} 不存在", v1); return; }
    };
    let rec2 = match store.get_version(key, v2) {
        Some(r) => r,
        None => { println!("❌ v{} 不存在", v2); return; }
    };

    println!("\n📊 差异比较: {}  v{} ↔ v{}", key, v1, v2);
    println!("{}", "─".repeat(60));

    let lines1: Vec<&str> = rec1.value.lines().collect();
    let lines2: Vec<&str> = rec2.value.lines().collect();

    let max = lines1.len().max(lines2.len());
    for i in 0..max {
        let l1 = lines1.get(i).unwrap_or(&"");
        let l2 = lines2.get(i).unwrap_or(&"");
        if l1 != l2 {
            println!("  {}  - {}", i + 1, l1);
            println!("  {}  + {}", i + 1, l2);
        }
    }
}

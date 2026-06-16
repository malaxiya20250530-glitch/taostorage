use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;

/// 节点声望条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEntry {
    pub node_id: String,
    pub score: i8,
    pub successes: u64,
    pub failures: u64,
    pub last_seen: u64,
}

/// 持久化声望表 — 基于 sled 存储
///
/// 节点离线/重启后信任数据不丢失。
/// 正反馈 +1，负反馈 -2，区间 [-100, 100]。
pub struct ReputationTable {
    entries: HashMap<String, ReputationEntry>,
    db: Option<Db>,
}

impl ReputationTable {
    /// 创建内存声望表
    pub fn new() -> Self {
        Self { entries: HashMap::new(), db: None }
    }

    /// 创建持久化声望表
    pub fn persistent(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        let mut entries = HashMap::new();

        // 从 sled 加载已有条目
        for item in db.iter() {
            let (key, value) = item?;
            if let Ok(entry) = bincode::deserialize::<ReputationEntry>(&value) {
                entries.insert(String::from_utf8_lossy(&key).to_string(), entry);
            }
        }

        Ok(Self { entries, db: Some(db) })
    }

    /// 获取或初始化条目
    pub fn get_or_create(&mut self, node_id: &str) -> &mut ReputationEntry {
        self.entries.entry(node_id.to_string()).or_insert_with(|| {
            ReputationEntry {
                node_id: node_id.to_string(), score: 0,
                successes: 0, failures: 0, last_seen: 0,
            }
        })
    }

    /// 正反馈
    pub fn reward(&mut self, node_id: &str) {
        let entry = self.get_or_create(node_id);
        entry.successes += 1;
        entry.score = (entry.score + 1).min(100);
        self.save_one(node_id);
    }

    /// 负反馈
    pub fn penalize(&mut self, node_id: &str) {
        let entry = self.get_or_create(node_id);
        entry.failures += 1;
        entry.score = (entry.score - 2).max(-100);
        self.save_one(node_id);
    }

    /// 更新最后活跃时间
    pub fn touch(&mut self, node_id: &str) {
        let entry = self.get_or_create(node_id);
        entry.last_seen = now_secs();
        self.save_one(node_id);
    }

    /// 查询声望
    pub fn score(&self, node_id: &str) -> i8 {
        self.entries.get(node_id).map(|e| e.score).unwrap_or(0)
    }

    /// 是否可信
    pub fn is_trusted(&self, node_id: &str) -> bool {
        self.score(node_id) >= 0
    }

    /// 获取所有条目引用
    pub fn all_entries(&self) -> impl Iterator<Item = &ReputationEntry> {
        self.entries.values()
    }

    /// 持久化单个条目
    fn save_one(&self, node_id: &str) {
        if let Some(ref db) = self.db {
            if let Some(entry) = self.entries.get(node_id) {
                if let Ok(data) = bincode::serialize(entry) {
                    let _ = db.insert(node_id.as_bytes(), data);
                }
            }
        }
    }

    /// 刷新所有数据到磁盘
    pub fn flush(&self) -> Result<(), sled::Error> {
        if let Some(ref db) = self.db {
            db.flush()?;
        }
        Ok(())
    }
}

impl Default for ReputationTable {
    fn default() -> Self { Self::new() }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_and_score() {
        let mut rt = ReputationTable::new();
        rt.reward("node-a");
        rt.reward("node-a");
        assert_eq!(rt.score("node-a"), 2);
        assert!(rt.is_trusted("node-a"));
    }

    #[test]
    fn test_penalize() {
        let mut rt = ReputationTable::new();
        rt.penalize("bad-node");
        rt.penalize("bad-node");
        assert_eq!(rt.score("bad-node"), -4);
        assert!(!rt.is_trusted("bad-node"));
    }

    }

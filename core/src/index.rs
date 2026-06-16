use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::error::{TaoError, TaoResult};
use crate::unit::DataUnit;

// ============================================================
// 标签索引 — 基于 sled 持久化，兼容 v0.2 的标签系统
// ============================================================

/// 标签到数据单元 ID（content_hash hex）的多对多映射
pub struct TagIndex {
    /// tag → Vec<content_hash_hex>
    db: Db,
}

impl TagIndex {
    /// 打开/创建标签索引数据库
    pub fn open(path: impl AsRef<Path>) -> TaoResult<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// 为一个 content_hash 关联多个标签
    pub fn add_tags(&self, content_hash: &[u8; 32], tags: &[String]) -> TaoResult<()> {
        let hash_hex = hex::encode(content_hash);
        for tag in tags {
            let key = self.tag_key(tag, &hash_hex);
            self.db.insert(key.as_bytes(), &[1])?;
        }
        self.db.flush()?;
        Ok(())
    }

    /// 移除某个数据单元的全部标签关联
    pub fn remove_hash(&self, content_hash: &[u8; 32]) -> TaoResult<()> {
        let hash_hex = hex::encode(content_hash);
        // 通过前缀扫描找到所有关联的标签条目
        let prefix = format!("tag:{}:", &hash_hex[..8]); // 用哈希前缀辅助
        let _keys: Vec<String> = self.db.scan_prefix(prefix.as_bytes())
            .filter_map(|r| r.ok())
            .filter(|(k, _)| k.ends_with(hash_hex.as_bytes()))
            .map(|(k, _)| String::from_utf8_lossy(&k).to_string())
            .collect();
        // 实际上我们用规范格式: "ti:{tag}:{hash_hex}"
        // 先通过反向索引找到所有 tag
        let rev_prefix = format!("ri:{}:", hash_hex);
        let tag_keys: Vec<String> = self.db.scan_prefix(rev_prefix.as_bytes())
            .filter_map(|r| r.ok())
            .map(|(k, _)| String::from_utf8_lossy(&k).to_string())
            .collect();

        for rk in &tag_keys {
            self.db.remove(rk.as_bytes())?;
            // 从 tag 的 tag_key 部分提取 tag 名
            if let Some(tag) = rk.strip_prefix(&format!("ri:{}:", hash_hex)) {
                let fwd_key = format!("ti:{}:{}", tag, hash_hex);
                self.db.remove(fwd_key.as_bytes())?;
            }
        }
        Ok(())
    }

    /// 查询某个标签下的所有 content_hash
    pub fn get_by_tag(&self, tag: &str) -> TaoResult<Vec<String>> {
        let prefix = format!("ti:{}:", tag);
        Ok(self.db.scan_prefix(prefix.as_bytes())
            .filter_map(|r| r.ok())
            .map(|(k, _)| {
                let s = String::from_utf8_lossy(&k).to_string();
                s.rsplit(':').next().unwrap_or("").to_string()
            })
            .collect())
    }

    /// 查询同时包含所有指定标签（AND）的数据单元
    pub fn get_by_tags_all(&self, tags: &[String]) -> TaoResult<Vec<String>> {
        if tags.is_empty() {
            return Ok(vec![]);
        }
        let mut sets: Vec<Vec<String>> = Vec::new();
        for tag in tags {
            sets.push(self.get_by_tag(tag)?);
        }
        // 取交集
        let mut result: Vec<String> = sets[0].clone();
        for set in &sets[1..] {
            result.retain(|h| set.contains(h));
        }
        Ok(result)
    }

    /// 查询包含任一指定标签（OR）的数据单元
    pub fn get_by_tags_any(&self, tags: &[String]) -> TaoResult<Vec<String>> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for tag in tags {
            for hash_hex in self.get_by_tag(tag)? {
                if seen.insert(hash_hex.clone()) {
                    result.push(hash_hex);
                }
            }
        }
        Ok(result)
    }

    /// 标签云：统计每个标签的使用次数
    pub fn tag_cloud(&self) -> TaoResult<Vec<(String, usize)>> {
        // 从 tag 前缀扫描，统计每个标签的出现次数
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for result in self.db.scan_prefix(b"ti:") {
            let (key, _) = result?;
            let s = String::from_utf8_lossy(&key).to_string();
            // 格式: "ti:{tag}:{hash_hex}"
            let parts: Vec<&str> = s.splitn(3, ':').collect();
            if parts.len() >= 2 {
                *counts.entry(parts[1].to_string()).or_insert(0) += 1;
            }
        }
        let mut cloud: Vec<(String, usize)> = counts.into_iter().collect();
        cloud.sort_by(|a, b| b.1.cmp(&a.1)); // 按次数降序
        Ok(cloud)
    }

    /// 添加时同时维护正向和反向索引
    pub fn add_tags_bidirectional(&self, content_hash: &[u8; 32], tags: &[String]) -> TaoResult<()> {
        let hash_hex = hex::encode(content_hash);
        for tag in tags {
            // 正向: tag → hash
            let fwd = format!("ti:{}:{}", tag, hash_hex);
            self.db.insert(fwd.as_bytes(), &[1])?;
            // 反向: hash → tag
            let rev = format!("ri:{}:{}", hash_hex, tag);
            self.db.insert(rev.as_bytes(), &[1])?;
        }
        self.db.flush()?;
        Ok(())
    }

    fn tag_key(&self, tag: &str, hash_hex: &str) -> String {
        format!("ti:{}:{}", tag, hash_hex)
    }
}

// ============================================================
// 模糊搜索引擎 — 兼容 v0.2 的 search 语义
// ============================================================

/// 搜索结果条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub content_hash: String,
    pub key: String,
    pub value_snippet: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub match_field: String, // "key", "value", "tag"
}

/// 模糊搜索 — 扫描 sled 中的所有 DataUnit，做子串匹配
pub fn fuzzy_search(
    store: &sled::Db,
    query: &str,
    max_results: usize,
) -> TaoResult<Vec<SearchHit>> {
    let q = query.to_lowercase();
    let mut results = Vec::new();

    for item in store.iter() {
        let (_key, value) = item?;
        if let Ok(unit) = bincode::deserialize::<DataUnit>(&value) {
            let key = &unit.yang.name;
            let value_str = String::from_utf8_lossy(&unit.yin.payload);
            let tags = &unit.yang.tags;

            let mut matched = false;
            let mut match_field = String::new();

            // 搜索 key
            if key.to_lowercase().contains(&q) {
                matched = true;
                match_field = "key".to_string();
            }

            // 搜索 value
            if !matched && value_str.to_lowercase().contains(&q) {
                matched = true;
                match_field = "value".to_string();
            }

            // 搜索 tag
            if !matched {
                for tag in tags {
                    if tag.to_lowercase().contains(&q) {
                        matched = true;
                        match_field = "tag".to_string();
                        break;
                    }
                }
            }

            if matched {
                let snippet = if value_str.len() > 80 {
                    format!("{}...", &value_str[..80])
                } else {
                    value_str.to_string()
                };

                results.push(SearchHit {
                    content_hash: hex::encode(unit.yin.content_hash),
                    key: key.clone(),
                    value_snippet: snippet,
                    tags: tags.clone(),
                    created_at: unit.yang.created_at,
                    match_field,
                });
            }
        }

        if results.len() >= max_results {
            break;
        }
    }

    Ok(results)
}

// ============================================================
// 统计引擎 — 兼容 v0.2 的 stats 命令
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    pub total_items: usize,
    pub unique_keys: usize,
    pub total_tags: usize,
    pub top_keys: Vec<(String, usize)>,
    pub top_tags: Vec<(String, usize)>,
}

/// 扫描整个 sled 数据库收集统计信息
pub fn collect_stats(
    store: &sled::Db,
    tag_index: &TagIndex,
    top_n: usize,
) -> TaoResult<StoreStats> {
    let mut total_items = 0usize;
    let mut key_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    for item in store.iter() {
        let (_key, value) = item?;
        if let Ok(unit) = bincode::deserialize::<DataUnit>(&value) {
            total_items += 1;
            *key_counts.entry(unit.yang.name.clone()).or_insert(0) += 1;
            for tag in &unit.yang.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }

    let unique_keys = key_counts.len();
    let total_tags: usize = tag_counts.values().sum();

    let mut top_keys: Vec<(String, usize)> = key_counts.into_iter().collect();
    top_keys.sort_by(|a, b| b.1.cmp(&a.1));
    top_keys.truncate(top_n);

    let top_tags = tag_index.tag_cloud().unwrap_or_default();
    let top_tags = top_tags.into_iter().take(top_n).collect();

    Ok(StoreStats {
        total_items,
        unique_keys,
        total_tags,
        top_keys,
        top_tags,
    })
}

// ============================================================
// 备份导出/导入 — JSON 格式兼容 v0.2
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupItem {
    pub id: String,
    pub key: String,
    pub value: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub version: String,
    pub exported_at: u64,
    pub stats: BackupStats,
    pub data: BackupItems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStats {
    pub total_items: usize,
    pub unique_keys: usize,
    pub total_tags: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupItems {
    pub items: Vec<BackupItem>,
    pub tags: Vec<BackupTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTag {
    pub item_id: String,
    pub tag: String,
}

/// 导出为 JSON（v0.2 兼容格式）
pub fn export_backup(
    store: &sled::Db,
    tag_index: &TagIndex,
) -> TaoResult<String> {
    let stats = collect_stats(store, tag_index, 999)?;
    let mut items = Vec::new();
    let mut tags = Vec::new();

    for item in store.iter() {
        let (_key, value) = item?;
        if let Ok(unit) = bincode::deserialize::<DataUnit>(&value) {
            let id = hex::encode(unit.yin.content_hash);
            let value_str = String::from_utf8_lossy(&unit.yin.payload).to_string();
            items.push(BackupItem {
                id: id.clone(),
                key: unit.yang.name.clone(),
                value: value_str,
                tags: unit.yang.tags.clone(),
                created_at: unit.yang.created_at,
                updated_at: unit.yang.last_accessed,
            });
            for tag in &unit.yang.tags {
                tags.push(BackupTag {
                    item_id: id.clone(),
                    tag: tag.clone(),
                });
            }
        }
    }

    let backup = BackupData {
        version: "0.2.0".to_string(),
        exported_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        stats: BackupStats {
            total_items: stats.total_items,
            unique_keys: stats.unique_keys,
            total_tags: stats.total_tags,
        },
        data: BackupItems { items, tags },
    };

    serde_json::to_string_pretty(&backup)
        .map_err(|e| TaoError::QiState(e.to_string()))
}

/// 从 JSON 备份导入
pub fn import_backup(
    store: &sled::Db,
    tag_index: &TagIndex,
    json_str: &str,
) -> TaoResult<ImportResult> {
    let backup: BackupData = serde_json::from_str(json_str)
        .map_err(|e| TaoError::QiState(e.to_string()))?;

    let mut imported = 0usize;
    let mut skipped = 0usize;

    for item in &backup.data.items {
        let payload = item.value.as_bytes().to_vec();
        let tags = item.tags.clone();

        // 从 hex id 解析 content_hash
        let content_hash: [u8; 32] = match hex::decode(&item.id) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            }
            _ => {
                // 如果 id 不是有效的 32 字节 hex，用 SHA256(value) 作为新 hash
                use sha2::{Sha256, Digest};
                Sha256::digest(&payload).into()
            }
        };

        let mut unit = DataUnit::new(payload, item.key.clone(), [0u8; 32]);
        unit.yin.content_hash = content_hash;
        unit.yang.tags = tags.clone();
        unit.yang.created_at = item.created_at;
        unit.yang.last_accessed = item.updated_at;

        // 检查是否已存在
        if store.contains_key(&content_hash)? {
            skipped += 1;
            continue;
        }

        bincode::serialize(&unit)
            .map_err(|e| TaoError::QiState(e.to_string()))
            .and_then(|bytes| {
                store.insert(&content_hash, bytes)?;
                Ok::<_, TaoError>(())
            })?;

        tag_index.add_tags_bidirectional(&content_hash, &tags)?;
        imported += 1;
    }

    store.flush()?;

    Ok(ImportResult { imported, skipped })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::LocalStore;
    use crate::unit::DataUnit;
    use std::path::Path;

    fn setup() -> (sled::Db, TagIndex) {
        let dir = tempfile::tempdir().unwrap();
        let db = sled::open(dir.path().join("store")).unwrap();
        let index = TagIndex::open(dir.path().join("tags")).unwrap();
        (db, index)
    }

    #[test]
    fn test_tag_add_and_query() {
        let (_db, index) = setup();
        let hash = [0x01u8; 32];
        let tags = vec!["哲学".to_string(), "道德经".to_string()];
        index.add_tags_bidirectional(&hash, &tags).unwrap();

        let found = index.get_by_tag("哲学").unwrap();
        assert_eq!(found, vec![hex::encode(hash)]);
    }

    #[test]
    fn test_tag_cloud() {
        let (_db, index) = setup();
        let h1 = [0x01u8; 32];
        let h2 = [0x02u8; 32];
        index.add_tags_bidirectional(&h1, &["哲学".to_string(), "道家".to_string()]).unwrap();
        index.add_tags_bidirectional(&h2, &["哲学".to_string()]).unwrap();

        let cloud = index.tag_cloud().unwrap();
        assert_eq!(cloud[0].0, "哲学");
        assert_eq!(cloud[0].1, 2);
    }

    #[test]
    fn test_fuzzy_search() {
        let dir = tempfile::tempdir().unwrap();
        let db = sled::open(dir.path().join("store")).unwrap();

        let unit = DataUnit::new("道可道，非常道".to_string().into_bytes(), "tao".into(), [0u8; 32]);
        let bytes = bincode::serialize(&unit).unwrap();
        db.insert(&unit.yin.content_hash, bytes).unwrap();

        let hits = fuzzy_search(&db, "常道", 10).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].key, "tao");
    }

    #[test]
    fn test_export_import_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let db = sled::open(dir.path().join("store")).unwrap();
        let index = TagIndex::open(dir.path().join("tags")).unwrap();

        let unit = DataUnit::new(b"test data".to_vec(), "test".into(), [0u8; 32]);
        let bytes = bincode::serialize(&unit).unwrap();
        db.insert(&unit.yin.content_hash, bytes).unwrap();
        index.add_tags_bidirectional(&unit.yin.content_hash, &["demo".to_string()]).unwrap();

        let json = export_backup(&db, &index).unwrap();
        assert!(json.contains("test data"));

        // 导入到新库
        let dir2 = tempfile::tempdir().unwrap();
        let db2 = sled::open(dir2.path().join("store")).unwrap();
        let index2 = TagIndex::open(dir2.path().join("tags")).unwrap();
        let result = import_backup(&db2, &index2, &json).unwrap();
        assert_eq!(result.imported, 1);
    }
}

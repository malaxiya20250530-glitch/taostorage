use crate::error::TaoResult;
use crate::unit::DataUnit;
use sled::Db;
use std::path::Path;

/// 本地持久化引擎 — "坤"层厚德载物
///
/// 用 sled（嵌入式 B+ 树）作为单节点存储后端。
/// Key: content_hash (32 bytes)
/// Value: bincode 编码的 DataUnit
pub struct LocalStore {
    db: Db,
}

impl LocalStore {
    pub fn open(path: impl AsRef<Path>) -> TaoResult<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// 写入数据单元（阴）
    pub fn store(&self, unit: &DataUnit) -> TaoResult<()> {
        let key = &unit.yin.content_hash;
        let value = bincode::serialize(unit)?;
        self.db.insert(key, value)?;
        self.db.flush()?;
        Ok(())
    }

    /// 按内容哈希读取（内容寻址）
    pub fn get(&self, content_hash: &[u8; 32]) -> TaoResult<Option<DataUnit>> {
        match self.db.get(content_hash)? {
            Some(bytes) => {
                let unit: DataUnit = bincode::deserialize(&bytes)?;
                Ok(Some(unit))
            }
            None => Ok(None),
        }
    }

    /// 删除数据单元
    pub fn remove(&self, content_hash: &[u8; 32]) -> TaoResult<()> {
        self.db.remove(content_hash)?;
        self.db.flush()?;
        Ok(())
    }

    /// 检查是否存在
    pub fn exists(&self, content_hash: &[u8; 32]) -> TaoResult<bool> {
        Ok(self.db.contains_key(content_hash)?)
    }

    /// 遍历所有数据单元
    pub fn iter(&self) -> impl Iterator<Item = TaoResult<DataUnit>> {
        self.db.iter().values().map(|r| {
            let bytes = r?;
            let unit: DataUnit = bincode::deserialize(&bytes)?;
            Ok(unit)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::DataUnit;

    #[test]
    fn test_store_and_retrieve() -> crate::error::TaoResult<()> {
        let store = LocalStore::open("./target/tao_test_store")?;
        let unit = DataUnit::new(b"persistent data".to_vec(), "test".into(), [0u8; 32]);
        let id = unit.yin.content_hash;

        store.store(&unit)?;
        assert!(store.exists(&id)?);

        let retrieved = store.get(&id)?.expect("should exist");
        assert_eq!(retrieved.yin.payload, b"persistent data");
        assert!(retrieved.verify());

        store.remove(&id)?;
        assert!(!store.exists(&id)?);

        Ok(())
    }
}

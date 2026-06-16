use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// 数据单元的生命周期阶段（六十四卦简化映射）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hexagram {
    /// 屯 — 初生：数据刚写入
    Zhun,
    /// 既济 — 功成：标准保护
    Jiji,
    /// 泰 — 通泰：热数据，高频访问
    Tai,
    /// 否 — 闭塞：冷数据，已压缩
    Pi,
    /// 剥 — 剥落：冗余不足，紧急重构
    Bo,
    /// 坤 — 归藏：已归档
    Kun,
}

/// 阴（静/体）：原始数据内容，通过内容哈希寻址
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Yin {
    /// 数据载荷
    pub payload: Vec<u8>,
    /// 内容哈希（不可变的地址）
    pub content_hash: [u8; 32],
}

/// 阳（动/用）：元数据、索引、访问热度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Yang {
    /// 逻辑名称（可变的寻址标签）
    pub name: String,
    /// 创建时间戳
    pub created_at: u64,
    /// 最后访问时间
    pub last_accessed: u64,
    /// 访问计数
    pub access_count: u64,
    /// 热度值（0=冷, 255=热）
    pub heat: u8,
    /// 关联标签
    pub tags: Vec<String>,
}

/// 气（动态内核）：当前状态、决策上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qi {
    /// 当前卦象（生命周期阶段）
    pub hexagram: Hexagram,
    /// 副本数
    pub replica_count: u8,
    /// 期望副本数
    pub target_replicas: u8,
    /// 纠删码配置：k 数据片
    pub ec_data_shards: u8,
    /// 纠删码配置：m 校验片
    pub ec_parity_shards: u8,
    /// 最后健康检查时间
    pub last_health_check: u64,
    /// 所有者（Ed25519 公钥）
    pub owner: [u8; 32],
}

/// DataUnit — 阴阳气三位一体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataUnit {
    pub yin: Yin,
    pub yang: Yang,
    pub qi: Qi,
}

impl DataUnit {
    /// 从原始数据创建新单元（屯卦·初生）
    pub fn new(
        payload: Vec<u8>,
        name: String,
        owner: [u8; 32],
    ) -> Self {
        let content_hash = Sha256::digest(&payload).into();
        let now = Self::now();

        Self {
            yin: Yin { payload, content_hash },
            yang: Yang {
                name,
                created_at: now,
                last_accessed: now,
                access_count: 0,
                heat: 0,
                tags: Vec::new(),
            },
            qi: Qi {
                hexagram: Hexagram::Zhun,
                replica_count: 1,
                target_replicas: 3,
                ec_data_shards: 6,
                ec_parity_shards: 2,
                last_health_check: now,
                owner,
            },
        }
    }

    /// 内容寻址 ID（base58 编码的 content_hash）
    pub fn id(&self) -> String {
        bs58::encode(&self.yin.content_hash).into_string()
    }

    /// 记录访问，更新阳的热度
    pub fn touch(&mut self) {
        let now = Self::now();
        self.yang.last_accessed = now;
        self.yang.access_count += 1;

        // 简单热度算法：访问越多越热
        let heat = (self.yang.access_count.min(255)) as u8;
        // 时间衰减：每 3600 秒减 1
        let age = now.saturating_sub(self.yang.created_at) / 3600;
        self.yang.heat = heat.saturating_sub(age.min(255) as u8);
    }

    /// 内容验证：重算哈希比对
    pub fn verify(&self) -> bool {
        let actual: [u8; 32] = Sha256::digest(&self.yin.payload).into();
        actual == self.yin.content_hash
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_unit_zhun_hexagram() {
        let owner = [0u8; 32];
        let unit = DataUnit::new(b"hello tao".to_vec(), "test".into(), owner);
        assert_eq!(unit.qi.hexagram, Hexagram::Zhun);
        assert_eq!(unit.qi.replica_count, 1);
        assert!(unit.verify());
    }

    #[test]
    fn test_verify_integrity() {
        let owner = [0u8; 32];
        let mut unit = DataUnit::new(b"immutable".to_vec(), "x".into(), owner);
        assert!(unit.verify());
        // 篡改后应失败
        unit.yin.payload = b"corrupted".to_vec();
        assert!(!unit.verify());
    }

    #[test]
    fn test_touch_updates_heat() {
        let owner = [0u8; 32];
        let mut unit = DataUnit::new(b"data".to_vec(), "hot".into(), owner);
        assert_eq!(unit.yang.heat, 0);
        for _ in 0..5 {
            unit.touch();
        }
        assert!(unit.yang.heat > 0);
        assert_eq!(unit.yang.access_count, 5);
    }

    #[test]
    fn test_id_is_content_addressed() {
        let owner = [0u8; 32];
        let a = DataUnit::new(b"same".to_vec(), "a".into(), owner);
        let b = DataUnit::new(b"same".to_vec(), "b".into(), owner);
        assert_eq!(a.id(), b.id()); // 相同内容 = 相同 ID
    }
}

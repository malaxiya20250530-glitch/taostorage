use crate::unit::{Hexagram, Qi};

/// 气机决策结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QiAction {
    /// 维持当前状态
    Noop,
    /// 增加副本
    Replicate,
    /// 进入紧急修复（剥卦）
    EmergencyRebuild,
    /// 提升到热层（泰卦）
    PromoteToHot,
    /// 降级到冷层并压缩（否卦）
    DemoteToCold,
    /// 归档（坤卦）
    Archive,
    /// 已达标准保护（既济卦）
    Stabilized,
}

/// 气机决策引擎：负阴抱阳，冲气为和
///
/// 每个 DataUnit 的守护进程只根据局部信息（自身 qi 状态、节点健康度）
/// 做决策，全局均衡从局部互动涌现。
pub fn decide(qi: &Qi, _neighbor_health: f64) -> QiAction {
    // 剥卦 → 紧急修复
    if qi.hexagram == Hexagram::Bo {
        return QiAction::EmergencyRebuild;
    }

    // 副本不足 → 复制
    if qi.replica_count < qi.target_replicas {
        return QiAction::Replicate;
    }

    // 根据热度决定升降
    match qi.hexagram {
        Hexagram::Zhun if qi.replica_count >= qi.target_replicas => {
            QiAction::Stabilized
        }
        Hexagram::Tai => {
            // 热数据保持在热层，检查副本
            if qi.replica_count < qi.target_replicas {
                QiAction::Replicate
            } else {
                QiAction::Noop
            }
        }
        Hexagram::Pi => {
            // 冷数据，检查是否需要归档
            QiAction::Noop
        }
        Hexagram::Jiji => QiAction::Noop,
        Hexagram::Kun => QiAction::Noop,
        Hexagram::Bo => QiAction::EmergencyRebuild,
        _ => QiAction::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::Qi;

    fn test_qi(replicas: u8, target: u8, hex: Hexagram) -> Qi {
        Qi {
            hexagram: hex,
            replica_count: replicas,
            target_replicas: target,
            ec_data_shards: 6,
            ec_parity_shards: 2,
            last_health_check: 0,
            owner: [0u8; 32],
        }
    }

    #[test]
    fn test_bo_triggers_rebuild() {
        let qi = test_qi(1, 3, Hexagram::Bo);
        assert_eq!(decide(&qi, 1.0), QiAction::EmergencyRebuild);
    }

    #[test]
    fn test_insufficient_replicas() {
        let qi = test_qi(1, 3, Hexagram::Zhun);
        assert_eq!(decide(&qi, 1.0), QiAction::Replicate);
    }

    #[test]
    fn test_zhun_stabilizes() {
        let qi = test_qi(3, 3, Hexagram::Zhun);
        assert_eq!(decide(&qi, 1.0), QiAction::Stabilized);
    }
}

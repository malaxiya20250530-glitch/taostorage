// ============================================================
// 🧲 TaoStorage 邀请奖励系统
// "邀请一人，节点信誉 +1" — 病毒式传播引擎
// ============================================================

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// 邀请码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCode {
    /// 邀请码 (8 字符)
    pub code: String,
    /// 邀请方节点 ID
    pub inviter_id: String,
    /// 创建时间戳
    pub created_at: u64,
    /// 过期时间 (7 天)
    pub expires_at: u64,
    /// 使用次数
    pub use_count: u32,
    /// 最大使用次数 (0 = 无限)
    pub max_uses: u32,
    /// 是否激活
    pub active: bool,
}

/// 节点信誉分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReputation {
    /// 节点 ID
    pub node_id: String,
    /// 总信誉分
    pub score: u64,
    /// 邀请成功的数量
    pub invites_sent: u64,
    /// 存储的数据量 (bytes)
    pub storage_bytes: u64,
    /// 在线时间 (秒)
    pub uptime_secs: u64,
    /// 等级
    pub rank: Rank,
    /// 获得的徽章
    pub badges: Vec<Badge>,
    /// 历史记录
    pub history: Vec<ReputationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    /// 凡 — 初入道
    Fan,
    /// 士 — 入门
    Shi,
    /// 道 — 得道
    Dao,
    /// 玄 — 玄妙
    Xuan,
    /// 圣 — 圣人
    Sheng,
}

impl Rank {
    pub fn from_score(score: u64) -> Self {
        match score {
            0..=9 => Rank::Fan,
            10..=49 => Rank::Shi,
            50..=199 => Rank::Dao,
            200..=999 => Rank::Xuan,
            _ => Rank::Sheng,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Rank::Fan => "凡 (学徒)",
            Rank::Shi => "士 (修士)",
            Rank::Dao => "道 (得道者)",
            Rank::Xuan => "玄 (玄妙境)",
            Rank::Sheng => "圣 (圣人)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Badge {
    /// 创始节点
    Genesis,
    /// 传道者 — 邀请 10 人
    Preacher,
    /// 宗师 — 邀请 100 人
    Master,
    /// 存储大师 — 存储 1GB
    StorageMaster,
    /// 长寿节点 — 在线 30 天
    Longevity,
    /// 泰卦 — 热数据贡献者
    Tai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub delta: i64,
    pub description: String,
}

/// 邀请奖励系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteSystem {
    pub codes: HashMap<String, InviteCode>,
    pub reputations: HashMap<String, NodeReputation>,
}

impl InviteSystem {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
            reputations: HashMap::new(),
        }
    }

    /// 生成邀请码
    pub fn generate_code(&mut self, inviter_id: &str) -> InviteCode {
        let now = chrono::Utc::now().timestamp() as u64;
        let raw = format!("{}:{}:{}", inviter_id, now, rand::random::<u64>());
        let hash = Sha256::digest(raw.as_bytes());
        let code = hex::encode(&hash[..4]).to_uppercase(); // 8 字符

        let invite = InviteCode {
            code: code.clone(),
            inviter_id: inviter_id.to_string(),
            created_at: now,
            expires_at: now + 7 * 24 * 3600, // 7 天
            use_count: 0,
            max_uses: 0, // 无限
            active: true,
        };

        self.codes.insert(code.clone(), invite.clone());

        // 记录事件
        self.add_event(inviter_id, "invite_generated", 0, "生成邀请码");

        invite
    }

    /// 使用邀请码
    pub fn use_code(&mut self, code: &str, new_node_id: &str) -> Result<String, String> {
        let invite = self.codes.get_mut(code).ok_or("邀请码无效")?;

        if !invite.active {
            return Err("邀请码已失效".into());
        }

        let now = chrono::Utc::now().timestamp() as u64;
        if now > invite.expires_at {
            invite.active = false;
            return Err("邀请码已过期".into());
        }

        if invite.max_uses > 0 && invite.use_count >= invite.max_uses {
            return Err("邀请码已达使用上限".into());
        }

        invite.use_count += 1;

        // 邀请方获得信誉分
        let inviter_id = invite.inviter_id.clone();
        self.add_reputation(&inviter_id, 10, &format!("邀请节点 {}", new_node_id));

        // 新节点获得初始信誉分
        self.add_reputation(new_node_id, 5, &format!("通过邀请码 {} 加入", code));

        // 检查徽章
        self.check_badges(&inviter_id);

        Ok(inviter_id)
    }

    /// 添加信誉分
    pub fn add_reputation(&mut self, node_id: &str, delta: u64, reason: &str) {
        let now = chrono::Utc::now().timestamp() as u64;
        let rep = self.reputations.entry(node_id.to_string()).or_insert_with(|| {
            NodeReputation {
                node_id: node_id.to_string(),
                score: 0,
                invites_sent: 0,
                storage_bytes: 0,
                uptime_secs: 0,
                rank: Rank::Fan,
                badges: vec![],
                history: vec![],
            }
        });

        rep.score += delta;
        rep.history.push(ReputationEvent {
            timestamp: now,
            event_type: "reputation_gained".into(),
            delta: delta as i64,
            description: reason.to_string(),
        });

        // 更新等级
        rep.rank = Rank::from_score(rep.score);
    }

    /// 记录邀请
    pub fn record_invite(&mut self, node_id: &str) {
        let rep = self.reputations.entry(node_id.to_string()).or_insert_with(|| {
            NodeReputation {
                node_id: node_id.to_string(),
                score: 0,
                invites_sent: 0,
                storage_bytes: 0,
                uptime_secs: 0,
                rank: Rank::Fan,
                badges: vec![],
                history: vec![],
            }
        });
        rep.invites_sent += 1;
    }

    /// 添加事件
    pub fn add_event(&mut self, node_id: &str, event_type: &str, delta: i64, description: &str) {
        let now = chrono::Utc::now().timestamp() as u64;
        let rep = self.reputations.entry(node_id.to_string()).or_insert_with(|| {
            NodeReputation {
                node_id: node_id.to_string(),
                score: 0,
                invites_sent: 0,
                storage_bytes: 0,
                uptime_secs: 0,
                rank: Rank::Fan,
                badges: vec![],
                history: vec![],
            }
        });
        rep.history.push(ReputationEvent {
            timestamp: now,
            event_type: event_type.to_string(),
            delta,
            description: description.to_string(),
        });

        if delta > 0 {
            rep.score = (rep.score as i64 + delta) as u64;
            rep.rank = Rank::from_score(rep.score);
        }
    }

    /// 检查并授予徽章
    pub fn check_badges(&mut self, node_id: &str) {
        let rep = match self.reputations.get(node_id) {
            Some(r) => r.clone(),
            None => return,
        };

        let mut new_badges = vec![];

        // Preacher: 邀请 10 人
        if rep.invites_sent >= 10 && !rep.badges.contains(&Badge::Preacher) {
            new_badges.push(Badge::Preacher);
        }

        // Master: 邀请 100 人
        if rep.invites_sent >= 100 && !rep.badges.contains(&Badge::Master) {
            new_badges.push(Badge::Master);
        }

        // Longevity: 在线 30 天
        if rep.uptime_secs >= 30 * 24 * 3600 && !rep.badges.contains(&Badge::Longevity) {
            new_badges.push(Badge::Longevity);
        }

        // StorageMaster: 存储 1GB
        if rep.storage_bytes >= 1_000_000_000 && !rep.badges.contains(&Badge::StorageMaster) {
            new_badges.push(Badge::StorageMaster);
        }

        if !new_badges.is_empty() {
            let rep = self.reputations.get_mut(node_id).unwrap();
            for badge in new_badges {
                rep.badges.push(badge.clone());
                let now = chrono::Utc::now().timestamp() as u64;
                rep.history.push(ReputationEvent {
                    timestamp: now,
                    event_type: "badge_earned".into(),
                    delta: 50,
                    description: format!("获得徽章: {:?}", badge),
                });
                rep.score += 50; // 徽章奖励
                rep.rank = Rank::from_score(rep.score);
            }
        }
    }

    /// 获取节点排行榜
    pub fn leaderboard(&self, top_n: usize) -> Vec<(&str, &NodeReputation)> {
        let mut entries: Vec<(&str, &NodeReputation)> = self.reputations
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        entries.sort_by(|a, b| b.1.score.cmp(&a.1.score));
        entries.into_iter().take(top_n).collect()
    }

    /// 获取邀请码信息
    pub fn get_code_info(&self, code: &str) -> Option<&InviteCode> {
        self.codes.get(code)
    }

    /// 获取节点信誉
    pub fn get_reputation(&self, node_id: &str) -> Option<&NodeReputation> {
        self.reputations.get(node_id)
    }

    /// 序列化到 JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// 从 JSON 加载
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

// ============================================================
// 积分规则常量
// ============================================================
pub mod rewards {
    /// 邀请一人
    pub const INVITE_BONUS: u64 = 10;
    /// 被邀请加入
    pub const JOIN_BONUS: u64 = 5;
    /// 存储 1MB 数据
    pub const STORAGE_PER_MB: u64 = 1;
    /// 在线一小时
    pub const UPTIME_PER_HOUR: u64 = 1;
    /// 获得徽章
    pub const BADGE_BONUS: u64 = 50;
    /// 创始节点奖励
    pub const GENESIS_BONUS: u64 = 100;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_use_code() {
        let mut system = InviteSystem::new();
        let invite = system.generate_code("node-1");
        assert_eq!(invite.code.len(), 8);
        assert!(invite.active);

        let result = system.use_code(&invite.code, "node-2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "node-1");

        // 检查信誉
        let rep1 = system.get_reputation("node-1").unwrap();
        assert_eq!(rep1.score, 10);
        assert_eq!(rep1.rank, Rank::Shi);

        let rep2 = system.get_reputation("node-2").unwrap();
        assert_eq!(rep2.score, 5);
        assert_eq!(rep2.rank, Rank::Fan);
    }

    #[test]
    fn test_rank_progression() {
        assert_eq!(Rank::from_score(0), Rank::Fan);
        assert_eq!(Rank::from_score(5), Rank::Fan);
        assert_eq!(Rank::from_score(10), Rank::Shi);
        assert_eq!(Rank::from_score(50), Rank::Dao);
        assert_eq!(Rank::from_score(200), Rank::Xuan);
        assert_eq!(Rank::from_score(1000), Rank::Sheng);
    }

    #[test]
    fn test_badges() {
        let mut system = InviteSystem::new();
        let node = "node-1";

        // 初始无徽章
        system.add_reputation(node, 0, "init");
        assert!(system.get_reputation(node).unwrap().badges.is_empty());

        // 邀请 10 人
        for i in 0..10 {
            system.add_reputation(node, rewards::INVITE_BONUS, &format!("invite {}", i));
            system.record_invite(node);
        }
        system.check_badges(node);

        let rep = system.get_reputation(node).unwrap();
        assert!(rep.badges.contains(&Badge::Preacher));
        assert!(rep.score >= 100);
    }

    #[test]
    fn test_leaderboard() {
        let mut system = InviteSystem::new();
        system.add_reputation("alice", 100, "good");
        system.add_reputation("bob", 50, "ok");
        system.add_reputation("charlie", 200, "great");

        let board = system.leaderboard(2);
        assert_eq!(board.len(), 2);
        assert_eq!(board[0].0, "charlie");
        assert_eq!(board[1].0, "alice");
    }

    #[test]
    fn test_serialization() {
        let mut system = InviteSystem::new();
        system.generate_code("node-1");
        system.add_reputation("node-1", 42, "test");

        let json = system.to_json();
        let loaded = InviteSystem::from_json(&json).unwrap();
        assert_eq!(loaded.codes.len(), 1);
        assert_eq!(loaded.get_reputation("node-1").unwrap().score, 42);
    }
}

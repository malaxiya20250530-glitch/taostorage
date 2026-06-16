use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// 社群契约 — "长生久视"的人因修复机制
///
/// 一群人通过多重签名共同声明对一份数据集的维护责任。
/// 协议层定期发起挑战-响应审计，确保守护者社群依然在线。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityContract {
    /// 合约 ID
    pub id: [u8; 32],
    /// 受保护的数据内容哈希
    pub content_hash: [u8; 32],
    /// 守护者公钥列表
    pub guardians: Vec<[u8; 32]>,
    /// 最少签名数（阈值）
    pub threshold: usize,
    /// 签名集合（公钥 → 签名）
    pub signatures: HashMap<[u8; 32], Vec<u8>>,
    /// 创建时间
    pub created_at: u64,
    /// 上次审计时间
    pub last_audit: u64,
    /// 审计间隔（秒）
    pub audit_interval: u64,
    /// 合约是否活跃
    pub active: bool,
}

impl CommunityContract {
    /// 创建新合约
    pub fn new(
        content_hash: [u8; 32],
        guardians: Vec<[u8; 32]>,
        threshold: usize,
        audit_interval: u64,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&content_hash);
        for g in &guardians {
            hasher.update(g);
        }
        let id: [u8; 32] = hasher.finalize().into();
        let now = now_secs();

        Self {
            id,
            content_hash,
            guardians,
            threshold,
            signatures: HashMap::new(),
            created_at: now,
            last_audit: now,
            audit_interval,
            active: true,
        }
    }

    /// 添加守护者签名
    pub fn sign(&mut self, guardian_pubkey: &[u8; 32], signature: Vec<u8>) {
        if self.guardians.contains(guardian_pubkey) {
            self.signatures.insert(*guardian_pubkey, signature);
        }
    }

    /// 是否达到阈值
    pub fn is_fulfilled(&self) -> bool {
        self.signatures.len() >= self.threshold
    }

    /// 检查是否需要审计
    pub fn needs_audit(&self) -> bool {
        self.active && now_secs() >= self.last_audit + self.audit_interval
    }

    /// 标记审计完成
    pub fn audit_passed(&mut self) {
        self.last_audit = now_secs();
    }

    /// 停用合约（审计失败）
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// 剩余守护者数量
    pub fn active_guardian_count(&self) -> usize {
        self.signatures.len()
    }
}

/// 审计挑战 — 定期向守护者发起存储证明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChallenge {
    pub contract_id: [u8; 32],
    pub challenge: [u8; 32],
    pub issued_at: u64,
    pub deadline: u64,
}

impl AuditChallenge {
    pub fn new(contract_id: [u8; 32]) -> Self {
        use rand::RngCore;
        let mut challenge = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut challenge);
        let now = now_secs();
        Self {
            contract_id,
            challenge,
            issued_at: now,
            deadline: now + 3600,
        }
    }

    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        now_secs() > self.deadline
    }
}

/// 审计响应 — 守护者证明仍持有数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResponse {
    pub challenge: AuditChallenge,
    /// proof = SHA256(data || challenge || guardian_pubkey)
    pub proof: [u8; 32],
    pub guardian: [u8; 32],
}

impl AuditResponse {
    /// 生成证明
    pub fn generate(
        data: &[u8],
        challenge: &AuditChallenge,
        guardian: &[u8; 32],
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&challenge.challenge);
        hasher.update(guardian);
        let proof: [u8; 32] = hasher.finalize().into();

        Self {
            challenge: challenge.clone(),
            proof,
            guardian: *guardian,
        }
    }
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
    fn test_contract_creation() {
        let hash = [0x42u8; 32];
        let guardians = vec![[0x01u8; 32], [0x02u8; 32], [0x03u8; 32]];
        let contract = CommunityContract::new(hash, guardians, 2, 3600);

        assert!(contract.active);
        assert_eq!(contract.threshold, 2);
        assert!(!contract.is_fulfilled());
    }

    #[test]
    fn test_sign_and_fulfill() {
        let hash = [0xabu8; 32];
        let g1 = [0x01u8; 32];
        let g2 = [0x02u8; 32];
        let mut contract = CommunityContract::new(hash, vec![g1, g2], 2, 3600);

        assert!(!contract.is_fulfilled());
        contract.sign(&g1, vec![1, 2, 3]);
        assert!(!contract.is_fulfilled());
        contract.sign(&g2, vec![4, 5, 6]);
        assert!(contract.is_fulfilled());
    }

    #[test]
    fn test_audit_challenge() {
        let contract_id = [0x11u8; 32];
        let challenge = AuditChallenge::new(contract_id);
        assert!(!challenge.is_expired());
        assert!(challenge.deadline > challenge.issued_at);
    }

    #[test]
    fn test_audit_response() {
        let data = b"immutable archive content";
        let contract_id = [0x22u8; 32];
        let challenge = AuditChallenge::new(contract_id);
        let guardian = [0x01u8; 32];

        let response = AuditResponse::generate(data, &challenge, &guardian);

        // 验证：重新计算证明
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&challenge.challenge);
        hasher.update(&guardian);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(response.proof, expected);
    }
}

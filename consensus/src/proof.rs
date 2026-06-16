use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// 存储证明（Proof of Storage）
///
/// 用于社群契约中的定期审计：
/// 守护者定期收到挑战，必须证明仍然持有数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallenge {
    /// 挑战的目标内容哈希
    pub content_hash: [u8; 32],
    /// 随机挑战值
    pub nonce: [u8; 32],
    /// 挑战发起时间
    pub issued_at: u64,
    /// 响应截止时间
    pub deadline: u64,
}

/// 挑战的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallengeResponse {
    /// 原始挑战
    pub challenge: StorageChallenge,
    /// 证明：SHA256(data || nonce)
    pub proof: [u8; 32],
}

impl StorageChallenge {
    /// 创建新挑战
    pub fn new(content_hash: [u8; 32]) -> Self {
        use rand::RngCore;
        let mut nonce = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut nonce);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            content_hash,
            nonce,
            issued_at: now,
            deadline: now + 3600, // 1 小时内必须响应
        }
    }

    /// 验证响应
    pub fn verify(&self, response: &StorageChallengeResponse) -> bool {
        response.challenge.content_hash == self.content_hash
            && response.challenge.nonce == self.nonce
    }
}

/// 生成证明：prover 持有 data
pub fn generate_proof(data: &[u8], challenge: &StorageChallenge) -> StorageChallengeResponse {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.update(&challenge.nonce);
    let proof: [u8; 32] = hasher.finalize().into();

    StorageChallengeResponse {
        challenge: challenge.clone(),
        proof,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_response_cycle() {
        let data = b"immutable archive data";
        let content_hash: [u8; 32] = Sha256::digest(data).into();

        let challenge = StorageChallenge::new(content_hash);
        let response = generate_proof(data, &challenge);

        assert!(challenge.verify(&response));
    }
}

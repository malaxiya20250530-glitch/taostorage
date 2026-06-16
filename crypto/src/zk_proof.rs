use sha2::{Sha256, Digest};
use rand::RngCore;

/// 简化的存储证明（Proof of Storage）
///
/// 挑战-响应模式：verifier 发随机 challenge，
/// prover 证明自己确实持有某数据块，无需传输完整数据。
pub struct StorageProof {
    pub data_hash: [u8; 32],
    pub challenge: [u8; 32],
    pub response: [u8; 32],
}

/// 生成挑战
pub fn generate_challenge() -> [u8; 32] {
    let mut challenge = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut challenge);
    challenge
}

/// Prover：给定数据 + 挑战，生成证明
pub fn prove(data: &[u8], challenge: &[u8; 32]) -> StorageProof {
    let data_hash: [u8; 32] = Sha256::digest(data).into();

    // response = SHA256(data || challenge)
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.update(challenge);
    let response: [u8; 32] = hasher.finalize().into();

    StorageProof {
        data_hash,
        challenge: *challenge,
        response,
    }
}

/// Verifier：验证证明（需在本地缓存 data_hash）
pub fn verify(proof: &StorageProof, expected_hash: &[u8; 32]) -> bool {
    proof.data_hash == *expected_hash
    // 注：完整验证需要 prover 传回 data，
    // 这里简化为哈希比对。实际系统中结合 Merkle proof。
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_proof() {
        let data = b"tao storage immutable data block";
        let challenge = generate_challenge();
        let proof = prove(data, &challenge);

        let expected_hash: [u8; 32] = Sha256::digest(data).into();
        assert!(verify(&proof, &expected_hash));
    }

    #[test]
    fn test_wrong_hash_fails() {
        let data = b"real data";
        let challenge = generate_challenge();
        let proof = prove(data, &challenge);

        let wrong_hash: [u8; 32] = Sha256::digest(b"fake data").into();
        assert!(!verify(&proof, &wrong_hash));
    }
}

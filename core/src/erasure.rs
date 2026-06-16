use reed_solomon_erasure::galois_8::ReedSolomon;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use crate::error::{TaoError, TaoResult};

/// 纠删码分片 — "二生三"与"三生万物"的数学内核
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    pub original_len: usize,
    pub index: usize,
    pub data_count: usize,
    pub parity_count: usize,
    pub original_hash: [u8; 32],
    pub data: Vec<u8>,
}

/// 纠删码编码器
pub struct ErasureEncoder {
    inner: ReedSolomon,
    data_shards: usize,
    parity_shards: usize,
}

impl ErasureEncoder {
    pub fn new(data_shards: usize, parity_shards: usize) -> Self {
        let inner = ReedSolomon::new(data_shards, parity_shards)
            .expect("valid reed-solomon params (max 256 total)");
        Self { inner, data_shards, parity_shards }
    }

    pub fn data_shards(&self) -> usize { self.data_shards }
    pub fn parity_shards(&self) -> usize { self.parity_shards }

    pub fn encode(&self, data: &[u8]) -> TaoResult<Vec<Shard>> {
        let k = self.data_shards;
        let m = self.parity_shards;
        let original_len = data.len();
        let original_hash: [u8; 32] = Sha256::digest(data).into();

        let shard_size = (data.len() + k - 1) / k;
        let padded_len = shard_size * k;

        let mut padded = vec![0u8; padded_len];
        padded[..data.len()].copy_from_slice(data);

        let mut shards: Vec<Vec<u8>> = (0..k)
            .map(|i| padded[i * shard_size..(i + 1) * shard_size].to_vec())
            .collect();

        for _ in 0..m {
            shards.push(vec![0u8; shard_size]);
        }

        self.inner.encode(&mut shards).map_err(|e| TaoError::QiState(e.to_string()))?;

        let result: Vec<Shard> = shards
            .into_iter()
            .enumerate()
            .map(|(i, data)| Shard {
                original_len,
                index: i,
                data_count: k,
                parity_count: m,
                original_hash,
                data,
            })
            .collect();

        Ok(result)
    }

    pub fn decode(&self, shard_pairs: &[(Option<Shard>, bool)]) -> TaoResult<Vec<u8>> {
        let k = self.data_shards;
        let m = self.parity_shards;

        if shard_pairs.len() != k + m {
            return Err(TaoError::InsufficientFragments {
                have: shard_pairs.len(), need: k + m,
            });
        }

        let present_count = shard_pairs.iter().filter(|(_, p)| *p).count();
        if present_count < k {
            return Err(TaoError::InsufficientFragments { have: present_count, need: k });
        }

        let shard_size = shard_pairs
            .iter()
            .find_map(|(s, _)| s.as_ref().map(|sh| sh.data.len()))
            .unwrap_or(0);
        if shard_size == 0 {
            return Err(TaoError::QiState("empty shards".into()));
        }

        // 获取原始长度
        let original_len = shard_pairs
            .iter()
            .find_map(|(s, _)| s.as_ref().map(|sh| sh.original_len))
            .unwrap_or(0);

        let mut raw: Vec<(Vec<u8>, bool)> = shard_pairs
            .iter()
            .map(|(opt, present)| {
                let data = opt.as_ref().map(|s| s.data.clone()).unwrap_or_else(|| vec![0u8; shard_size]);
                (data, *present)
            })
            .collect();

        self.inner.reconstruct(&mut raw).map_err(|e| TaoError::QiState(e.to_string()))?;

        let mut full = Vec::with_capacity(k * shard_size);
        for i in 0..k {
            full.extend_from_slice(&raw[i].0);
        }

        // 截断到原始长度，再验证哈希
        let result = full[..original_len].to_vec();
        let recovered_hash: [u8; 32] = Sha256::digest(&result).into();
        let expected_hash = shard_pairs.iter().find_map(|(s, _)| s.as_ref().map(|sh| sh.original_hash));
        if let Some(h) = expected_hash {
            if recovered_hash != h {
                return Err(TaoError::HashMismatch {
                    expected: hex::encode(h),
                    actual: hex::encode(recovered_hash),
                });
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_no_loss() {
        let data = "万物负阴而抱阳，冲气以为和。道生一，一生二，二生三，三生万物。".as_bytes();
        let encoder = ErasureEncoder::new(6, 2);
        let shards = encoder.encode(data).expect("encode");
        assert_eq!(shards.len(), 8);

        let pairs: Vec<(Option<Shard>, bool)> = shards.into_iter().map(|s| (Some(s), true)).collect();
        let recovered = encoder.decode(&pairs).expect("decode");
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_decode_with_parity_loss() {
        let data = "二生三——数据片与校验片共生共存".as_bytes();
        let encoder = ErasureEncoder::new(6, 2);
        let shards = encoder.encode(data).expect("encode");

        let pairs: Vec<(Option<Shard>, bool)> = shards.iter().enumerate().map(|(i, s)| {
            (Some(s.clone()), i < 6)
        }).collect();

        let recovered = encoder.decode(&pairs).expect("decode with parity loss");
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_decode_with_data_loss() {
        let data = "阴中有阳，阳中有阴——数据与校验不可分割".as_bytes();
        let encoder = ErasureEncoder::new(6, 2);
        let shards = encoder.encode(data).expect("encode");

        let pairs: Vec<(Option<Shard>, bool)> = (0..8).map(|i| {
            (Some(shards[i].clone()), i >= 2)
        }).collect();

        let recovered = encoder.decode(&pairs).expect("decode with data loss");
        assert_eq!(recovered, data);
    }

    #[test]
    fn test_insufficient_shards() {
        let data = "不足以恢复".as_bytes();
        let encoder = ErasureEncoder::new(6, 2);
        let shards = encoder.encode(data).expect("encode");

        let pairs: Vec<(Option<Shard>, bool)> = (0..8).map(|i| {
            (Some(shards[i].clone()), i < 4)
        }).collect();

        assert!(encoder.decode(&pairs).is_err());
    }
}

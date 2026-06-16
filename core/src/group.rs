use std::collections::HashMap;

use crate::erasure::{ErasureEncoder, Shard};
use crate::error::TaoResult;
use crate::unit::DataUnit;

/// 纠删码组 — 管理一份数据的所有分片
pub struct ErasureGroup {
    pub content_hash: [u8; 32],
    encoder: ErasureEncoder,
    shard_placement: HashMap<usize, String>,
    local_shards: Vec<Shard>,
}

impl ErasureGroup {
    pub fn from_unit(unit: &DataUnit, data_shards: usize, parity_shards: usize) -> TaoResult<Self> {
        let encoder = ErasureEncoder::new(data_shards, parity_shards);
        let shards = encoder.encode(&unit.yin.payload)?;

        Ok(Self {
            content_hash: unit.yin.content_hash,
            encoder,
            shard_placement: HashMap::new(),
            local_shards: shards,
        })
    }

    pub fn total_shards(&self) -> usize { self.local_shards.len() }
    pub fn data_shards(&self) -> usize { self.encoder.data_shards() }
    pub fn parity_shards(&self) -> usize { self.encoder.parity_shards() }

    pub fn get_shard(&self, index: usize) -> Option<&Shard> {
        self.local_shards.get(index)
    }

    pub fn place_shard(&mut self, index: usize, node_id: &str) {
        self.shard_placement.insert(index, node_id.to_string());
    }

    pub fn shard_node(&self, index: usize) -> Option<&str> {
        self.shard_placement.get(&index).map(|s| s.as_str())
    }

    pub fn placement_map(&self) -> &HashMap<usize, String> {
        &self.shard_placement
    }

    /// 从可用分片恢复原始数据
    pub fn reconstruct(&self, available_indices: &[usize]) -> TaoResult<Vec<u8>> {
        let total = self.total_shards();
        let pairs: Vec<(Option<Shard>, bool)> = (0..total)
            .map(|i| {
                let present = available_indices.contains(&i);
                (self.local_shards.get(i).cloned(), present)
            })
            .collect();

        self.encoder.decode(&pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::DataUnit;

    #[test]
    fn test_group_create_and_reconstruct() {
        let owner = [0u8; 32];
        let unit = DataUnit::new("分布式纠删码组测试数据".as_bytes().to_vec(), "test".into(), owner);
        let group = ErasureGroup::from_unit(&unit, 6, 2).expect("create group");

        assert_eq!(group.total_shards(), 8);
        assert_eq!(group.data_shards(), 6);
        assert_eq!(group.parity_shards(), 2);

        let recovered = group.reconstruct(&[0, 1, 2, 3, 4, 5, 6, 7]).expect("reconstruct");
        assert_eq!(recovered, unit.yin.payload);
    }

    #[test]
    fn test_group_lossy_reconstruct() {
        let owner = [0u8; 32];
        let unit = DataUnit::new("容错重建——丢失两片仍可恢复".as_bytes().to_vec(), "fault".into(), owner);
        let group = ErasureGroup::from_unit(&unit, 6, 2).expect("create group");

        let recovered = group.reconstruct(&[0, 1, 2, 3, 4, 5]).expect("reconstruct 6/8");
        assert_eq!(recovered, unit.yin.payload);
    }

    #[test]
    fn test_shard_placement() {
        let owner = [0u8; 32];
        let unit = DataUnit::new("分片放置".as_bytes().to_vec(), "place".into(), owner);
        let mut group = ErasureGroup::from_unit(&unit, 6, 2).expect("create group");

        group.place_shard(0, "node-a");
        group.place_shard(6, "node-c");

        assert_eq!(group.shard_node(0), Some("node-a"));
        assert_eq!(group.shard_node(6), Some("node-c"));
        assert_eq!(group.shard_node(99), None);
    }
}

use std::future::Future;

impl ErasureGroup {
    /// 分布式重建：从远程节点获取分片并解码
    ///
    /// fetch 回调：(shard_index) → 从远程节点获取 Shard 的异步函数。
    /// 调用方负责按需并行拉取。
    pub async fn distributed_reconstruct<F, Fut>(
        &self,
        available_indices: &[usize],
        mut fetch: F,
    ) -> TaoResult<Vec<u8>>
    where
        F: FnMut(usize) -> Fut,
        Fut: Future<Output = Option<Vec<u8>>>,
    {
        let total = self.total_shards();
        let mut shard_data: Vec<Option<Vec<u8>>> = vec![None; total];

        // 先填本地已有的
        for &idx in available_indices {
            if let Some(s) = self.local_shards.get(idx) {
                shard_data[idx] = Some(s.data.clone());
            }
        }

        // 拉取缺失的分片
        let k = self.data_shards();
        let mut present_count = shard_data.iter().filter(|s| s.is_some()).count();

        for i in 0..total {
            if present_count >= k { break; }
            if shard_data[i].is_some() { continue; }

            if let Some(data) = fetch(i).await {
                shard_data[i] = Some(data);
                present_count += 1;
            }
        }

        // 构造 (Option<Shard>, bool) 对
        let pairs: Vec<(Option<Shard>, bool)> = (0..total)
            .map(|i| {
                let shard = shard_data[i].as_ref().map(|data| Shard {
                    original_len: 0,
                    index: i,
                    data_count: self.data_shards(),
                    parity_count: self.parity_shards(),
                    original_hash: self.content_hash,
                    data: data.clone(),
                });
                (shard, shard_data[i].is_some())
            })
            .collect();

        let present_total = pairs.iter().filter(|(_, p)| *p).count();
        if present_total < k {
            return Err(crate::error::TaoError::InsufficientFragments {
                have: present_total, need: k,
            });
        }

        // 创建临时编码器解码
        let encoder = ErasureEncoder::new(self.data_shards(), self.parity_shards());
        encoder.decode(&pairs)
    }
}

use serde::{Deserialize, Serialize};

/// Gossipsub 传播消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaoMessage {
    StoreRequest { request_id: u64, data: Vec<u8>, target_replicas: u8 },
    StoreResponse { request_id: u64, accepted: bool, node_id: String },
    RetrieveRequest { request_id: u64, content_hash: [u8; 32] },
    RetrieveResponse { request_id: u64, data: Option<Vec<u8>> },
    Ping { nonce: u64 },
    Pong { nonce: u64, healthy: bool },
    Challenge { challenge: Vec<u8> },
    ChallengeResponse { proof: Vec<u8> },
    QiGossip { data: Vec<u8> },
    /// 请求某个分片（Gossipsub 广播）
    NeedShard { content_hash: [u8; 32], shard_index: usize },
}

/// 统一的点到点请求（RequestResponse 协议用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaoRequest {
    /// 存储分片
    Store(TaoStorePayload),
    /// 检索分片
    Retrieve(TaoRetrievePayload),
}

/// 统一的点到点响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaoResponse {
    /// 存储确认
    StoreAck(TaoStoreAck),
    /// 检索结果
    RetrieveData(TaoRetrieveData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaoStorePayload {
    pub content_hash: [u8; 32],
    pub shard_index: usize,
    pub shard_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaoStoreAck {
    pub accepted: bool,
    pub stored_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaoRetrievePayload {
    pub content_hash: [u8; 32],
    pub shard_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaoRetrieveData {
    pub found: bool,
    pub shard_data: Vec<u8>,
}

pub const PROTOCOL_NAME: &str = "/tao/storage/1.0.0";

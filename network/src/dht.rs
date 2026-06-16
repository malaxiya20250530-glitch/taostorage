use libp2p::kad::RecordKey;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// "名实分离" — Kademlia DHT 桥接层
pub struct TaoDht {
    name_cache: HashMap<String, [u8; 32]>,
}

/// DHT 中存储的条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtEntry {
    pub content_hash: [u8; 32],
    pub data_shards: u8,
    pub parity_shards: u8,
    pub registrar: String,
}

impl TaoDht {
    pub fn new() -> Self {
        Self { name_cache: HashMap::new() }
    }

    pub fn name_key(name: &str) -> RecordKey {
        let hash = Sha256::digest(format!("tao:name:{}", name).as_bytes());
        RecordKey::new(&hash)
    }

    pub fn content_key(content_hash: &[u8; 32]) -> RecordKey {
        RecordKey::new(content_hash)
    }

    pub fn make_register_record(
        name: &str, content_hash: &[u8; 32],
        data_shards: u8, parity_shards: u8, registrar: &PeerId,
    ) -> (RecordKey, Vec<u8>) {
        let key = Self::name_key(name);
        let entry = DhtEntry {
            content_hash: *content_hash, data_shards, parity_shards,
            registrar: registrar.to_base58(),
        };
        let value = bincode::serialize(&entry).unwrap_or_default();
        (key, value)
    }

    pub fn parse_entry(data: &[u8]) -> Option<DhtEntry> {
        bincode::deserialize(data).ok()
    }

    pub fn cache_name(&mut self, name: &str, content_hash: &[u8; 32]) {
        self.name_cache.insert(name.to_string(), *content_hash);
    }

    pub fn resolve_cached(&self, name: &str) -> Option<&[u8; 32]> {
        self.name_cache.get(name)
    }
}

impl Default for TaoDht {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_key_is_deterministic() {
        let k1 = TaoDht::name_key("道德经");
        let k2 = TaoDht::name_key("道德经");
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_name_key_differs() {
        assert_ne!(TaoDht::name_key("道"), TaoDht::name_key("德"));
    }

    #[test]
    fn test_entry_roundtrip() {
        let peer = PeerId::random();
        let hash = [0x42u8; 32];
        let (_key, value) = TaoDht::make_register_record("test", &hash, 6, 2, &peer);
        let entry = TaoDht::parse_entry(&value).expect("parse");
        assert_eq!(entry.content_hash, hash);
        assert_eq!(entry.data_shards, 6);
    }

    #[test]
    fn test_cache_hit() {
        let mut dht = TaoDht::new();
        let hash = [0x42u8; 32];
        dht.cache_name("test", &hash);
        assert_eq!(dht.resolve_cached("test"), Some(&hash));
    }
}

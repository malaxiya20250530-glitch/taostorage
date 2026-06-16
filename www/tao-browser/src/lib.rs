// ============================================================
// 🦀 TaoStorage WASM Browser Node
// "道可道，非常道" — 浏览器中的 P2P 存储节点
// ============================================================

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;

// ============================================================
// 核心数据模型（与 Rust 版一致）
// ============================================================

/// 数据单元的生命周期阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hexagram {
    Zhun,   // 屯 — 初生
    Jiji,   // 既济 — 功成
    Tai,    // 泰 — 通泰（热数据）
    Pi,     // 否 — 闭塞（冷数据）
    Bo,     // 剥 — 剥落（紧急修复）
    Kun,    // 坤 — 归藏（已归档）
}

/// 阴 — 数据本体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Yin {
    pub payload: Vec<u8>,
    pub content_hash: String, // hex 编码
}

/// 阳 — 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Yang {
    pub name: String,
    pub created_at: u64,
    pub tags: Vec<String>,
    pub heat: u8,
}

/// 气 — 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qi {
    pub hexagram: Hexagram,
    pub replica_count: u8,
}

/// DataUnit — 阴阳气三位一体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataUnit {
    pub yin: Yin,
    pub yang: Yang,
    pub qi: Qi,
}

impl DataUnit {
    pub fn new(payload: Vec<u8>, name: String) -> Self {
        let hash = Sha256::digest(&payload);
        let content_hash = hex::encode(hash);
        let now = js_sys::Date::now() as u64 / 1000;

        Self {
            yin: Yin { payload, content_hash },
            yang: Yang {
                name,
                created_at: now,
                tags: vec![],
                heat: 0,
            },
            qi: Qi {
                hexagram: Hexagram::Zhun,
                replica_count: 1,
            },
        }
    }

    pub fn id(&self) -> String {
        bs58::encode(hex::decode(&self.yin.content_hash).unwrap_or_default()).into_string()
    }

    pub fn verify(&self) -> bool {
        let hash = Sha256::digest(&self.yin.payload);
        hex::encode(hash) == self.yin.content_hash
    }
}

// ============================================================
// WASM 导出接口
// ============================================================

#[wasm_bindgen]
pub struct TaoBrowserNode {
    node_id: String,
    storage: Vec<DataUnit>,
    peers: Vec<String>,
    connected: bool,
}

#[wasm_bindgen]
impl TaoBrowserNode {
    /// 创建一个新的浏览器节点
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console::log_1(&"🦀 TaoStorage 浏览器节点启动中...".into());

        // 生成随机节点 ID
        let node_id = format!(
            "tao-browser-{:04x}",
            js_sys::Math::random() as u32
        );

        Self {
            node_id,
            storage: vec![],
            peers: vec![],
            connected: false,
        }
    }

    /// 获取节点 ID
    pub fn get_node_id(&self) -> String {
        self.node_id.clone()
    }

    /// 写入数据
    pub fn put(&mut self, key: String, value: String, tags: JsValue) -> Result<String, JsValue> {
        let tags: Vec<String> = serde_wasm_bindgen::from_value(tags).unwrap_or_default();
        let mut unit = DataUnit::new(value.into_bytes(), key.clone());
        unit.yang.tags = tags;

        let id = unit.id().clone();
        self.storage.push(unit);
        console::log_2(&"📝 已写入:".into(), &key.into());
        Ok(id)
    }

    /// 读取数据
    pub fn get(&self, key: String) -> Result<JsValue, JsValue> {
        for unit in &self.storage {
            if unit.yang.name == key {
                let json = serde_json::to_string(&unit).unwrap_or_default();
                return Ok(JsValue::from_str(&json));
            }
        }
        Err(JsValue::from_str(&format!("Key '{}' 未找到", key)))
    }

    /// 列出所有数据
    pub fn list(&self) -> Result<JsValue, JsValue> {
        let items: Vec<serde_json::Value> = self.storage.iter().map(|u| {
            serde_json::json!({
                "key": u.yang.name,
                "tags": u.yang.tags,
                "heat": u.yang.heat,
                "hexagram": format!("{:?}", u.qi.hexagram),
                "id": u.id(),
            })
        }).collect();

        Ok(serde_wasm_bindgen::to_value(&items).unwrap_or(JsValue::NULL))
    }

    /// 搜索
    pub fn search(&self, query: String) -> Result<JsValue, JsValue> {
        let q = query.to_lowercase();
        let results: Vec<serde_json::Value> = self.storage.iter().filter(|u| {
            let name = u.yang.name.to_lowercase();
            let payload = String::from_utf8_lossy(&u.yin.payload).to_lowercase();
            let tags: String = u.yang.tags.join(" ").to_lowercase();
            name.contains(&q) || payload.contains(&q) || tags.contains(&q)
        }).map(|u| {
            serde_json::json!({
                "key": u.yang.name,
                "value": String::from_utf8_lossy(&u.yin.payload).to_string(),
                "tags": u.yang.tags,
            })
        }).collect();

        Ok(serde_wasm_bindgen::to_value(&results).unwrap_or(JsValue::NULL))
    }

    /// 获取统计
    pub fn stats(&self) -> Result<JsValue, JsValue> {
        let stats = serde_json::json!({
            "node_id": self.node_id,
            "objects": self.storage.len(),
            "peers": self.peers.len(),
            "connected": self.connected,
            "tags": self.get_all_tags(),
        });
        Ok(serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL))
    }

    fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.storage.iter()
            .flat_map(|u| u.yang.tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// 连接信令服务器
    pub async fn connect_signaling(&mut self, url: String) -> Result<JsValue, JsValue> {
        console::log_2(&"🌐 连接信令服务器:".into(), &url.into());

        let ws = web_sys::WebSocket::new(&url)?;
        let node_id = self.node_id.clone();

        // 注册事件回调
        let cloned_ws = ws.clone();
        let onopen_cb = Closure::<dyn Fn()>::new(move || {
            console::log_1(&"✅ WebSocket 已连接".into());
            // 发送注册消息
            let msg = serde_json::json!({
                "type": "register",
                "node_id": node_id,
                "role": "browser"
            });
            cloned_ws.send_with_str(&msg.to_string()).ok();
        });
        ws.set_onopen(Some(onopen_cb.as_ref().unchecked_ref()));
        onopen_cb.forget();

        self.connected = true;
        Ok(serde_wasm_bindgen::to_value(&serde_json::json!({
            "status": "connected",
            "node_id": self.node_id
        })).unwrap_or(JsValue::NULL))
    }

    /// 导出备份
    pub fn export_backup(&self) -> Result<JsValue, JsValue> {
        let json = serde_json::to_string(&self.storage).unwrap_or("[]".into());
        Ok(JsValue::from_str(&json))
    }

    /// 导入备份
    pub fn import_backup(&mut self, json: String) -> usize {
        if let Ok(units) = serde_json::from_str::<Vec<DataUnit>>(&json) {
            let count = units.len();
            self.storage.extend(units);
            console::log_2(&"📦 导入:".into(), &count.into());
            count
        } else {
            0
        }
    }
}

// ============================================================
// 工具函数
// ============================================================

/// 计算 SHA256 哈希
#[wasm_bindgen]
pub fn sha256_hex(data: &str) -> String {
    let hash = Sha256::digest(data.as_bytes());
    hex::encode(hash)
}

/// 生成节点 ID
#[wasm_bindgen]
pub fn generate_node_id() -> String {
    format!(
        "tao-{:08x}",
        (js_sys::Math::random() * u32::MAX as f64) as u32
    )
}

/// 验证数据完整性
#[wasm_bindgen]
pub fn verify_content(payload: &str, hash: &str) -> bool {
    let actual = sha256_hex(payload);
    actual == hash
}

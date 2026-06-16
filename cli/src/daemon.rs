use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, watch, oneshot, Mutex as TokioMutex};
use tracing::{info, warn};

use tao_core::*;
use tao_network::{NetworkCommand, NetworkEvent, TaoNetwork};

// ============================================================
// JSON 模型
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PutRequest { pub key: String, pub value: String, #[serde(default)] pub tags: Vec<String> }
#[derive(Debug, Serialize, Deserialize)]
pub struct PutResponse { pub id: String, pub key: String }
#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    pub id: String, pub key: String, pub value: String,
    pub tags: Vec<String>, pub heat: u8, pub hexagram: String, pub created_at: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRequest { pub key: String, #[serde(default)] pub id: Option<String> }
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse { pub deleted: usize }
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchParams { pub q: String, #[serde(default = "default_limit")] pub limit: usize }
fn default_limit() -> usize { 20 }
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TagQuery { pub tags: Option<Vec<String>>, #[serde(default)] pub all_mode: bool }
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String, pub node_id: String, pub api_port: u16, pub p2p: String,
    pub objects: usize, pub peers: usize, pub uptime_secs: u64, pub version: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub running: bool, pub pid: Option<u32>, pub node_id: Option<String>,
    pub data_dir: Option<String>, pub api_port: Option<u16>,
    pub peers: Option<usize>, pub objects: Option<usize>, pub uptime_secs: Option<u64>,
}

// ============================================================
// Daemon 状态
// ============================================================

pub struct DaemonState {
    pub data_dir: PathBuf,
    pub store: Arc<sled::Db>,
    pub tag_index: Arc<TagIndex>,
    pub p2p_net: Option<Arc<RwLock<TaoNetwork>>>,
    pub node_id: Option<String>,
    pub started_at: u64,
    pub api_port: u16,
    pub peer_count_rx: Option<watch::Receiver<usize>>,
    /// content_hash → 等待 P2P 检索结果的 oneshot 发送者
    pub pending_retrievals: Arc<TokioMutex<HashMap<[u8; 32], Vec<oneshot::Sender<Result<Vec<u8>, String>>>>>>,
    /// key 名称 → 等待 DHT 名称解析结果的 oneshot 发送者
    pub pending_names: Arc<TokioMutex<HashMap<String, Vec<oneshot::Sender<Result<[u8; 32], String>>>>>>,
}

type ApiError = (StatusCode, String);
type ApiResult<T> = Result<Json<T>, ApiError>;

// ============================================================
// 存储辅助
// ============================================================

fn store_get(store: &sled::Db, hash: &[u8; 32]) -> Result<Option<DataUnit>, ApiError> {
    match store.get(hash).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? {
        Some(data) => {
            let u: DataUnit = bincode::deserialize(&data)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            Ok(Some(u))
        }
        None => Ok(None),
    }
}

fn store_put(store: &sled::Db, tag_index: &TagIndex, unit: &DataUnit) -> Result<(), ApiError> {
    let bytes = bincode::serialize(unit)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    store.insert(&unit.yin.content_hash, bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !unit.yang.tags.is_empty() {
        tag_index.add_tags_bidirectional(&unit.yin.content_hash, &unit.yang.tags)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    Ok(())
}

fn to_resp(u: DataUnit) -> GetResponse {
    let v = String::from_utf8_lossy(&u.yin.payload).to_string();
    GetResponse { id: u.id(), key: u.yang.name, value: v, tags: u.yang.tags,
        heat: u.yang.heat, hexagram: format!("{:?}", u.qi.hexagram), created_at: u.yang.created_at }
}

fn scan(store: &sled::Db, filter: Option<&str>) -> Result<Vec<DataUnit>, ApiError> {
    let mut r = Vec::new();
    for item in store.iter() {
        let (_k, v) = item.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if let Ok(u) = bincode::deserialize::<DataUnit>(&v) {
            if let Some(f) = filter { if u.yang.name != f { continue; } }
            r.push(u);
        }
    }
    r.sort_by(|a, b| b.yang.created_at.cmp(&a.yang.created_at));
    Ok(r)
}

#[derive(Deserialize, Default)]
struct KeyOpt { key: Option<String> }

// ============================================================
// HTTP 路由
// ============================================================

pub fn build_router(state: Arc<DaemonState>) -> Router {
    Router::new()
        .route("/v1/health", get(health_handler))
        .route("/v1/memory", put(put_handler))
        .route("/v1/memory/{id}", get(get_handler))
        .route("/v1/memory", get(list_handler))
        .route("/v1/memory", axum::routing::delete(delete_handler))
        .route("/v1/search", get(search_handler))
        .route("/v1/tags/{tag}", get(by_tag_handler))
        .route("/v1/tag-cloud", get(tag_cloud_handler))
        .route("/v1/stats", get(stats_handler))
        .route("/v1/peers", get(peers_handler))
        .route("/v1/export", get(export_handler))
        .route("/v1/import", put(import_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

// ============================================================
// HTTP Handlers
// ============================================================

async fn health_handler(State(s): State<Arc<DaemonState>>) -> Json<HealthResponse> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let peers = s.peer_count_rx.as_ref().map(|rx| *rx.borrow()).unwrap_or(0);
    let p2p = if s.p2p_net.is_some() { "enabled" } else { "disabled" };
    Json(HealthResponse {
        status: "ok".into(), node_id: s.node_id.clone().unwrap_or_default(),
        api_port: s.api_port, p2p: p2p.into(),
        objects: s.store.iter().count(), peers, uptime_secs: now.saturating_sub(s.started_at),
        version: "0.3.0".into(),
    })
}

async fn put_handler(State(s): State<Arc<DaemonState>>, Json(req): Json<PutRequest>) -> ApiResult<PutResponse> {
    let mut u = DataUnit::new(req.value.as_bytes().to_vec(), req.key.clone(), [0u8; 32]);
    u.yang.tags = req.tags.clone();
    let ch = u.yin.content_hash;
    store_put(&s.store, &s.tag_index, &u)?;
    s.store.flush().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // P2P 广播：提供内容 + 注册 key → content_hash
    if let Some(ref net) = s.p2p_net {
        let net = net.read().await;
        let _ = net.send_command(NetworkCommand::ProvideContent { content_hash: ch }).await;
        let _ = net.send_command(NetworkCommand::RegisterName {
            name: req.key.clone(), content_hash: ch, data_shards: 6, parity_shards: 2,
        }).await;
        let _ = net.send_command(NetworkCommand::GossipQi { data: ch.to_vec() }).await;
    }

    Ok(Json(PutResponse { id: u.id(), key: req.key }))
}

async fn get_handler(State(s): State<Arc<DaemonState>>, AxumPath(id): AxumPath<String>) -> ApiResult<GetResponse> {
    // 1. 按 content_hash 本地查找
    if let Ok(bytes) = hex::decode(&id) { if bytes.len() == 32 {
        let mut h = [0u8; 32]; h.copy_from_slice(&bytes);
        if let Some(u) = store_get(&s.store, &h)? { return Ok(Json(to_resp(u))); }
    }}
    // 2. 按 key 本地查找
    if let Some(u) = scan(&s.store, Some(&id))?.into_iter().next() { return Ok(Json(to_resp(u))); }

    // 3. P2P 跨节点检索
    if let Some(ref net) = s.p2p_net {
        // 判断 id 是 content_hash 还是 key 名称
        let is_hash = hex::decode(&id).map(|b| b.len() == 32).unwrap_or(false);

        if is_hash {
            // 按 content_hash 检索
            let bytes = hex::decode(&id).unwrap();
            let mut ch = [0u8; 32]; ch.copy_from_slice(&bytes);
            let (tx, rx) = oneshot::channel();
            s.pending_retrievals.lock().await.entry(ch).or_default().push(tx);
            net.read().await.send_command(NetworkCommand::FindProviders { content_hash: ch }).await.ok();
            info!("🔍 P2P by hash: {}...", &id[..12]);

            // timeout → oneshot::Receiver → Result<Vec<u8>, String>
            if let Ok(Ok(Ok(data))) = tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
                let unit = DataUnit::new(data.clone(), format!("p2p_{}", &id[..8]), [0u8; 32]);
                store_put(&s.store, &s.tag_index, &unit).ok();
                s.store.flush().ok();
                return Ok(Json(GetResponse {
                    id: id.clone(), key: format!("p2p_{}", &id[..8]),
                    value: String::from_utf8_lossy(&data).to_string(),
                    tags: vec![], heat: 0, hexagram: "P2P".into(), created_at: 0,
                }));
            }
        } else {
            // 按 key 名称检索：先 DHT 解析
            let name = id.clone();
            let (name_tx, name_rx) = oneshot::channel();
            s.pending_names.lock().await.entry(name.clone()).or_default().push(name_tx);
            net.read().await.send_command(NetworkCommand::ResolveName { name: name.clone() }).await.ok();
            info!("🔍 DHT resolve: {}", &name);

            // timeout → oneshot → Result<[u8; 32], String>
            if let Ok(Ok(Ok(ch))) = tokio::time::timeout(std::time::Duration::from_secs(5), name_rx).await {
                // DHT 解析成功，用 content_hash 做 P2P 检索
                let (tx, rx) = oneshot::channel();
                s.pending_retrievals.lock().await.entry(ch).or_default().push(tx);
                net.read().await.send_command(NetworkCommand::FindProviders { content_hash: ch }).await.ok();
                info!("🔍 P2P by name: {}", hex::encode(&ch[..8]));

                if let Ok(Ok(Ok(data))) = tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
                    let unit = DataUnit::new(data.clone(), name.clone(), [0u8; 32]);
                    store_put(&s.store, &s.tag_index, &unit).ok();
                    s.store.flush().ok();
                    return Ok(Json(GetResponse {
                        id: hex::encode(ch), key: name.clone(),
                        value: String::from_utf8_lossy(&data).to_string(),
                        tags: vec![], heat: 0, hexagram: "P2P".into(), created_at: 0,
                    }));
                }
            }
        }
    }

    Err((StatusCode::NOT_FOUND, id))
}

async fn list_handler(State(s): State<Arc<DaemonState>>, Query(p): Query<KeyOpt>) -> ApiResult<Vec<GetResponse>> {
    Ok(Json(scan(&s.store, p.key.as_deref())?.into_iter().map(to_resp).collect()))
}

async fn delete_handler(State(s): State<Arc<DaemonState>>, Json(req): Json<DeleteRequest>) -> ApiResult<DeleteResponse> {
    let mut n = 0;
    if let Some(ref id_hex) = req.id {
        if let Ok(bytes) = hex::decode(id_hex) { if bytes.len() == 32 {
            let mut h = [0u8; 32]; h.copy_from_slice(&bytes);
            s.tag_index.remove_hash(&h).ok();
            s.store.remove(&h).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            n = 1;
        }}
    } else {
        for u in scan(&s.store, Some(&req.key))? {
            s.tag_index.remove_hash(&u.yin.content_hash).ok();
            s.store.remove(&u.yin.content_hash).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            n += 1;
        }
    }
    s.store.flush().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(DeleteResponse { deleted: n }))
}

async fn search_handler(State(s): State<Arc<DaemonState>>, Query(p): Query<SearchParams>) -> ApiResult<Vec<GetResponse>> {
    let hits = fuzzy_search(&s.store, &p.q, p.limit).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut r = Vec::new();
    for hit in &hits {
        if let Ok(bytes) = hex::decode(&hit.content_hash) { if bytes.len() == 32 {
            let mut h = [0u8; 32]; h.copy_from_slice(&bytes);
            if let Some(u) = store_get(&s.store, &h)? { r.push(to_resp(u)); }
        }}
    }
    Ok(Json(r))
}

async fn by_tag_handler(State(s): State<Arc<DaemonState>>, AxumPath(tag): AxumPath<String>, Query(p): Query<TagQuery>) -> ApiResult<Vec<GetResponse>> {
    let mut tags = vec![tag];
    if let Some(ref x) = p.tags { tags.extend(x.clone()); }
    let hexes = (if p.all_mode { s.tag_index.get_by_tags_all(&tags) } else { s.tag_index.get_by_tags_any(&tags) })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut r = Vec::new();
    for h in &hexes {
        if let Ok(bytes) = hex::decode(h) { if bytes.len() == 32 {
            let mut ha = [0u8; 32]; ha.copy_from_slice(&bytes);
            if let Some(u) = store_get(&s.store, &ha)? { r.push(to_resp(u)); }
        }}
    }
    Ok(Json(r))
}

async fn tag_cloud_handler(State(s): State<Arc<DaemonState>>) -> ApiResult<Vec<(String, usize)>> {
    Ok(Json(s.tag_index.tag_cloud().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn stats_handler(State(s): State<Arc<DaemonState>>) -> ApiResult<StoreStats> {
    Ok(Json(collect_stats(&s.store, &s.tag_index, 10).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn export_handler(State(s): State<Arc<DaemonState>>) -> ApiResult<serde_json::Value> {
    let j = export_backup(&s.store, &s.tag_index).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::from_str(&j).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn import_handler(State(s): State<Arc<DaemonState>>, Json(body): Json<serde_json::Value>) -> ApiResult<ImportResult> {
    let j = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(import_backup(&s.store, &s.tag_index, &j).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn peers_handler(State(s): State<Arc<DaemonState>>) -> Json<Vec<String>> {
    if s.p2p_net.is_some() {
        let n = s.peer_count_rx.as_ref().map(|rx| *rx.borrow()).unwrap_or(0);
        Json(vec![format!("Peers: {}", n), format!("Objects: {}", s.store.iter().count())])
    } else {
        Json(vec!["P2P: offline".to_string()])
    }
}

// ============================================================
// Daemon 启动
// ============================================================

pub async fn run_daemon(
    data_dir: &str, _listen: &str, api_port: u16, bootstrap: &[String], enable_network: bool,
) -> anyhow::Result<()> {
    let dp = PathBuf::from(data_dir);
    std::fs::create_dir_all(&dp)?;
    std::fs::write(dp.join("daemon.pid"), std::process::id().to_string())?;

    let store = Arc::new(sled::open(dp.join("store"))?);
    let tag_index = Arc::new(TagIndex::open(dp.join("tags"))?);
    let started_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

    let pending_retrievals: Arc<TokioMutex<HashMap<[u8; 32], Vec<oneshot::Sender<Result<Vec<u8>, String>>>>>> =
        Arc::new(TokioMutex::new(HashMap::new()));
    let pending_names: Arc<TokioMutex<HashMap<String, Vec<oneshot::Sender<Result<[u8; 32], String>>>>>> =
        Arc::new(TokioMutex::new(HashMap::new()));

    let (p2p_net, node_id, peer_rx) = if enable_network {
        match start_p2p(_listen, bootstrap, store.clone(), pending_retrievals.clone(), pending_names.clone()).await {
            Ok((net, nid, rx)) => (Some(net), Some(nid), Some(rx)),
            Err(e) => {
                warn!("P2P 启动失败: {}", e);
                (None, Some(format!("local-{}", std::process::id())), None)
            }
        }
    } else {
        (None, Some(format!("local-{}", std::process::id())), None)
    };

    let state = Arc::new(DaemonState {
        data_dir: dp, store, tag_index, p2p_net, node_id,
        started_at, api_port, peer_count_rx: peer_rx,
        pending_retrievals, pending_names,
    });

    let app = build_router(state);
    let addr = format!("127.0.0.1:{}", api_port);
    println!("\n🌀 Tao daemon started");
    println!("   API:     http://{}", addr);
    println!("   Storage: {}", data_dir);
    println!("   Ctrl-C   to stop\n");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ============================================================
// P2P 事件循环（带跨节点检索 + 名称解析）
// ============================================================

async fn start_p2p(
    listen_addr: &str, bootstrap: &[String],
    store: Arc<sled::Db>,
    pending_retrievals: Arc<TokioMutex<HashMap<[u8; 32], Vec<oneshot::Sender<Result<Vec<u8>, String>>>>>>,
    pending_names: Arc<TokioMutex<HashMap<String, Vec<oneshot::Sender<Result<[u8; 32], String>>>>>>,
) -> anyhow::Result<(Arc<RwLock<TaoNetwork>>, String, watch::Receiver<usize>)> {
    let net = Arc::new(RwLock::new(TaoNetwork::new_with_listen(listen_addr).await?));
    let nid = net.read().await.local_peer_id.to_string();
    info!("🌐 P2P: {}", nid);

    let (peer_tx, peer_rx) = watch::channel(0usize);

    // 拨号引导节点
    for addr in bootstrap {
        info!("   Dial: {}", addr);
        net.read().await.send_command(NetworkCommand::Dial { addr: addr.clone() }).await.ok();
    }

    // P2P 事件循环
    let ev_net = net.clone();
    let ev_store = store;
    let ev_retrievals = pending_retrievals;
    let ev_names = pending_names;
    tokio::spawn(async move {
        let mut peer_count = 0usize;
        loop {
            let event = ev_net.write().await.recv_event().await;
            match event {
                Some(NetworkEvent::PeerDiscovered(_)) | Some(NetworkEvent::ListenAddr(_)) => {
                    if let Some(NetworkEvent::PeerDiscovered(_)) = event {
                        peer_count += 1; let _ = peer_tx.send(peer_count);
                    }
                }
                Some(NetworkEvent::PeerExpired(_)) => {
                    peer_count = peer_count.saturating_sub(1); let _ = peer_tx.send(peer_count);
                }
                Some(NetworkEvent::StoreRequest { peer, content_hash, shard_index, shard_data, .. }) => {
                    let name = format!("peered_{}_{}", hex::encode(&content_hash[..6]), shard_index);
                    if let Ok(bytes) = bincode::serialize(&DataUnit::new(shard_data, name, [0u8; 32])) {
                        let _ = ev_store.insert(&content_hash, bytes);
                        info!("📥 Shard[{}] from {}", shard_index, peer);
                    }
                }
                Some(NetworkEvent::RetrieveRequest { peer, request_id, content_hash, shard_index }) => {
                    let data = ev_store.get(&content_hash).ok().flatten().map(|d| d.to_vec()).unwrap_or_default();
                    let mut guard = ev_net.write().await;
                    let _ = guard.send_command(NetworkCommand::SendRetrieveResponse { request_id, shard_data: data }).await;
                    info!("📤 Shard[{}] → {}", shard_index, peer);
                }
                Some(NetworkEvent::ShardFetched { peer, content_hash, shard_index, shard_data }) => {
                    info!("📦 Shard[{}] from {} ({}B)", shard_index, peer, shard_data.len());
                    let key = format!("__shard_{}_{}", hex::encode(&content_hash[..6]), shard_index);
                    if let Ok(bytes) = bincode::serialize(&DataUnit::new(shard_data.clone(), key, [0u8; 32])) {
                        let _ = ev_store.insert(&content_hash, bytes);
                    }
                    // 检查是否能重建
                    let has_pending = ev_retrievals.lock().await.contains_key(&content_hash);
                    if has_pending {
                        let mut collected: Vec<(usize, Vec<u8>)> = vec![(shard_index, shard_data)];
                        for si in 0..8 {
                            let sk = format!("__shard_{}_{}", hex::encode(&content_hash[..6]), si);
                            if let Ok(Some(data)) = ev_store.get(&content_hash) {
                                if let Ok(su) = bincode::deserialize::<DataUnit>(&data) {
                                    if su.yang.name == sk && !collected.iter().any(|(i,_)| *i == si) {
                                        collected.push((si, su.yin.payload));
                                    }
                                }
                            }
                        }
                        if collected.len() >= 6 {
                            info!("🧩 Reconstructing from {} shards", collected.len());
                            let encoder = ErasureEncoder::new(6, 2);
                            let mut pairs = Vec::with_capacity(8);
                            for si in 0..8 {
                                if let Some((_, data)) = collected.iter().find(|(i,_)| *i == si) {
                                    pairs.push((Some(Shard {
                                        original_len: 0, index: si, data_count: 6, parity_count: 2,
                                        original_hash: content_hash, data: data.clone(),
                                    }), true));
                                } else { pairs.push((None, false)); }
                            }
                            match encoder.decode(&pairs) {
                                Ok(reconstructed) => {
                                    info!("✅ Reconstructed ({}B)", reconstructed.len());
                                    let mut pend = ev_retrievals.lock().await;
                                    if let Some(senders) = pend.remove(&content_hash) {
                                        for tx in senders { let _ = tx.send(Ok(reconstructed.clone())); }
                                    }
                                }
                                Err(e) => warn!("❌ Reconstruct failed: {}", e),
                            }
                        }
                    }
                }
                Some(NetworkEvent::ProvidersFound { content_hash, providers }) => {
                    info!("🔍 Providers for {}: {}", hex::encode(&content_hash[..8]), providers.len());
                    let guard = ev_net.write().await;
                    for p in &providers {
                        for si in 0..8 {
                            let _ = guard.send_command(NetworkCommand::FetchShard {
                                peer: *p, content_hash, shard_index: si,
                            }).await;
                        }
                    }
                }
                Some(NetworkEvent::NameResolved { name, entry }) => {
                    match entry {
                        Some(e) => {
                            info!("📇 '{}' → {}", name, hex::encode(&e.content_hash[..8]));
                            let mut pn = ev_names.lock().await;
                            if let Some(senders) = pn.remove(&name) {
                                for tx in senders { let _ = tx.send(Ok(e.content_hash)); }
                            }
                        }
                        None => {
                            info!("📇 '{}' not found", name);
                            let mut pn = ev_names.lock().await;
                            if let Some(senders) = pn.remove(&name) {
                                for tx in senders { let _ = tx.send(Err("not found".into())); }
                            }
                        }
                    }
                }
                Some(NetworkEvent::QueryTimeout { content_hash }) => {
                    info!("⏱️ Query timeout for {}", hex::encode(&content_hash[..8]));
                    let mut pend = ev_retrievals.lock().await;
                    if let Some(senders) = pend.remove(&content_hash) {
                        for tx in senders { let _ = tx.send(Err("timeout".into())); }
                    }
                }
                Some(_) => {}
                None => { info!("P2P 事件循环结束"); break; }
            }
        }
    });

    Ok((net, nid, peer_rx))
}

// ============================================================
// 生命周期管理
// ============================================================

pub fn check_status(data_dir: &str) -> StatusResponse {
    let pid_path = PathBuf::from(data_dir).join("daemon.pid");
    if !pid_path.exists() {
        return StatusResponse {
            running: false, pid: None, node_id: None, data_dir: Some(data_dir.into()),
            api_port: None, peers: None, objects: None, uptime_secs: None,
        };
    }
    match std::fs::read_to_string(&pid_path) {
        Ok(s) => {
            if let Ok(pid) = s.trim().parse::<u32>() {
                if Path::new(&format!("/proc/{}", pid)).exists() {
                    StatusResponse { running: true, pid: Some(pid), node_id: None, data_dir: Some(data_dir.into()), api_port: None, peers: None, objects: None, uptime_secs: None }
                } else {
                    let _ = std::fs::remove_file(&pid_path);
                    StatusResponse { running: false, pid: None, node_id: None, data_dir: Some(data_dir.into()), api_port: None, peers: None, objects: None, uptime_secs: None }
                }
            } else {
                StatusResponse { running: false, pid: None, node_id: None, data_dir: Some(data_dir.into()), api_port: None, peers: None, objects: None, uptime_secs: None }
            }
        }
        Err(_) => StatusResponse { running: false, pid: None, node_id: None, data_dir: Some(data_dir.into()), api_port: None, peers: None, objects: None, uptime_secs: None }
    }
}

pub fn stop_daemon(data_dir: &str) -> anyhow::Result<()> {
    let pid_path = PathBuf::from(data_dir).join("daemon.pid");
    if !pid_path.exists() { anyhow::bail!("Daemon 未运行"); }
    let pid: u32 = std::fs::read_to_string(&pid_path)?.trim().parse()?;
    unsafe { libc::kill(pid as i32, libc::SIGTERM); }
    for _ in 0..50 {
        if !Path::new(&format!("/proc/{}", pid)).exists() {
            let _ = std::fs::remove_file(&pid_path);
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    unsafe { libc::kill(pid as i32, libc::SIGKILL); }
    let _ = std::fs::remove_file(&pid_path);
    Ok(())
}

pub fn start_daemon_background(data_dir: &str, _listen: &str, api_port: u16, bootstrap: &[String]) -> anyhow::Result<()> {
    let dp = PathBuf::from(data_dir);
    std::fs::create_dir_all(&dp)?;
    if check_status(data_dir).running { anyhow::bail!("Daemon 已在运行"); }
    match unsafe { libc::fork() } {
        -1 => anyhow::bail!("fork 失败"),
        0 => {
            unsafe { libc::setsid(); }
            std::thread::sleep(std::time::Duration::from_millis(300));
            let mut args = vec!["--data-dir".into(), data_dir.into(), "daemon".into(), "start".into(), "--port".into(), api_port.to_string()];
            for b in bootstrap { args.push("--bootstrap".into()); args.push(b.clone()); }
            exec_self(&args);
            std::process::exit(1);
        }
        pid => {
            println!("🌀 启动 Daemon... (PID: {})", pid);
            std::thread::sleep(std::time::Duration::from_secs(2));
            if check_status(data_dir).running {
                println!("✅ Daemon 已启动 (PID: {}, API: http://127.0.0.1:{})", pid, api_port);
            } else {
                println!("⚠️ Daemon 可能未正常启动");
            }
        }
    }
    Ok(())
}

fn exec_self(args: &[String]) {
    use std::ffi::CString;
    if let Ok(exe) = std::env::current_exe() {
        let c = CString::new(exe.to_string_lossy().as_bytes()).unwrap_or_default();
        let mut raw = vec![c.as_ptr()];
        for a in args {
            if let Ok(ca) = CString::new(a.as_bytes()) { raw.push(ca.as_ptr()); std::mem::forget(ca); }
        }
        raw.push(std::ptr::null());
        unsafe { libc::execvp(c.as_ptr(), raw.as_ptr()); }
    }
}

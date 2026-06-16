use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use tao_core::{DataUnit, LocalStore, ErasureEncoder};
use tao_consensus::{QiConsensus, ReputationTable};
use tao_crypto::{encrypt, decrypt, generate_ed25519_keypair};
use tao_network::{
    NetworkCommand, NetworkEvent, TaoNetwork,
    TaoStoreAck,
};

#[derive(Parser, Debug)]
#[command(name = "tao-node", version = "0.1.0")]
struct Cli {
    #[arg(short = 'l', long, default_value = "/ip4/0.0.0.0/tcp/0")]
    listen: String,
    #[arg(short = 'b', long)]
    bootstrap: Vec<String>,
    #[arg(short = 'd', long, default_value = "~/.taostorage")]
    data_dir: String,
    #[arg(short = 'k', long)]
    key_file: Option<String>,
    #[arg(long, default_value = "6")]
    ec_data: usize,
    #[arg(long, default_value = "2")]
    ec_parity: usize,
    #[arg(long, default_value = "3")]
    distribute: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let data_dir = expand_tilde(&cli.data_dir);
    let data_path = PathBuf::from(&data_dir);
    std::fs::create_dir_all(&data_path)?;
    tracing::info!("数据目录: {}", data_path.display());

    let keypair = load_or_generate_key(&cli.key_file, &data_path)?;
    let owner_pubkey: [u8; 32] = keypair.verifying_key().to_bytes();
    tracing::info!("身份: {}", hex::encode(&owner_pubkey[..8]));

    let store = LocalStore::open(data_path.join("storage"))?;

    let rep_table = ReputationTable::persistent(
        &format!("{}/reputation", &data_dir)
    ).unwrap_or_else(|_| ReputationTable::new());
    let _consensus = QiConsensus::with_reputation(rep_table, 15);

    tracing::info!("启动 P2P 网络...");
    let mut network = TaoNetwork::new().await?;
    let peer_id = network.local_peer_id;
    tracing::info!("PeerId: {}", peer_id);

    // === DHT 引导：拨号已知节点 ===
    for addr in &cli.bootstrap {
        tracing::info!("拨号引导节点: {}", addr);
        network.send_command(NetworkCommand::Dial { addr: addr.clone() }).await.ok();
    }

    // 写入示例数据
    let content = "道可道，非常道；名可名，非常名。".as_bytes();
    let encrypted = encrypt(content, &[0x42u8; 32]);
    let unit = DataUnit::new(encrypted.clone(), "道德经·第一章".to_string(), owner_pubkey);
    tracing::info!("写入: id={}", unit.id());
    store.store(&unit)?;

    let encoder = ErasureEncoder::new(cli.ec_data, cli.ec_parity);
    let shards = encoder.encode(&unit.yin.payload)?;
    tracing::info!("RS({}+{}) → {} 片", cli.ec_data, cli.ec_parity, shards.len());

    // 分片索引缓存：(content_hash, shard_index) → DataUnit id
    let mut shard_lookup: HashMap<([u8; 32], usize), String> = HashMap::new();

    let mut sent = 0;
    let max_send = cli.distribute;
    tracing::info!("等待对等节点... (Ctrl-C 退出)");

    loop {
        tokio::select! {
            ev = network.recv_event() => {
                match ev {
                    Some(NetworkEvent::PeerDiscovered(peer)) => {
                        if peer == network.local_peer_id { continue; }
                        tracing::info!("发现: {}", peer);
                        if sent < max_send {
                            let s = &shards[sent % shards.len()];
                            tracing::info!("→ 片[{}] ({}B)", s.index, s.data.len());
                            network.send_command(NetworkCommand::StoreShard {
                                peer, content_hash: s.original_hash,
                                shard_index: s.index, shard_data: s.data.clone(),
                            }).await.ok();
                            sent += 1;
                        }
                        network.send_command(NetworkCommand::ProvideContent {
                            content_hash: unit.yin.content_hash,
                        }).await.ok();
                    }

                    // === 收到存储请求 → 存入本地 + 索引 ===
                    Some(NetworkEvent::StoreRequest {
                        peer, content_hash, shard_index, shard_data, ..
                    }) => {
                        let name = format!("shard_{}_{}",
                            hex::encode(&content_hash[..6]), shard_index);
                        let su = DataUnit::new(shard_data.clone(), name.clone(), owner_pubkey);
                        let id = su.id();
                        store.store(&su).ok();
                        shard_lookup.insert((content_hash, shard_index), id.clone());
                        tracing::info!("存储分片: {} from {}", id, peer);
                    }

                    // === 收到检索请求 → 查本地索引 → 回复 ===
                    Some(NetworkEvent::RetrieveRequest {
                        peer, request_id, content_hash, shard_index,
                    }) => {
                        tracing::info!("检索请求: 片[{}] from {}", shard_index, peer);

                        let shard_data = shard_lookup
                            .get(&(content_hash, shard_index))
                            .and_then(|id_str| {
                                // 解析 content_hash → 从 store 读取
                                store.iter()
                                    .filter_map(|r| r.ok())
                                    .find(|u| u.id() == *id_str)
                                    .map(|u| u.yin.payload.clone())
                            });

                        match shard_data {
                            Some(data) => {
                                tracing::info!("→ 回复片[{}] ({} bytes)", shard_index, data.len());
                                network.send_command(NetworkCommand::SendRetrieveResponse {
                                    request_id,
                                    shard_data: data,
                                }).await.ok();
                            }
                            None => {
                                tracing::warn!("→ 片[{}] 未找到，回复空", shard_index);
                                network.send_command(NetworkCommand::SendRetrieveResponse {
                                    request_id,
                                    shard_data: vec![],
                                }).await.ok();
                            }
                        }
                    }

                    Some(NetworkEvent::ShardFetched { peer, shard_data, .. }) => {
                        tracing::info!("收到分片 from {}: {} bytes", peer, shard_data.len());
                    }

                    Some(NetworkEvent::StoreResponse { peer, res, .. }) => {
                        match res {
                            Ok(TaoStoreAck { accepted, stored_hash }) => {
                                tracing::info!("确认 from {}: accepted={}, hash={}", peer, accepted, stored_hash);
                            }
                            Err(e) => tracing::warn!("错误 from {}: {}", peer, e),
                        }
                    }

                    Some(NetworkEvent::NameResolved { name, entry }) => {
                        if let Some(e) = entry {
                            tracing::info!("DHT: '{}' → {}", name, hex::encode(&e.content_hash[..8]));
                        }
                    }
                    Some(NetworkEvent::PeerExpired(peer)) => {
                        tracing::info!("离线: {}", peer);
                    }
                    _ => {}
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("退出信号");
                break;
            }
        }
    }

    let retrieved = store.get(&unit.yin.content_hash)?.expect("数据应存在");
    let decrypted = decrypt(&retrieved.yin.payload, &[0x42u8; 32]).expect("解密");
    assert_eq!(decrypted, content);
    tracing::info!("=== 道存储节点已退出 ===");
    Ok(())
}

fn load_or_generate_key(
    key_file: &Option<String>, data_dir: &PathBuf,
) -> anyhow::Result<ed25519_dalek::SigningKey> {
    let path = match key_file {
        Some(p) => PathBuf::from(expand_tilde(p)),
        None => data_dir.join("node.key"),
    };
    if path.exists() {
        let bytes = std::fs::read(&path)?;
        let seed: [u8; 32] = bytes[..32].try_into().unwrap();
        Ok(ed25519_dalek::SigningKey::from_bytes(&seed))
    } else {
        let kp = generate_ed25519_keypair();
        std::fs::write(&path, kp.as_bytes())?;
        tracing::info!("生成密钥: {}", path.display());
        Ok(kp)
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        if path == "~" { home } else { format!("{}/{}", home, &path[2..]) }
    } else {
        path.to_string()
    }
}

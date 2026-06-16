use tao_core::{DataUnit, ErasureEncoder, LocalStore};
use tao_crypto::encrypt;
use tao_network::{NetworkCommand, NetworkEvent, TaoNetwork};
use tokio::time::{sleep, Duration};

/// 多节点集成测试
///
/// 启动 2 个节点 → mDNS 互相发现 → 分片传输 → 本地存储验证
#[tokio::test]
async fn test_two_nodes_discover_and_transfer_shard() {
    let dir_a = std::env::temp_dir().join("tao_int_a");
    let dir_b = std::env::temp_dir().join("tao_int_b");
    let _ = std::fs::remove_dir_all(&dir_a);
    let _ = std::fs::remove_dir_all(&dir_b);
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();

    let store_a = LocalStore::open(dir_a.join("storage")).expect("store a");
    let store_b = LocalStore::open(dir_b.join("storage")).expect("store b");

    let mut net_a = TaoNetwork::new().await.expect("net a");
    let mut net_b = TaoNetwork::new().await.expect("net b");

    tracing::info!("节点 A: {}", net_a.local_peer_id);
    tracing::info!("节点 B: {}", net_b.local_peer_id);

    // 等待 mDNS 发现
    let mut a_found_b = false;
    let timeout = Duration::from_secs(30);
    let start = tokio::time::Instant::now();

    while !a_found_b && start.elapsed() < timeout {
        tokio::select! {
            ev_a = net_a.recv_event() => {
                if let Some(NetworkEvent::PeerDiscovered(peer)) = ev_a {
                    if peer == net_b.local_peer_id {
                        tracing::info!("A 发现了 B!");
                        a_found_b = true;
                    }
                }
            }
            ev_b = net_b.recv_event() => {
                if let Some(NetworkEvent::PeerDiscovered(peer)) = ev_b {
                    if peer == net_a.local_peer_id {
                        tracing::info!("B 发现了 A!");
                    }
                }
            }
            _ = sleep(Duration::from_millis(100)) => {}
        }
    }

    assert!(a_found_b, "mDNS 发现超时");

    // 节点 A：创建数据 → 纠删码编码 → 发送分片给 B
    let content = "道可道，非常道".as_bytes();
    let encrypted = encrypt(content, &[0x42u8; 32]);

    let encoder = ErasureEncoder::new(2, 1); // 2+1: 小规模测试
    let shards = encoder.encode(&encrypted).expect("encode");
    assert_eq!(shards.len(), 3);

    // 发送片[2]（校验片）给节点 B
    let test_shard = &shards[2];
    net_a.send_command(NetworkCommand::StoreShard {
        peer: net_b.local_peer_id,
        content_hash: test_shard.original_hash,
        shard_index: test_shard.index,
        shard_data: test_shard.data.clone(),
    }).await.expect("send shard");

    // 节点 B：等待接收并存储
    let mut b_received = false;
    let start = tokio::time::Instant::now();
    while !b_received && start.elapsed() < timeout {
        tokio::select! {
            ev = net_b.recv_event() => {
                match ev {
                    Some(NetworkEvent::StoreRequest { peer, content_hash, shard_index, shard_data, .. }) => {
                        tracing::info!("B 收到片[{}] from {}", shard_index, peer);
                        let local_name = format!("shard_{}", shard_index);
                        let shard_unit = DataUnit::new(shard_data, local_name, [0u8; 32]);
                        store_b.store(&shard_unit).expect("store shard");
                        assert!(shard_unit.verify());
                        b_received = true;
                    }
                    _ => {}
                }
            }
            _ = sleep(Duration::from_millis(100)) => {}
        }
    }

    assert!(b_received, "B 应在超时前收到分片");

    // 清理
    let _ = std::fs::remove_dir_all(&dir_a);
    let _ = std::fs::remove_dir_all(&dir_b);
}

/// 名实分离测试：DHT 注册 + 解析
#[tokio::test]
async fn test_name_registration_and_resolution() {
    let dir = std::env::temp_dir().join("tao_int_nr");
    let _ = std::fs::remove_dir_all(&dir);

    let mut net = TaoNetwork::new().await.expect("net");
    tracing::info!("节点: {}", net.local_peer_id);

    let content_hash = [0xabu8; 32];

    net.send_command(NetworkCommand::RegisterName {
        name: "道德经".into(),
        content_hash,
        data_shards: 6,
        parity_shards: 2,
    }).await.expect("register name");

    net.send_command(NetworkCommand::ResolveName {
        name: "道德经".into(),
    }).await.expect("resolve name");

    tracing::info!("名实分离 DHT 操作完成");

    let _ = std::fs::remove_dir_all(&dir);
}

use anyhow::Context;
use libp2p::{
    futures::StreamExt,
    gossipsub, identify, kad, mdns,
    request_response::{self as rr},
    swarm::SwarmEvent,
    PeerId, Swarm,
};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::behaviour::{TaoBehaviour, TaoBehaviourEvent};
use crate::dht::{TaoDht, DhtEntry};
use crate::protocol::{
    TaoMessage, TaoRequest, TaoResponse,
    TaoStorePayload, TaoStoreAck,
    TaoRetrievePayload, TaoRetrieveData,
    PROTOCOL_NAME,
};
use crate::transport::build_transport;

#[derive(Debug)]
pub enum NetworkEvent {
    StoreRequest {
        peer: PeerId,
        request_id: rr::InboundRequestId,
        content_hash: [u8; 32],
        shard_index: usize,
        shard_data: Vec<u8>,
    },
    RetrieveRequest {
        peer: PeerId,
        request_id: rr::InboundRequestId,
        content_hash: [u8; 32],
        shard_index: usize,
    },
    StoreResponse {
        peer: PeerId,
        request_id: rr::OutboundRequestId,
        res: Result<TaoStoreAck, String>,
    },
    ShardFetched {
        peer: PeerId,
        content_hash: [u8; 32],
        shard_index: usize,
        shard_data: Vec<u8>,
    },
    NameResolved { name: String, entry: Option<DhtEntry> },
    PeerDiscovered(PeerId),
    PeerExpired(PeerId),
    DhtRecordFound(kad::Record),
    ProvidersFound { content_hash: [u8; 32], providers: Vec<PeerId> },
    QueryTimeout { content_hash: [u8; 32] },
    GossipMessage(TaoMessage),
    ListenAddr(String),

}

#[derive(Debug)]
pub enum NetworkCommand {
    StoreShard {
        peer: PeerId,
        content_hash: [u8; 32],
        shard_index: usize,
        shard_data: Vec<u8>,
    },
    FetchShard {
        peer: PeerId,
        content_hash: [u8; 32],
        shard_index: usize,
    },
    /// 回复 Retrieve 请求（必须调用，否则对端超时）
    SendRetrieveResponse {
        request_id: rr::InboundRequestId,
        shard_data: Vec<u8>,
    },
    RegisterName { name: String, content_hash: [u8; 32], data_shards: u8, parity_shards: u8 },
    ResolveName { name: String },
    ProvideContent { content_hash: [u8; 32] },
    FindProviders { content_hash: [u8; 32] },
    GossipQi { data: Vec<u8> },
    /// 拨号引导节点
    Dial { addr: String },
}

pub struct TaoNetwork {
    cmd_tx: mpsc::Sender<NetworkCommand>,
    event_rx: mpsc::Receiver<NetworkEvent>,
    pub local_peer_id: PeerId,
}

impl TaoNetwork {
    pub async fn new() -> anyhow::Result<Self> {
        Self::new_with_listen("/ip4/0.0.0.0/tcp/0").await
    }

    pub async fn new_with_listen(listen_addr: &str) -> anyhow::Result<Self> {
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(keypair.public());
        let transport = build_transport(&keypair)?;
        let kad = kad::Behaviour::new(local_peer_id, kad::store::MemoryStore::new(local_peer_id));
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub::Config::default(),
        ).map_err(|s| anyhow::anyhow!("{}", s))?;
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;
        let identify = identify::Behaviour::new(
            identify::Config::new("/ipfs/0.1.0".into(), keypair.public()),
        );
        let request_response = rr::cbor::Behaviour::new(
            [(libp2p::StreamProtocol::new(PROTOCOL_NAME), rr::ProtocolSupport::Full)],
            rr::Config::default(),
        );
        let behaviour = TaoBehaviour { kademlia: kad, gossipsub, mdns, identify, request_response };
        let swarm = Swarm::new(transport, behaviour, local_peer_id,
            libp2p::swarm::Config::with_tokio_executor());
        let (cmd_tx, cmd_rx) = mpsc::channel(128);
        let (event_tx, event_rx) = mpsc::channel(128);
        tokio::spawn(event_loop(swarm, cmd_rx, event_tx, listen_addr.to_string()));
        Ok(Self { cmd_tx, event_rx, local_peer_id })
    }

    pub async fn send_command(&self, cmd: NetworkCommand) -> anyhow::Result<()> {
        self.cmd_tx.send(cmd).await.context("channel closed")?;
        Ok(())
    }

    pub async fn recv_event(&mut self) -> Option<NetworkEvent> {
        self.event_rx.recv().await
    }
}

async fn event_loop(
    mut swarm: Swarm<TaoBehaviour>,
    mut cmd_rx: mpsc::Receiver<NetworkCommand>,
    event_tx: mpsc::Sender<NetworkEvent>,
    listen_addr: String,
) {
    // 挂起的 Retrieve 请求通道
    let mut pending_retrieves: HashMap<rr::InboundRequestId, rr::ResponseChannel<TaoResponse>> = HashMap::new();
    let mut pending_fetches: HashMap<rr::OutboundRequestId, ([u8; 32], usize)> = HashMap::new();

    let _ = swarm.behaviour_mut().gossipsub.subscribe(&gossipsub::IdentTopic::new("tao-qi"));
    if let Ok(addr) = listen_addr.parse::<libp2p::Multiaddr>() {
        let _ = swarm.listen_on(addr);
    } else {
        let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap());
    }
    swarm.behaviour_mut().kademlia.bootstrap().ok();

    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    NetworkCommand::StoreShard { peer, content_hash, shard_index, shard_data } => {
                        swarm.behaviour_mut().request_response.send_request(&peer,
                            TaoRequest::Store(TaoStorePayload { content_hash, shard_index, shard_data }));
                    }
                    NetworkCommand::FetchShard { peer, content_hash, shard_index } => {
                        let req_id = swarm.behaviour_mut().request_response.send_request(&peer,
                            TaoRequest::Retrieve(TaoRetrievePayload { content_hash, shard_index }));
                        pending_fetches.insert(req_id, (content_hash, shard_index));
                    }
                    NetworkCommand::SendRetrieveResponse { request_id, shard_data } => {
                        if let Some(channel) = pending_retrieves.remove(&request_id) {
                            let _ = swarm.behaviour_mut().request_response.send_response(
                                channel,
                                TaoResponse::RetrieveData(TaoRetrieveData { found: true, shard_data }),
                            );
                        }
                    }
                    NetworkCommand::RegisterName { name, content_hash, data_shards, parity_shards } => {
                        let (key, value) = TaoDht::make_register_record(
                            &name, &content_hash, data_shards, parity_shards, &swarm.local_peer_id());
                        let _ = swarm.behaviour_mut().kademlia.put_record(
                            kad::Record::new(key, value), kad::Quorum::One);
                    }
                    NetworkCommand::ResolveName { name } => {
                        swarm.behaviour_mut().kademlia.get_record(TaoDht::name_key(&name));
                    }
                    NetworkCommand::ProvideContent { content_hash } => {
                        let _ = swarm.behaviour_mut().kademlia.start_providing(
                            TaoDht::content_key(&content_hash));
                    }
                    NetworkCommand::FindProviders { content_hash } => {
                        swarm.behaviour_mut().kademlia.get_providers(
                            TaoDht::content_key(&content_hash));
                    }
                    NetworkCommand::GossipQi { data } => {
                        let _ = swarm.behaviour_mut().gossipsub.publish(
                            gossipsub::IdentTopic::new("tao-qi"), data);
                    }
                    NetworkCommand::Dial { addr } => {
                        if let Ok(ma) = addr.parse::<libp2p::Multiaddr>() {
                            let _ = swarm.dial(ma);
                        }
                    }
                }
            }

            event = swarm.select_next_some() => {
                let send = |ev| async { let _ = event_tx.send(ev).await; };
                match event {
                    SwarmEvent::Behaviour(TaoBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer, addr) in list {
                            swarm.behaviour_mut().kademlia.add_address(&peer, addr);
                            send(NetworkEvent::PeerDiscovered(peer)).await;
                        }
                    }
                    SwarmEvent::Behaviour(TaoBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer, _) in list { send(NetworkEvent::PeerExpired(peer)).await; }
                    }
                    SwarmEvent::Behaviour(TaoBehaviourEvent::Kademlia(
                        kad::Event::OutboundQueryProgressed {
                            result: kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(r))),
                            ..
                        }
                    )) => {
                        if let Some(entry) = TaoDht::parse_entry(&r.record.value) {
                            send(NetworkEvent::NameResolved { name: String::new(), entry: Some(entry) }).await;
                        } else {
                            send(NetworkEvent::DhtRecordFound(r.record)).await;
                        }
                    }
                    SwarmEvent::Behaviour(TaoBehaviourEvent::RequestResponse(
                        rr::Event::Message { peer, message }
                    )) => {
                        match message {
                            rr::Message::Request { request_id, request, channel } => {
                                match request {
                                    TaoRequest::Store(p) => {
                                        send(NetworkEvent::StoreRequest {
                                            peer, request_id,
                                            content_hash: p.content_hash,
                                            shard_index: p.shard_index,
                                            shard_data: p.shard_data,
                                        }).await;
                                    }
                                    TaoRequest::Retrieve(p) => {
                                        // 保存通道，等待上层回复
                                        pending_retrieves.insert(request_id, channel);
                                        send(NetworkEvent::RetrieveRequest {
                                            peer, request_id,
                                            content_hash: p.content_hash,
                                            shard_index: p.shard_index,
                                        }).await;
                                    }
                                }
                            }
                            rr::Message::Response { request_id, response } => {
                                match response {
                                    TaoResponse::StoreAck(ack) => {
                                        send(NetworkEvent::StoreResponse {
                                            peer, request_id, res: Ok(ack),
                                        }).await;
                                    }
                                    TaoResponse::RetrieveData(data) => {
                                        let (ch, si) = pending_fetches.remove(&request_id).unwrap_or(([0u8; 32], 0));
                                        send(NetworkEvent::ShardFetched {
                                            peer,
                                            content_hash: ch,
                                            shard_index: si,
                                            shard_data: data.shard_data,
                                        }).await;
                                    }
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(TaoBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { message, .. }
                    )) => {
                        if let Ok(msg) = bincode::deserialize::<TaoMessage>(&message.data) {
                            send(NetworkEvent::GossipMessage(msg)).await;
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        tracing::info!("监听: {}", address);
                        send(NetworkEvent::ListenAddr(address.to_string())).await;
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        tracing::info!("已连接: {}", peer_id);
                        swarm.behaviour_mut().kademlia.bootstrap().ok();
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        tracing::warn!("连接失败 {}: {}", peer_id.unwrap_or(PeerId::random()), error);
                    }
                    _ => {}
                }
            }
        }
    }
}

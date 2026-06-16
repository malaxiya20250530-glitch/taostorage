use libp2p::{
    gossipsub, identify, kad, mdns,
    request_response::{self as rr},
    swarm::NetworkBehaviour,
};
use crate::protocol::{TaoRequest, TaoResponse};

#[derive(NetworkBehaviour)]
pub struct TaoBehaviour {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub request_response: rr::cbor::Behaviour<TaoRequest, TaoResponse>,
}

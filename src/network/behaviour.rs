use libp2p::{
    gossipsub, mdns, ping,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};

use serde::{Deserialize, Serialize};

/// ================= 私聊消息 =================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateMessage {
    pub from: String,
    pub message: String,
}

/// ================= 事件 =================
#[derive(Debug)]
pub enum NodBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Ping(ping::Event),
    Private(request_response::Event<PrivateMessage, ()>),
}

/// ================= Behaviour =================
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "NodBehaviourEvent")]
pub struct NodBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub request_response: request_response::json::Behaviour<PrivateMessage, ()>,
}

impl NodBehaviour {
    pub fn new(key: &libp2p::identity::Keypair) -> Self {
        let peer_id = key.public().to_peer_id();

        // ===== gossipsub =====
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .mesh_n(6)
            .mesh_n_low(4)
            .mesh_n_high(12)
            .heartbeat_interval(std::time::Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::None)
            .build()
            .unwrap();

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )
        .unwrap();

        // ===== mdns =====
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).unwrap();

        // ===== ping =====
        let ping = ping::Behaviour::default();

        // ===== 私聊（关键）=====
        let protocols =
            std::iter::once((StreamProtocol::new("/nod/private/1"), ProtocolSupport::Full));

        let request_response =
            request_response::json::Behaviour::new(protocols, request_response::Config::default());

        Self {
            gossipsub,
            mdns,
            ping,
            request_response,
        }
    }
}

//
// ================= 事件转换（必须） =================
//

impl From<gossipsub::Event> for NodBehaviourEvent {
    fn from(e: gossipsub::Event) -> Self {
        Self::Gossipsub(e)
    }
}

impl From<mdns::Event> for NodBehaviourEvent {
    fn from(e: mdns::Event) -> Self {
        Self::Mdns(e)
    }
}

impl From<ping::Event> for NodBehaviourEvent {
    fn from(e: ping::Event) -> Self {
        Self::Ping(e)
    }
}

impl From<request_response::Event<PrivateMessage, ()>> for NodBehaviourEvent {
    fn from(e: request_response::Event<PrivateMessage, ()>) -> Self {
        Self::Private(e)
    }
}

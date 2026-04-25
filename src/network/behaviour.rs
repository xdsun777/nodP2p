use libp2p::{
    gossipsub, identity, mdns, ping,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use serde::{Deserialize, Serialize};
use std::iter::once;

// 私聊消息：支持文字 + 文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivateMessage {
    Text(String),
    File {
        name: String,
        data: Vec<u8>,
    },
    BinaryFile {
        name: String,
        data: Vec<u8>,
    },
}

// 事件枚举
#[derive(Debug)]
pub enum NodBehaviourEvent {
    Mdns(mdns::Event),
    Ping(ping::Event),
    Gossipsub(gossipsub::Event),
    Private(request_response::Event<PrivateMessage, ()>),
}

// 行为组合
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "NodBehaviourEvent")]
pub struct NodBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub request_response: request_response::json::Behaviour<PrivateMessage, ()>,
}

impl NodBehaviour {
    pub fn new(key: &identity::Keypair) -> Self {
        let peer_id = key.public().to_peer_id();

        // MDNS
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).unwrap();

        // Ping
        let ping = ping::Behaviour::new(ping::Config::new());

        // 广播
        let config = gossipsub::Config::default();
        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            config,
        ).unwrap();
        let topic = gossipsub::IdentTopic::new("chat");
        gossipsub.subscribe(&topic).unwrap();

        // 私聊协议
        let proto = once((
            StreamProtocol::new("/p2p/chat/private/1"),
            ProtocolSupport::Full,
        ));
        let request_response = request_response::json::Behaviour::new(proto, Default::default());

        Self {
            mdns,
            ping,
            gossipsub,
            request_response,
        }
    }
}

// 事件转换
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

impl From<gossipsub::Event> for NodBehaviourEvent {
    fn from(e: gossipsub::Event) -> Self {
        Self::Gossipsub(e)
    }
}

impl From<request_response::Event<PrivateMessage, ()>> for NodBehaviourEvent {
    fn from(e: request_response::Event<PrivateMessage, ()>) -> Self {
        Self::Private(e)
    }
}
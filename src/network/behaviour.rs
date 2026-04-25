use libp2p::{
    gossipsub, identity, mdns, ping,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use serde::{Deserialize, Serialize};
use std::iter::once;

/// 私聊消息类型，支持文字和文件传输
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivateMessage {
    /// 文本消息
    Text(String),
    /// 文件请求
    FileRequest { 
        /// 传输ID
        transfer_id: u64, 
        /// 文件名
        file_name: String, 
        /// 文件大小（字节）
        file_size: u64 
    },
    /// 接受文件传输
    FileAccept { 
        /// 传输ID
        transfer_id: u64 
    },
    /// 拒绝文件传输
    FileDeny { 
        /// 传输ID
        transfer_id: u64 
    },
    /// 文件数据块
    FileChunk { 
        /// 传输ID
        transfer_id: u64, 
        /// 文件偏移
        offset: u64, 
        /// 数据块内容
        data: Vec<u8>, 
        /// 是否为最后一块
        is_last: bool 
    },
}

/// 网络行为事件
#[derive(Debug)]
pub enum NodBehaviourEvent {
    /// mDNS 事件
    Mdns(mdns::Event),
    /// Ping 事件
    Ping(ping::Event),
    /// Gossipsub 事件
    Gossipsub(gossipsub::Event),
    /// 私聊请求-响应事件
    Private(request_response::Event<PrivateMessage, ()>),
}

/// 节点网络行为组合
/// 
/// 包含所有网络协议的实现：
/// - mDNS: 本地节点发现
/// - Ping: 节点活跃性检测
/// - Gossipsub: 消息广播
/// - Request-Response: 点对点私聊
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "NodBehaviourEvent")]
pub struct NodBehaviour {
    /// mDNS 行为
    pub mdns: mdns::tokio::Behaviour,
    /// Ping 行为
    pub ping: ping::Behaviour,
    /// Gossipsub 行为
    pub gossipsub: gossipsub::Behaviour,
    /// 请求-响应行为（私聊）
    pub request_response: request_response::json::Behaviour<PrivateMessage, ()>,
}

impl NodBehaviour {
    /// 创建新的网络行为实例
    /// 
    /// # Arguments
    /// * `key` - 节点身份密钥对
    pub fn new(key: &identity::Keypair) -> Self {
        let peer_id = key.public().to_peer_id();

        // mDNS 发现
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).unwrap();

        // Ping 协议
        let ping = ping::Behaviour::new(ping::Config::new());

        // Gossipsub 广播
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
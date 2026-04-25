use libp2p::PeerId;
use std::collections::HashSet;

/// 对等体管理器，用于跟踪网络中已连接的节点
#[derive(Default)]
pub struct PeerManager {
    peers: HashSet<PeerId>,
}

impl PeerManager {
    /// 添加一个新的对等体到管理器
    pub fn add(&mut self, peer: PeerId) {
        self.peers.insert(peer);
    }

    /// 从管理器中移除一个对等体
    pub fn remove(&mut self, peer: &PeerId) {
        self.peers.remove(peer);
    }

    /// 获取所有已连接的对等体列表
    pub fn all(&self) -> Vec<PeerId> {
        self.peers.iter().cloned().collect()
    }

    /// 检查指定的对等体是否已连接
    pub fn contains(&self, peer: &PeerId) -> bool {
        self.peers.contains(peer)
    }
}

use libp2p::PeerId;
use std::collections::HashSet;

#[derive(Default)]
pub struct PeerManager {
    peers: HashSet<PeerId>,
}

impl PeerManager {
    pub fn add(&mut self, peer: PeerId) {
        self.peers.insert(peer);
    }

    pub fn remove(&mut self, peer: &PeerId) {
        self.peers.remove(peer);
    }

    pub fn all(&self) -> Vec<PeerId> {
        self.peers.iter().cloned().collect()
    }

    pub fn contains(&self, peer: &PeerId) -> bool {
        self.peers.contains(peer)
    }
}

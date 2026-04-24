use libp2p::{Multiaddr, PeerId};
use serde::Serialize;

#[derive(Debug, Serialize)] // 加上 Serialize
pub enum AppEvent {
    PeerDiscovered(PeerId, Multiaddr),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    MessageReceived { peer: PeerId, message: String },
}

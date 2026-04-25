use libp2p::{Multiaddr, PeerId};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppEvent {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    PeerDiscovered(PeerId, Multiaddr),
    MessageReceived {
        peer: PeerId,
        message: String,
    },
    PrivateText(PeerId, String), // 私聊文字
}

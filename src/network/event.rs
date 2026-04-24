use libp2p::{Multiaddr, PeerId};

#[derive(Debug)]
pub enum AppEvent {
    PeerDiscovered(PeerId, Multiaddr),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    MessageReceived {
        peer: PeerId,
        message: String,
    },
}
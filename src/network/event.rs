use libp2p::{PeerId, Multiaddr};

#[derive(Debug)]
pub enum AppEvent {
    PeerDiscovered(PeerId, Multiaddr),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
}
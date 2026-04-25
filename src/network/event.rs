use libp2p::{Multiaddr, PeerId};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub enum AppEvent {
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    PeerDiscovered(PeerId, Multiaddr),
    MessageReceived { peer: PeerId, message: String },
    PrivateText(PeerId, String),
    FileRequestReceived { peer: PeerId, transfer_id: u64, file_name: String, file_size: u64 },
    FileTransferStarted { peer: PeerId, transfer_id: u64, file_name: String },
    FileTransferProgress { peer: PeerId, transfer_id: u64, received: u64, total: u64 },
    FileReceived { peer: PeerId, file_name: String, saved_path: PathBuf },
    FileSent { peer: PeerId, transfer_id: u64 },
}
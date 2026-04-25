use libp2p::PeerId;
use std::path::PathBuf;
#[derive(Debug)]
pub enum Command {
    Broadcast(String),
    SendPrivateText { peer: PeerId, text: String },
    SendFile { peer: PeerId, path: PathBuf },
    SendFileChunk {
        transfer_id: u64,
        peer: PeerId,
        offset: u64,
        data: Vec<u8>,
        is_last: bool,
    },
}
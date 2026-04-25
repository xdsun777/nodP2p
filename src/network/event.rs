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
    PrivateFile(PeerId, String), // 收到文件
    PrivateFileBinary {          //二进制文件
        peer: PeerId,
        name: String,
        data: Vec<u8>,
    },
}

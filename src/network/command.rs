use libp2p::PeerId;

#[derive(Debug)]
pub enum Command {
    Broadcast(String),
    SendPrivateText { peer: PeerId, text: String },
    SendPrivateFile { peer: PeerId, path: String },
    SendTo { peer: PeerId, msg: String },
    SendPrivateBinary { peer: PeerId, name: String, data: Vec<u8> },

}
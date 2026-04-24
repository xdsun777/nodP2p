use libp2p::PeerId;

#[derive(Debug)]
pub enum Command {
    Broadcast(String),
    SendTo { peer: PeerId, msg: String },
}
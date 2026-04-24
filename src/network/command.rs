use libp2p::PeerId;

#[derive(Debug)]
pub enum Command {
    Broadcast(String),
    Private { peer: PeerId, message: String },
}

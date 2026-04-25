use libp2p::PeerId;

#[derive(Debug)]
pub enum Command {
    Broadcast(String),
    SendPrivateText { peer: PeerId, text: String },

}
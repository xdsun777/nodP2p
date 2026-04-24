use libp2p::{
    swarm::{NetworkBehaviour},
    mdns,
    ping,
    identity,
};
use std::time::Duration;

#[derive(NetworkBehaviour)]
pub struct NodBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
}

impl NodBehaviour {
    pub fn new(key: &identity::Keypair) -> Self {
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            key.public().to_peer_id(),
        ).expect("mdns 初始化失败");

        let ping = ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(10))
        );

        Self { mdns, ping }
    }
}
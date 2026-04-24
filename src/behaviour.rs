use libp2p::{
    swarm::{NetworkBehaviour},
    mdns,
    ping,
    identity,
};
use std::time::Duration;

// 定义行为
#[derive(NetworkBehaviour)]
pub struct NodBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
}

impl NodBehaviour {
    // 必须是同步函数
    pub fn new(key: &identity::Keypair) -> anyhow::Result<Self> {

        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            key.public().to_peer_id(),
        )?;

        let ping = ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(10))
        );

        Ok(Self {
            mdns,
            ping,
        })
    }
}
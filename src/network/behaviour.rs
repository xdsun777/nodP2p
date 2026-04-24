use libp2p::{gossipsub, identity, mdns, ping, swarm::NetworkBehaviour};

#[derive(NetworkBehaviour)]
pub struct NodBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
}

impl NodBehaviour {
    pub fn new(key: &identity::Keypair) -> Self {
        let peer_id = key.public().to_peer_id();

        // mdns
        let mdns =
            mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).expect("mdns 初始化失败");

        // ping
        let ping = ping::Behaviour::new(ping::Config::new());

        // 👇 gossipsub
        let config = gossipsub::Config::default();

        let mut gossipsub =
            gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(key.clone()), config)
                .expect("gossipsub 初始化失败");

        // 订阅主题
        let topic = gossipsub::IdentTopic::new("chat");
        gossipsub.subscribe(&topic).unwrap();

        Self {
            mdns,
            ping,
            gossipsub,
        }
    }
}

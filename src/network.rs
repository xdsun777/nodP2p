use libp2p::{identity, noise, tcp, yamux, Swarm, SwarmBuilder};

use crate::behaviour::NodBehaviour;

// =============================
// 构建 Swarm网络
// =============================
pub fn build_swarm() -> anyhow::Result<Swarm<NodBehaviour>> {
    // =============================
    // 1. 身份（PeerId）
    // =============================
    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();

    println!("本节点 PeerId: {}", peer_id);

    // =============================
    // 2. 使用 Builder 构建 Transport + Behaviour
    // =============================
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| NodBehaviour::new(key).expect("行为初始化失败"))?
        .build();

    Ok(swarm)
}

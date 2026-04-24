use libp2p::{
    identity,
    noise,
    yamux,
    tcp,
    Swarm,
    swarm::{SwarmEvent},
    SwarmBuilder,
    futures::StreamExt,
};

use tokio::sync::mpsc;

use crate::network::{
    behaviour::{NodBehaviour, NodBehaviourEvent},
    command::Command,
    event::AppEvent,
    peer::PeerManager,
};

pub async fn start_swarm(
) -> anyhow::Result<(
    mpsc::UnboundedSender<Command>,
    mpsc::UnboundedReceiver<AppEvent>,
)> {

    // =============================
    // 通道
    // =============================
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // =============================
    // 身份
    // =============================
    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();

    println!("本节点: {}", peer_id);

    // =============================
    // 构建 Swarm
    // =============================
    let mut swarm: Swarm<NodBehaviour> =
        SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| NodBehaviour::new(key))?
            .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // =============================
    // 启动任务
    // =============================
    tokio::spawn(async move {

        let mut peers = PeerManager::default();

        loop {
            tokio::select! {

                // 网络事件
                event = swarm.select_next_some() => {
                    match event {

                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("监听: {}", address);
                        }

                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            peers.add(peer_id);
                            let _ = event_tx.send(AppEvent::PeerConnected(peer_id));
                        }

                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            peers.remove(&peer_id);
                            let _ = event_tx.send(AppEvent::PeerDisconnected(peer_id));
                        }

                        SwarmEvent::Behaviour(NodBehaviourEvent::Mdns(event)) => {
                            match event {
                                libp2p::mdns::Event::Discovered(list) => {
                                    for (peer, addr) in list {
                                        swarm.dial(addr.clone()).ok();
                                        let _ = event_tx.send(
                                            AppEvent::PeerDiscovered(peer, addr)
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }

                        _ => {}
                    }
                }

                // 命令处理
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {

                        Command::Broadcast(msg) => {
                            println!("广播消息: {}", msg);
                            // TODO: gossipsub
                        }

                        Command::SendTo { peer, msg } => {
                            println!("发送给 {}: {}", peer, msg);
                            // TODO: request-response
                        }
                    }
                }
            }
        }
    });

    Ok((cmd_tx, event_rx))
}
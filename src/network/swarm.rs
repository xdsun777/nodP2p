use libp2p::{
    futures::StreamExt, gossipsub::IdentTopic, identity, noise, swarm::SwarmEvent, tcp, yamux,
    Swarm, SwarmBuilder,
};

use tokio::sync::mpsc;

use crate::network::{
    behaviour::{NodBehaviour, NodBehaviourEvent},
    command::Command,
    event::AppEvent,
    peer::PeerManager,
};

pub async fn start_swarm() -> anyhow::Result<(
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
    let mut swarm: Swarm<NodBehaviour> = SwarmBuilder::with_existing_identity(keypair)
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
                        // 监听地址
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
                                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                                    }
                                }
                                _ => {}
                            }
                        }


                        SwarmEvent::Behaviour(NodBehaviourEvent::Gossipsub(event)) => {
                            match event {
                                libp2p::gossipsub::Event::Message {
                                propagation_source,
                                message,
                                    ..
                                } => {
                                    let msg = String::from_utf8_lossy(&message.data);

                                    println!("收到消息 [{}]: {}", propagation_source, msg);
                                    }
                            _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                // 3.****************命令处理*******************
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {

                        Command::Broadcast(msg) => {
                            let topic = IdentTopic::new("chat");

                            swarm.behaviour_mut().gossipsub.publish(topic, msg.as_bytes()).unwrap();
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

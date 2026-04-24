use libp2p::{
    futures::StreamExt,
    gossipsub::{Event as GossipsubEvent, IdentTopic},
    identity, mdns, noise, request_response,
    swarm::SwarmEvent,
    tcp, yamux, Swarm, SwarmBuilder,
};

use tokio::sync::mpsc;

use crate::network::{
    behaviour::{NodBehaviour, NodBehaviourEvent, PrivateMessage},
    command::Command,
    event::AppEvent,
    peer::PeerManager,
};

pub async fn start_swarm() -> anyhow::Result<(
    mpsc::UnboundedSender<Command>,
    mpsc::UnboundedReceiver<AppEvent>,
)> {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();

    println!("本节点: {}", peer_id);

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
    let topic = IdentTopic::new("chat");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    tokio::spawn(async move {
        let mut peers = PeerManager::default();
        let topic = topic.clone();

        loop {
            tokio::select! {

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

                        // ================= MDNS =================
                        SwarmEvent::Behaviour(NodBehaviourEvent::Mdns(event)) => {
                            match event {

                                mdns::Event::Discovered(list) => {
                                    for (peer, addr) in list {

                                        println!("发现节点: {}", peer);

                                        let _ = event_tx.send(
                                            AppEvent::PeerDiscovered(peer, addr.clone())
                                        );

                                        match swarm.dial(addr.clone()) {
                                            Ok(_) => println!("dial成功: {}", peer),
                                            Err(e) => println!("dial失败: {:?}", e),
                                        }

                                        swarm.behaviour_mut()
                                            .gossipsub
                                            .add_explicit_peer(&peer);
                                    }
                                }

                                mdns::Event::Expired(list) => {
                                    for (peer, _) in list {
                                        println!("节点过期: {}", peer);
                                    }
                                }
                            }
                        }

                        // ================= GOSSIPSUB =================
                        SwarmEvent::Behaviour(NodBehaviourEvent::Gossipsub(event)) => {
                            match event {

                                GossipsubEvent::Message {
                                    propagation_source,
                                    message,
                                    ..
                                } => {
                                    let msg = String::from_utf8_lossy(&message.data).into_owned();

                                    let _ = event_tx.send(AppEvent::MessageReceived {
                                        peer: propagation_source,
                                        message: msg,
                                    });
                                }

                                _ => {}
                            }
                        }


                        // =======================私聊=====================
                        SwarmEvent::Behaviour(NodBehaviourEvent::Private(event)) => {
                            match event {

                                request_response::Event::Message { peer, message } => {
                                    match message {

                                        request_response::Message::Request { request, channel, .. } => {

                                            println!("私聊来自 {}: {}", request.from, request.message);

                                            let _ = event_tx.send(AppEvent::MessageReceived {
                                                peer,
                                                message: format!("[私聊] {}", request.message),
                                            });

                                            // 回复 ACK
                                            swarm.behaviour_mut()
                                                .request_response
                                                .send_response(channel, ())
                                                .ok();
                                        }

                                        _ => {}
                                    }
                                }

                            _ => {}
                        }
                    }
                    _ => {}
                    }
                }

                Some(cmd) = cmd_rx.recv() => {
                    match cmd {

                        Command::Broadcast(msg) => {
                            let _ = swarm.behaviour_mut()
                                .gossipsub
                                .publish(topic.clone(), msg.as_bytes());
                        }

                        Command::Private { peer, message } => {
                            let req = PrivateMessage  {
                                from: peer_id.to_string(),
                                message:message,
                            };

                            swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer, req);
                        }
                    }
                }
            }
        }
    });

    Ok((cmd_tx, event_rx))
}

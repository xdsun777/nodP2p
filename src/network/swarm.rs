use libp2p::{
    futures::StreamExt, gossipsub::IdentTopic, identity, noise, request_response,
    swarm::SwarmEvent, tcp, yamux, PeerId, Swarm, SwarmBuilder,
};
use std::path::Path;
use tokio::sync::mpsc;

use crate::network::{
    behaviour::{NodBehaviour, NodBehaviourEvent, PrivateMessage},
    command::Command,
    event::AppEvent,
    peer::PeerManager,
};

pub async fn start_swarm(
    key: libp2p::identity::Keypair,
) -> anyhow::Result<(
    mpsc::UnboundedSender<Command>,
    mpsc::UnboundedReceiver<AppEvent>,
)> {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // let keypair = identity::Keypair::generate_ed25519();
    let peer_id = key.public().to_peer_id();
    println!("使用前端密钥启动：{}", peer_id);

    let mut swarm: Swarm<NodBehaviour> = SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, || {
            yamux::Config::default()
        })?
        .with_behaviour(|key| NodBehaviour::new(key))?
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    tokio::spawn(async move {
        let mut peers = PeerManager::default();

        loop {
            tokio::select! {
                            event = swarm.select_next_some() => {
                                match event {
                                    // 监听地址
                                    SwarmEvent::NewListenAddr { address, .. } => {
                                        println!("监听: {}", address);
                                    }

                                    // 连接建立
                                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                                        peers.add(peer_id);
                                        let _ = event_tx.send(AppEvent::PeerConnected(peer_id));
                                    }

                                    // 断开连接
                                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                        peers.remove(&peer_id);
                                        let _ = event_tx.send(AppEvent::PeerDisconnected(peer_id));
                                    }

                                    // MDNS 发现
                                    SwarmEvent::Behaviour(NodBehaviourEvent::Mdns(event)) => {
                                        if let libp2p::mdns::Event::Discovered(list) = event {
                                            for (peer, addr) in list {
                                                let _ = event_tx.send(AppEvent::PeerDiscovered(peer, addr.clone()));
                                                swarm.dial(addr).ok();
                                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                                            }
                                        }
                                    }

                                    // 广播消息
                                    SwarmEvent::Behaviour(NodBehaviourEvent::Gossipsub(event)) => {
                                        if let libp2p::gossipsub::Event::Message {
                                            propagation_source,
                                            message,
                                            ..
                                        } = event {
                                            let msg = String::from_utf8_lossy(&message.data).into_owned();
                                            let _ = event_tx.send(AppEvent::MessageReceived {
                                                peer: propagation_source,
                                                message: msg,
                                            });
                                        }
                                    }

                                    // ===================== 私聊/文件 接收 =====================
                                    SwarmEvent::Behaviour(NodBehaviourEvent::Private(event)) => {
                match event {
                    // 正确：Message 统一匹配，内部再分 Request / Response
                    request_response::Event::Message { peer, message } => {
                        match message {
                            // 1. 处理请求
                            request_response::Message::Request { request, channel,.. } => {
                                match request {
                                    PrivateMessage::Text(text) => {
                                        println!("[私聊] {}: {}", peer, text);
                                        let _ = event_tx.send(AppEvent::PrivateText(peer, text));
                                    }

                                    PrivateMessage::File { name, data } => {
                                        println!("[DEBUG] 成功收到 File 消息：{}", name);
                                        let save_path = std::env::current_dir().unwrap_or_default().join(&name);
                                        if let Err(e) = tokio::fs::write(&save_path, &data).await {
                                            println!("[写入失败] {}: {}", name, e);
                                        } else {
                                            println!("[写入成功] {}", name);
                                        }
                                        let _ = event_tx.send(AppEvent::PrivateFile(peer, name));
                                    }

                                    PrivateMessage::BinaryFile { name, data } => {
                                        println!("[收二进制文件] {} -> {}", peer, name);
                                        let _ = event_tx.send(AppEvent::PrivateFileBinary { peer, name, data });
                                    }
                                }
                                // 必须回复
                                let _ = swarm.behaviour_mut().request_response.send_response(channel, ());
                            }

                            // 2. 处理响应（和 Request 同级）
                            request_response::Message::Response { .. } => {
                                println!("[DEBUG] 文件发送成功，收到对方回执");
                            }
                        }
                    }

                    // 错误处理
                    request_response::Event::OutboundFailure { error, .. } => {
                        println!("[发送失败]: {:?}", error);
                    }
                    request_response::Event::InboundFailure { error, .. } => {
                        println!("[接收失败]: {:?}", error);
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
                                    // 广播
                                    Command::Broadcast(msg) => {
                                        let topic = IdentTopic::new("chat");
                                        let _ = swarm.behaviour_mut().gossipsub.publish(topic, msg.as_bytes());
                                    }

                                    // 私聊发文字
                                    Command::SendPrivateText { peer, text } => {
                                        let msg = PrivateMessage::Text(text);
                                        swarm.behaviour_mut().request_response.send_request(&peer, msg);
                                    }

                                    // 私聊发文件
                                    Command::SendPrivateFile { peer, path } => {
                println!("尝试读取文件: {}", path);
                match tokio::fs::read(&path).await {
                    Ok(data) => {
                        let name = Path::new(&path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();

                        println!("✅ 读取文件成功: {} ({} 字节)", path, data.len());
                        println!("📛 发送文件名: {}", name);

                        // ✅ 修复：克隆 data
                        let msg = PrivateMessage::File {
                            name,
                            data: data.clone(),
                        };

                        swarm.behaviour_mut().request_response.send_request(&peer, msg);
                        println!("✅ 文件已发送至: {}", peer);
                    }
                    Err(e) => {
                        println!("❌ 读取文件失败: {} -> 错误: {}", path, e);
                    }
                }
            }

                                    // 二进制文件发送命令
                                    Command::SendPrivateBinary { peer, name, data } => {
                                        let msg = PrivateMessage::BinaryFile { name, data };
                                        swarm.behaviour_mut().request_response.send_request(&peer, msg);
                                        println!("二进制文件发送成功");
                                    }



                                    _ => {}
                                }
                            }
                        }
        }
    });

    Ok((cmd_tx, event_rx))
}

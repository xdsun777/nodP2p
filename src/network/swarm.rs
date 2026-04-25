use libp2p::{
    futures::StreamExt, gossipsub::IdentTopic, noise, request_response, swarm::SwarmEvent, tcp,
    yamux, PeerId, Swarm, SwarmBuilder,
};
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

                // ================= 网络事件 =================
                event = swarm.select_next_some() => {
                    match event {

                        // 本地监听地址
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("监听地址: {}", address);
                        }

                        // 建立连接
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            let is_new = !peers.contains(&peer_id);

                            peers.add(peer_id);

                            if is_new {
                                println!("连接成功: {}", peer_id);
                                let _ = event_tx.send(AppEvent::PeerConnected(peer_id));
                            }
                        }

                        // 连接关闭
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            peers.remove(&peer_id);

                            println!("连接断开: {}", peer_id);
                            let _ = event_tx.send(AppEvent::PeerDisconnected(peer_id));
                        }

                        // ================= MDNS 自动发现 =================
                        SwarmEvent::Behaviour(
                            NodBehaviourEvent::Mdns(
                                libp2p::mdns::Event::Discovered(list)
                            )
                        ) => {
                            for (peer, addr) in list {

                                // 通知前端发现节点
                                let _ = event_tx.send(
                                    AppEvent::PeerDiscovered(peer, addr.clone())
                                );

                                // 关键修复：
                                // 已连接节点不再重复 dial
                                if !peers.contains(&peer) {
                                    println!("发现新节点，开始连接: {}", peer);

                                    let _ = swarm.dial(addr);

                                    swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .add_explicit_peer(&peer);
                                }
                            }
                        }

                        // ================= 广播消息 =================
                        SwarmEvent::Behaviour(
                            NodBehaviourEvent::Gossipsub(
                                libp2p::gossipsub::Event::Message {
                                    propagation_source,
                                    message,
                                    ..
                                }
                            )
                        ) => {
                            let text =
                                String::from_utf8_lossy(&message.data).to_string();

                            let _ = event_tx.send(
                                AppEvent::MessageReceived {
                                    peer: propagation_source,
                                    message: text,
                                }
                            );
                        }

                        // ================= 私聊消息 =================
                        SwarmEvent::Behaviour(
                            NodBehaviourEvent::Private(event)
                        ) => {
                            match event {

                                // 收到请求 / 响应
                                request_response::Event::Message {
                                    peer,
                                    message,
                                } => {
                                    match message {

                                        // 收到私聊请求
                                        request_response::Message::Request {
                                            request,
                                            channel,
                                            ..
                                        } => {
                                            match request {
                                                PrivateMessage::Text(text) => {
                                                    println!(
                                                        "[私聊收到] {}: {}",
                                                        peer,
                                                        text
                                                    );

                                                    let _ = event_tx.send(
                                                        AppEvent::PrivateText(
                                                            peer,
                                                            text,
                                                        )
                                                    );
                                                }
                                            }

                                            // 回复 ACK
                                            let _ = swarm
                                                .behaviour_mut()
                                                .request_response
                                                .send_response(channel, ());
                                        }

                                        // 收到回执
                                        request_response::Message::Response {
                                            ..
                                        } => {
                                            println!(
                                                "[私聊发送成功] 对方已确认接收"
                                            );
                                        }
                                    }
                                }

                                request_response::Event::OutboundFailure {
                                    peer,
                                    error,
                                    ..
                                } => {
                                    println!(
                                        "[发送失败] {} {:?}",
                                        peer,
                                        error
                                    );
                                }

                                request_response::Event::InboundFailure {
                                    peer,
                                    error,
                                    ..
                                } => {
                                    println!(
                                        "[接收失败] {} {:?}",
                                        peer,
                                        error
                                    );
                                }

                                request_response::Event::ResponseSent {
                                    peer,
                                    ..
                                } => {
                                    println!(
                                        "[已回复 ACK] {}",
                                        peer
                                    );
                                }
                            }
                        }

                        _ => {}
                    }
                }

                // ================= 命令处理 =================
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {

                        // 广播消息
                        Command::Broadcast(text) => {
                            let topic = IdentTopic::new("chat");

                            let _ = swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(topic, text.as_bytes());
                        }

                        // 私聊文字
                        Command::SendPrivateText { peer, text } => {

                            // 防止发给离线节点
                            if !peers.contains(&peer) {
                                println!("目标节点未连接: {}", peer);
                                continue;
                            }

                            println!("发送私聊 -> {}", peer);

                            swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(
                                    &peer,
                                    PrivateMessage::Text(text),
                                );
                        }

                        _ => {}
                    }
                }
            }
        }
    });

    Ok((cmd_tx, event_rx))
}

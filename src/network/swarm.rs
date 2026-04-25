use libp2p::{
    futures::StreamExt, gossipsub::IdentTopic, noise, request_response, swarm::SwarmEvent, tcp,
    yamux, Swarm, SwarmBuilder,
};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::mpsc;

use crate::network::{
    behaviour::{NodBehaviour, NodBehaviourEvent, PrivateMessage},
    command::Command,
    event::AppEvent,
    peer::PeerManager,
};

struct FileReceiver {
    file_name: String,
    file_size: u64,
    received: u64,
    data: Vec<u8>,
    buffer: BTreeMap<u64, Vec<u8>>,
}

/// 启动 P2P 网络节点
///
/// 初始化一个完整的 libp2p 网络节点，包括 TCP 传输、mDNS 发现、Gossipsub 消息广播
/// 和点对点私聊功能。该函数返回一个命令发送通道和事件接收通道。
///
/// # Arguments
/// * `key` - Ed25519 密钥对，用于节点身份标识
///
/// # Returns
/// 返回 `(cmd_tx, event_rx)` 元组：
/// - `cmd_tx`: 用于发送命令的无界通道发送器（广播、私聊、文件传输）
/// - `event_rx`: 用于接收网络事件的无界通道接收器
///
/// # Errors
/// 当监听地址解析失败或网络初始化失败时返回错误
///
/// # Example
/// ```no_run
/// # use nodp2p::start_swarm;
/// # use libp2p::identity::Keypair;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let key = Keypair::generate_ed25519();
/// let (cmd_tx, mut event_rx) = start_swarm(key).await?;
///
/// // 发送消息
/// cmd_tx.send(nodp2p::Command::Broadcast("Hello".to_string()))?;
///
/// // 接收事件
/// if let Some(event) = event_rx.recv().await {
///     // 处理事件
/// }
/// # Ok(())
/// # }
/// ```

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

    let cmd_tx_clone = cmd_tx.clone();
    tokio::spawn(async move {
        let mut peers = PeerManager::default();
        let mut pending_files: HashMap<u64, FileReceiver> = HashMap::new();
        let mut next_transfer_id: u64 = 0;

        loop {
            tokio::select! {
                // ================= 网络事件 =================
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("监听地址: {}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            let is_new = !peers.contains(&peer_id);
                            peers.add(peer_id);
                            if is_new {
                                println!("连接成功: {}", peer_id);
                                let _ = event_tx.send(AppEvent::PeerConnected(peer_id));
                            }
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            peers.remove(&peer_id);
                            println!("连接断开: {}", peer_id);
                            let _ = event_tx.send(AppEvent::PeerDisconnected(peer_id));
                        }
                        SwarmEvent::Behaviour(
                            NodBehaviourEvent::Mdns(
                                libp2p::mdns::Event::Discovered(list)
                            )
                        ) => {
                            for (peer, addr) in list {
                                let _ = event_tx.send(
                                    AppEvent::PeerDiscovered(peer, addr.clone())
                                );
                                if !peers.contains(&peer) {
                                    println!("发现新节点，开始连接: {}", peer);
                                    let _ = swarm.dial(addr);
                                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer);
                                }
                            }
                        }
                        SwarmEvent::Behaviour(
                            NodBehaviourEvent::Gossipsub(
                                libp2p::gossipsub::Event::Message {
                                    propagation_source,
                                    message,
                                    ..
                                }
                            )
                        ) => {
                            let text = String::from_utf8_lossy(&message.data).to_string();
                            let _ = event_tx.send(
                                AppEvent::MessageReceived {
                                    peer: propagation_source,
                                    message: text,
                                }
                            );
                        }
                        SwarmEvent::Behaviour(
                            NodBehaviourEvent::Private(event)
                        ) => {
                            match event {
                                request_response::Event::Message { peer, message } => {
                                    match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            match request {
                                                PrivateMessage::Text(text) => {
                                                    println!("[私聊收到] {}: {}", peer, text);
                                                    let _ = event_tx.send(
                                                        AppEvent::PrivateText(peer, text)
                                                    );
                                                    let _ = swarm
                                                        .behaviour_mut()
                                                        .request_response
                                                        .send_response(channel, ());
                                                }
                                                PrivateMessage::FileRequest { transfer_id, file_name, file_size } => {
                                                    // 初始化文件接收器来存储数据块
                                                    pending_files.insert(transfer_id, FileReceiver {
                                                        file_name: file_name.clone(),
                                                        file_size,
                                                        received: 0,
                                                        data: Vec::new(),
                                                        buffer: BTreeMap::new(),
                                                    });
                                                    let _ = swarm
                                                        .behaviour_mut()
                                                        .request_response
                                                        .send_response(channel, ());
                                                    let _ = event_tx.send(
                                                        AppEvent::FileTransferStarted {
                                                            peer,
                                                            transfer_id,
                                                            file_name,
                                                        }
                                                    );
                                                }
                                                PrivateMessage::FileChunk { transfer_id, offset, data, is_last } => {
                                                    // 立即回复，避免 ResponseOmission
                                                    let _ = swarm
                                                        .behaviour_mut()
                                                        .request_response
                                                        .send_response(channel, ());

                                                    if let Some(receiver) = pending_files.get_mut(&transfer_id) {
                                                        // 发送数据块事件给外部处理
                                                        let _ = event_tx.send(AppEvent::FileChunkReceived {
                                                            peer,
                                                            transfer_id,
                                                            offset,
                                                            data: data.clone(),
                                                            is_last,
                                                        });

                                                        if offset == receiver.received {
                                                            // 顺序接收数据
                                                            receiver.data.extend_from_slice(&data);
                                                            receiver.received += data.len() as u64;

                                                            // 处理缓冲区中连续的数据
                                                            while let Some(buffered_data) = receiver.buffer.remove(&receiver.received) {
                                                                receiver.data.extend_from_slice(&buffered_data);
                                                                receiver.received += buffered_data.len() as u64;
                                                            }

                                                            let _ = event_tx.send(
                                                                AppEvent::FileTransferProgress {
                                                                    peer,
                                                                    transfer_id,
                                                                    received: receiver.received,
                                                                    total: receiver.file_size,
                                                                }
                                                            );

                                                            if is_last || receiver.received >= receiver.file_size {
                                                                // 传输完成，发送完整文件数据
                                                                let _ = event_tx.send(
                                                                    AppEvent::FileReceived {
                                                                        peer,
                                                                        file_name: receiver.file_name.clone(),
                                                                        data: receiver.data.clone(),
                                                                    }
                                                                );
                                                                pending_files.remove(&transfer_id);
                                                            }
                                                        } else if offset > receiver.received {
                                                            // 超前到达的数据，缓存起来
                                                            receiver.buffer.insert(offset, data);
                                                        }
                                                        // 重复数据忽略
                                                    } else {
                                                        println!("未识别的 transfer_id: {}", transfer_id);
                                                    }
                                                }
                                                PrivateMessage::FileAccept { transfer_id } => {
                                                    println!("对方接受文件传输: transfer_id={}", transfer_id);
                                                }
                                                PrivateMessage::FileDeny { transfer_id } => {
                                                    println!("对方拒绝文件传输: transfer_id={}", transfer_id);
                                                }
                                            }
                                        }
                                        request_response::Message::Response { .. } => {
                                            println!("[私聊发送成功] 对方已确认接收");
                                        }
                                    }
                                }
                                request_response::Event::OutboundFailure { peer, error, .. } => {
                                    println!("[发送失败] {} {:?}", peer, error);
                                }
                                request_response::Event::InboundFailure { peer, error, .. } => {
                                    println!("[接收失败] {} {:?}", peer, error);
                                }
                                request_response::Event::ResponseSent { peer, .. } => {
                                    println!("[已回复 ACK] {}", peer);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // ================= 命令处理 =================
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        Command::Broadcast(text) => {
                            let topic = IdentTopic::new("chat");
                            let _ = swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(topic, text.as_bytes());
                        }
                        Command::SendPrivateText { peer, text } => {
                            if !peers.contains(&peer) {
                                println!("目标节点未连接: {}", peer);
                                continue;
                            }
                            println!("发送私聊 -> {}", peer);
                            swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer, PrivateMessage::Text(text));
                        }
                        Command::SendFile { peer, path } => {
                            if !peers.contains(&peer) {
                                println!("目标节点未连接: {}", peer);
                                continue;
                            }

                            let transfer_id = next_transfer_id;
                            next_transfer_id += 1;

                            match tokio::fs::metadata(&path).await {
                                Ok(meta) => {
                                    let file_size = meta.len();
                                    let file_name = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string();

                                    // 发送文件请求
                                    let req = PrivateMessage::FileRequest {
                                        transfer_id,
                                        file_name: file_name.clone(),
                                        file_size,
                                    };
                                    swarm.behaviour_mut().request_response.send_request(&peer, req);
                                    let _ = event_tx.send(AppEvent::FileTransferStarted {
                                        peer,
                                        transfer_id,
                                        file_name: file_name.clone(),
                                    });

                                    // 使用 cmd_tx_clone 来避免移动问题
                                    let tx = cmd_tx_clone.clone();
                                    tokio::spawn(async move {
                                        match tokio::fs::File::open(&path).await {
                                            Ok(mut file) => {
                                                use tokio::io::AsyncReadExt;
                                                let mut offset: u64 = 0;
                                                let chunk_size: usize = 256 * 1024;
                                                loop {
                                                    let mut buf = vec![0u8; chunk_size];
                                                    match file.read(&mut buf).await {
                                                        Ok(0) => break,
                                                        Ok(n) => {
                                                            buf.truncate(n);
                                                            let is_last = offset + n as u64 >= file_size;
                                                            let _ = tx.send(Command::SendFileChunk {
                                                                transfer_id,
                                                                peer,
                                                                offset,
                                                                data: buf,
                                                                is_last,
                                                            });
                                                            offset += n as u64;
                                                            if is_last {
                                                                break;
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("读取文件错误: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => eprintln!("无法打开文件: {}", e),
                                        }
                                    });
                                }
                                Err(e) => println!("无法读取文件: {}", e),
                            }
                        }
                        Command::SendFileChunk { transfer_id, peer, offset, data, is_last } => {
                            if !peers.contains(&peer) {
                                println!("目标节点已断开，无法发送数据块");
                                continue;
                            }
                            let msg = PrivateMessage::FileChunk {
                                transfer_id,
                                offset,
                                data,
                                is_last,
                            };
                            swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer, msg);
                        }
                        // 所有变体都已列出，无需 _ 兜底
                    }
                }
            }
        }
    });

    Ok((cmd_tx, event_rx))
}
use nodp2p::{start_swarm, AppEvent, Command};
use tokio::io::{self, AsyncBufReadExt};
use libp2p::PeerId;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (cmd_tx, mut event_rx) = start_swarm(libp2p::identity::Keypair::generate_ed25519()).await?;

    println!("节点启动完成");
    println!("--------------------------------");
    println!("普通输入              = 群发消息");
    println!("/s <peer> <msg>       = 私聊消息");
    println!("/file <peer> <path>   = 发送文件");
    println!("--------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match event {
                    AppEvent::PeerDiscovered(peer, addr) => {
                        println!("发现节点: {} @ {}", peer, addr);
                    }
                    AppEvent::PeerConnected(peer) => {
                        println!("节点已连接: {}", peer);
                    }
                    AppEvent::PeerDisconnected(peer) => {
                        println!("节点断开: {}", peer);
                    }
                    AppEvent::MessageReceived { peer, message } => {
                        println!("广播消息 [{}]: {}", peer, message);
                    }
                    AppEvent::PrivateText(peer, text) => {
                        println!("[私聊] {}: {}", peer, text);
                    }
                    AppEvent::FileRequestReceived { peer, file_name, file_size, .. } => {
                        println!("📥 收到文件请求: {} ({} bytes) 来自 {}", file_name, file_size, peer);
                    }
                    AppEvent::FileTransferStarted { peer, file_name, .. } => {
                        println!("📁 文件传输开始: {} -> {}", peer, file_name);
                    }
                    AppEvent::FileTransferProgress { transfer_id, peer, received, total } => {
                        println!("📶 传输进度 [{}] {}: {}/{}", transfer_id, peer, received, total);
                    }
                    AppEvent::FileReceived { peer, file_name, saved_path } => {
                        println!("✅ 文件接收完成: {} 保存至 {:?}", file_name, saved_path);
                    }
                    AppEvent::FileSent { peer, transfer_id } => {
                        println!("✅ 文件发送完成: transfer_id={}", transfer_id);
                    }
                    _ => {}
                }
            }

            Ok(Some(line)) = stdin.next_line() => {
                handle_input(line, &cmd_tx);
            }
        }
    }
}

fn handle_input(line: String, cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }

    if line.starts_with("/s ") {
        let mut parts = line.splitn(3, ' ');
        parts.next();
        let peer_str = match parts.next() {
            Some(p) => p,
            None => {
                println!("用法: /s <peer_id> <消息>");
                return;
            }
        };
        let text = match parts.next() {
            Some(t) => t,
            None => {
                println!("用法: /s <peer_id> <消息>");
                return;
            }
        };
        match peer_str.parse::<PeerId>() {
            Ok(peer_id) => {
                cmd_tx.send(Command::SendPrivateText {
                    peer: peer_id,
                    text: text.to_string(),
                }).unwrap();
                println!("✅ 私聊发送 -> {}: {}", peer_id, text);
            }
            Err(_) => println!("❌ PeerId 格式错误"),
        }
        return;
    }

    if line.starts_with("/file ") {
        let mut parts = line.splitn(3, ' ');
        parts.next();
        let peer_str = match parts.next() {
            Some(p) => p,
            None => {
                println!("用法: /file <peer_id> <文件路径>");
                return;
            }
        };
        let file_path = match parts.next() {
            Some(p) => p,
            None => {
                println!("用法: /file <peer_id> <文件路径>");
                return;
            }
        };
        match peer_str.parse::<PeerId>() {
            Ok(peer_id) => {
                let path = std::path::PathBuf::from(file_path);
                cmd_tx.send(Command::SendFile { peer: peer_id, path }).unwrap();
                println!("📁 开始发送文件 -> {}: {}", peer_id, file_path);
            }
            Err(_) => println!("❌ PeerId 格式错误"),
        }
        return;
    }

    // 默认广播消息
    cmd_tx.send(Command::Broadcast(line.to_string())).unwrap();
}
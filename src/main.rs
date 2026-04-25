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
    println!("/file <peer> <路径>    = 发送文件（路径模式）");
    println!("/binary <peer> <路径>  = 发送二进制文件（内存模式）"); // 👈 新增说明
    println!("--------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {
            // 网络事件
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
                    AppEvent::PrivateFile(peer, name) => {
                        println!("[文件] 收到来自 {} 的文件: {}", peer, name);
                    }
                    // 新增：二进制文件接收输出
                    AppEvent::PrivateFileBinary { peer, name, data } => {
                        println!("[二进制文件] {} -> {} (大小: {} 字节)", peer, name, data.len());
                    }
                }
            }

            // 用户输入
            Ok(Some(line)) = stdin.next_line() => {
                handle_input(line, &cmd_tx);
            }
        }
    }
}

// 处理输入命令
fn handle_input(line: String, cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }

    // -------------------- 私聊命令 --------------------
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

    // -------------------- 发送文件命令（路径） --------------------
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

        let path = match parts.next() {
            Some(p) => p,
            None => {
                println!("用法: /file <peer_id> <文件路径>");
                return;
            }
        };
        let path = path.trim_matches('"'); 
        match peer_str.parse::<PeerId>() {
            Ok(peer_id) => {
                cmd_tx.send(Command::SendPrivateFile {
                    peer: peer_id,
                    path: path.to_string(),
                }).unwrap();
                println!("✅ 开始发送文件 -> {}: {}", peer_id, path);
            }
            Err(_) => println!("❌ PeerId 格式错误"),
        }
        return;
    }

    // -------------------- 【新增】二进制文件发送命令 --------------------
    if line.starts_with("/binary ") {
        let mut parts = line.splitn(3, ' ');
        parts.next();

        let peer_str = match parts.next() {
            Some(p) => p,
            None => {
                println!("用法: /binary <peer_id> <文件路径>");
                return;
            }
        };

        let path = match parts.next() {
            Some(p) => p,
            None => {
                println!("用法: /binary <peer_id> <文件路径>");
                return;
            }
        };

        // 读取文件为二进制
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                println!("❌ 读取文件失败: {}", e);
                return;
            }
        };

        // 文件名
        let name = match std::path::Path::new(path).file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => "unknown_file".to_string(),
        };

        // 发送
        match peer_str.parse::<PeerId>() {
            Ok(peer_id) => {
                cmd_tx.send(Command::SendPrivateBinary {
                    peer: peer_id,
                    name,
                    data,
                }).unwrap();
                println!("✅ 二进制文件发送 -> {}", peer_id);
            }
            Err(_) => println!("❌ PeerId 格式错误"),
        }
        return;
    }

    // -------------------- 广播消息 --------------------
    cmd_tx.send(Command::Broadcast(line.to_string())).unwrap();
}
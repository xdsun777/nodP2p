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
                        println!("main：[私聊] {}: {}", peer, text);
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

    

    // -------------------- 广播消息 --------------------
    cmd_tx.send(Command::Broadcast(line.to_string())).unwrap();
}
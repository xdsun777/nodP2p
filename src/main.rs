use nodp2p::{start_swarm, AppEvent, Command};
use tokio::io::{self, AsyncBufReadExt};
use libp2p::PeerId;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (cmd_tx, mut event_rx) = start_swarm().await?;

    println!("节点启动完成");
    println!("--------------------------------");
    println!("普通输入 = 群发");
    println!("/s <peer_id> <msg> = 私聊");
    println!("--------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {

            // ================= 事件 =================
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
                        println!("广播消息： [{}]: {}", peer, message);
                    }
                }
            }

            // ================= 输入 =================
            Ok(Some(line)) = stdin.next_line() => {
                handle_input(line, &cmd_tx);
            }
        }
    }
}

fn handle_input(line: String, cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>) {
    let line = line.trim();

    // ================= 私聊 =================
    if line.starts_with("/s ") {
        // 格式: /s peer_id message
        let mut parts = line.splitn(3, ' ');

        parts.next(); // 跳过 /s

        let peer_str = match parts.next() {
            Some(p) => p,
            None => {
                println!("❌ 用法: /s <peer_id> <msg>");
                return;
            }
        };

        let msg = match parts.next() {
            Some(m) => m,
            None => {
                println!("❌ 用法: /s <peer_id> <msg>");
                return;
            }
        };

        // 解析 PeerId
        match peer_str.parse::<PeerId>() {
            Ok(peer_id) => {
                let _ = cmd_tx.send(Command::Private  {
                    peer: peer_id,
                    message: msg.to_string(),
                });

                println!("📤 私聊发送 -> {}: {}", peer_id, msg);
            }
            Err(_) => {
                println!("❌ 无效 PeerId");
            }
        }

        return;
    }

    // ================= 群发 =================
    let _ = cmd_tx.send(Command::Broadcast(line.to_string()));
}
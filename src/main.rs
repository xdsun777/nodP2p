use nodp2p::{start_swarm, AppEvent, Command};
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (cmd_tx, mut event_rx) = start_swarm().await?;

    println!("节点启动完成，可输入内容测试");

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
                        println!("收到消息 [{}]: {}", peer, message);
                    }
                }
            }

            Ok(Some(line)) = stdin.next_line() => {
                let _ = cmd_tx.send(Command::Broadcast(line));
            }
        }
    }
}
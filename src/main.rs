mod network;

use network::swarm::start_swarm;
use network::command::Command;

use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (cmd_tx, mut event_rx) = start_swarm().await?;

    println!("节点启动完成，可输入内容测试");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {

            // 接收网络事件
            Some(event) = event_rx.recv() => {
                println!("事件: {:?}", event);
            }

            // 用户输入
            Ok(Some(line)) = stdin.next_line() => {
                cmd_tx.send(Command::Broadcast(line)).unwrap();
            }
        }
    }
}
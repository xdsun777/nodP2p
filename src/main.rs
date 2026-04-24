mod network;
mod behaviour;
mod event;
use futures::StreamExt;
use network::build_swarm;
use event::handle_event;
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 构建网络 Swarm
    let mut swarm = build_swarm()?;

    // 启动监听（随机端口）
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("节点启动成功，开始监听...");

    // 异步读取控制台输入（预留给聊天等功能）
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {
            // 处理网络事件
            event = swarm.select_next_some() => {
                handle_event(event, &mut swarm).await;
            }

            // 处理用户输入（业务扩展入口）
            line = stdin.next_line() => {
                if let Ok(Some(line)) = line {
                    println!("输入内容：{}", line);

                    // TODO: 在这里发送消息（聊天功能扩展点）
                }
            }
        }
    }
}
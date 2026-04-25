/// 基本的 P2P 节点示例
///
/// 这个示例演示如何创建和启动一个 P2P 网络节点。

use nodp2p::{start_swarm, AppEvent, Command};
use libp2p::identity::Keypair;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 生成或加载密钥对
    let key = Keypair::generate_ed25519();
    
    // 启动 P2P 节点
    let (cmd_tx, mut event_rx) = start_swarm(key).await?;
    
    println!("P2P 节点启动成功");
    println!("等待节点连接...");
    
    // 在后台任务中发送消息
    let cmd_tx_clone = cmd_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // 发送一条广播消息
        if let Err(e) = cmd_tx_clone.send(Command::Broadcast(
            "Hello from nodp2p!".to_string()
        )) {
            eprintln!("Failed to send message: {}", e);
        }
    });
    
    // 接收并处理事件
    while let Some(event) = event_rx.recv().await {
        match event {
            AppEvent::PeerConnected(peer) => {
                println!("✓ 节点已连接: {}", peer);
            }
            AppEvent::PeerDisconnected(peer) => {
                println!("✗ 节点已断开: {}", peer);
            }
            AppEvent::PeerDiscovered(peer, addr) => {
                println!("◇ 发现新节点: {} @ {}", peer, addr);
            }
            AppEvent::MessageReceived { peer, message } => {
                println!("💬 广播消息 [{}]: {}", peer, message);
            }
            AppEvent::PrivateText(peer, text) => {
                println!("🔒 私聊 [{}]: {}", peer, text);
            }
            AppEvent::FileTransferStarted { peer, file_name, .. } => {
                println!("📤 文件传输开始: {} -> {}", peer, file_name);
            }
            AppEvent::FileTransferProgress { received, total, .. } => {
                println!("📊 传输进度: {}/{} bytes", received, total);
            }
            AppEvent::FileReceived { file_name, .. } => {
                println!("✓ 文件接收完成: {} )", file_name);
            }
            _ => {}
        }
    }
    
    Ok(())
}

# nodP2p - P2P 网络库

一个基于 libp2p 的高性能 P2P 网络库，支持消息广播、私聊和文件传输。

## 特性

- 🚀 **高性能 P2P 网络** - 基于 libp2p 框架
- 📡 **mDNS 节点发现** - 自动发现局域网内的节点
- 💬 **消息广播** - 使用 Gossipsub 协议实现可靠的消息广播
- 🔐 **私聊通信** - 点对点的私密通信
- 📁 **文件传输** - 支持节点间的文件传输
- 🔑 **密钥管理** - Ed25519 密钥对管理和持久化

## 快速开始

### 添加依赖

```toml
[dependencies]
nodp2p = "0.1"
tokio = { version = "1.0", features = ["full"] }
libp2p = { version = "0.52", features = ["tcp", "tokio", "yamux", "noise", "gossipsub", "mdns"] }
```

### 基本用法

```rust
use nodp2p::start_swarm;
use libp2p::identity::Keypair;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建密钥对
    let key = Keypair::generate_ed25519();
    
    // 启动节点
    let (cmd_tx, mut event_rx) = start_swarm(key).await?;
    
    // 发送广播消息
    cmd_tx.send(nodp2p::Command::Broadcast("Hello P2P".to_string()))?;
    
    // 接收事件
    while let Some(event) = event_rx.recv().await {
        println!("Event: {:?}", event);
    }
    
    Ok(())
}
```

## API 概览

### 命令 (Command)

- `Broadcast(String)` - 广播消息到所有节点
- `SendPrivateText { peer, text }` - 发送私聊消息
- `SendFile { peer, path }` - 发送文件

### 事件 (AppEvent)

- `PeerConnected(PeerId)` - 节点连接事件
- `PeerDisconnected(PeerId)` - 节点断开连接事件
- `PeerDiscovered(PeerId, Multiaddr)` - 发现新节点
- `MessageReceived { peer, message }` - 接收广播消息
- `PrivateText(PeerId, String)` - 接收私聊消息
- `FileTransferStarted { ... }` - 文件传输开始
- `FileTransferProgress { ... }` - 文件传输进度
- `FileReceived { ... }` - 文件接收完成

## 配置

使用 `AppConfig` 自定义节点行为：

```rust
use nodp2p::AppConfig;
use std::path::PathBuf;

let config = AppConfig::new()
    .with_mdns(true)
    .with_identity_path(PathBuf::from("./keypair.bin"));
```

## 许可证

MIT

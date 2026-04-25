# 使用指南

## 快速开始

### 1. 添加到你的项目
```toml
[dependencies]
nodp2p = { path = "../nodp2p" }
tokio = { version = "1.0", features = ["full"] }
libp2p = { version = "0.52", features = ["tcp", "tokio", "gossipsub", "mdns"] }
```

### 2. 基本用法
```rust
use nodp2p::start_swarm;
use libp2p::identity::Keypair;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建节点
    let key = Keypair::generate_ed25519();
    let (cmd_tx, mut event_rx) = start_swarm(key).await?;
    
    // 发送消息
    cmd_tx.send(nodp2p::Command::Broadcast("Hello!".to_string()))?;
    
    // 接收事件
    while let Some(event) = event_rx.recv().await {
        println!("Event: {:?}", event);
    }
    
    Ok(())
}
```

## 功能详解

### 消息广播
```rust
// 向所有节点发送消息
cmd_tx.send(Command::Broadcast("message".to_string()))?;
```

事件：
```rust
AppEvent::MessageReceived { peer, message } => {
    println!("From {}: {}", peer, message);
}
```

### 私聊通信
```rust
// 向特定节点发送私聊
cmd_tx.send(Command::SendPrivateText {
    peer: peer_id,
    text: "private message".to_string(),
})?;
```

事件：
```rust
AppEvent::PrivateText(peer, text) => {
    println!("Private message from {}: {}", peer, text);
}
```

### 文件传输
```rust
// 发送文件
cmd_tx.send(Command::SendFile {
    peer: peer_id,
    path: PathBuf::from("./document.pdf"),
})?;
```

事件：
```rust
AppEvent::FileTransferStarted { peer, file_name, .. } => {
    println!("Sending {} to {}", file_name, peer);
}

AppEvent::FileTransferProgress { received, total, .. } => {
    println!("Progress: {}/{} bytes", received, total);
}

AppEvent::FileReceived { file_name, saved_path, .. } => {
    println!("File {} saved to {:?}", file_name, saved_path);
}
```

## 错误处理

所有操作返回 `Result`：
```rust
match cmd_tx.send(command) {
    Ok(_) => println!("Command sent"),
    Err(e) => eprintln!("Failed to send command: {:?}", e),
}
```

## 配置

### 自定义配置
当前支持的配置选项：
- `enable_mdns` - 启用/禁用局域网发现
- `identity_path` - 密钥文件持久化路径

```rust
use nodp2p::AppConfig;
use std::path::PathBuf;

let config = AppConfig::new()
    .with_mdns(true)
    .with_identity_path(PathBuf::from("keypair.bin"));
```

## 常见问题

### Q: 如何持久化节点身份？
A: 使用 `load_or_generate()` 函数和 `identity_path` 配置。

### Q: 可以在同一台机器上运行多个节点吗？
A: 可以，每个节点会监听不同的端口。

### Q: 如何监控节点性能？
A: 目前暂未提供性能监控 API，但可以通过日志了解网络状态。

## 性能建议

- 使用发布版本构建以获得最佳性能
- 对于大文件传输，建议增加块大小（当前为 256KB）
- 在资源受限的环境中，可以禁用 mDNS

## 贡献

欢迎提交 Issue 和 Pull Request！

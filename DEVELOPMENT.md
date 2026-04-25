# 开发指南

## 项目结构

```
src/
├── lib.rs                 # 库的主入口
├── main.rs                # 示例二进制
└── network/
    ├── mod.rs            # 模块声明和文档
    ├── behaviour.rs      # libp2p 网络行为定义
    ├── command.rs        # 网络命令类型
    ├── config.rs         # 配置选项
    ├── event.rs          # 网络事件类型
    ├── identity.rs       # 密钥管理
    ├── message.rs        # 消息定义
    ├── peer.rs           # 对等体管理
    └── swarm.rs          # P2P swarm 核心逻辑
```

## 核心概念

### Swarm（网络节点）
`start_swarm()` 函数创建并启动一个 P2P 网络节点。该节点：
- 监听 TCP 连接
- 通过 mDNS 发现其他节点
- 支持多种协议（gossipsub、request-response 等）

### 命令通道（Command Channel）
通过 `UnboundedSender<Command>` 与节点交互：
```rust
cmd_tx.send(Command::Broadcast("message".to_string()))?;
```

### 事件通道（Event Channel）
通过 `UnboundedReceiver<AppEvent>` 接收网络事件：
```rust
while let Some(event) = event_rx.recv().await {
    match event {
        AppEvent::MessageReceived { peer, message } => { /* ... */ }
        _ => {}
    }
}
```

## 常见任务

### 发送广播消息
```rust
cmd_tx.send(Command::Broadcast("Hello everyone!".to_string()))?;
```

### 发送私聊消息
```rust
cmd_tx.send(Command::SendPrivateText {
    peer: peer_id,
    text: "Private message".to_string(),
})?;
```

### 发送文件
```rust
cmd_tx.send(Command::SendFile {
    peer: peer_id,
    path: PathBuf::from("./file.txt"),
})?;
```

### 监听事件
```rust
match event {
    AppEvent::PeerConnected(peer) => {
        println!("Node connected: {}", peer);
    }
    AppEvent::MessageReceived { peer, message } => {
        println!("Message: {}", message);
    }
    AppEvent::FileTransferProgress { received, total, .. } => {
        println!("Progress: {}/{}", received, total);
    }
    // ...
}
```

## 扩展指南

### 添加新的协议
1. 在 `behaviour.rs` 中定义新的消息类型
2. 在 `NodBehaviour::new()` 中初始化协议
3. 在 `swarm.rs` 中处理相关事件

### 自定义配置
使用 `AppConfig` 结构体自定义节点行为：
```rust
let config = AppConfig::new()
    .with_mdns(true)
    .with_identity_path(PathBuf::from("./keypair.bin"));
```

## 性能优化

### 发布版本构建
```bash
cargo build --release
```

编译配置已优化以实现最佳性能（LTO、代码生成单位=1）。

### 测试
```bash
cargo test
```

## 依赖管理

主要依赖：
- `libp2p` - P2P 网络框架
- `tokio` - 异步运行时
- `serde` - 序列化框架
- `base64` - 密钥编码

## 文档生成

生成并查看本地文档：
```bash
cargo doc --lib --open
```

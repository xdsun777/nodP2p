# nodP2p - P2P 网络库

一个基于 libp2p 的高性能 P2P 网络库，支持消息广播、私聊和文件传输。

## 特性

- 🚀 **高性能 P2P 网络** - 基于 libp2p 框架
- 📡 **mDNS 节点发现** - 自动发现局域网内的节点
- 💬 **消息广播** - 使用 Gossipsub 协议实现可靠的消息广播
- 🔐 **私聊通信** - 点对点的私密通信
- 📁 **文件传输** - 支持节点间的文件传输，数据流可传递给前端
- 🔑 **密钥管理** - Ed25519 密钥对管理和持久化
- 🖥️ **前端集成** - 特别适合 Tauri 应用，可将二进制数据传递给前端

### 命令行工具

项目包含一个完整的命令行工具，支持以下功能：

```bash
# 查看帮助
cargo run -- --help

# 使用默认设置启动
cargo run

# 指定保存目录和启用详细输出
cargo run -- --save-dir ./downloads --verbose

# 使用持久化密钥
cargo run -- --key-file ./my_key.bin
```

#### 命令行选项

- `-k, --key-file <PATH>`: 密钥文件路径（可选）
- `-s, --save-dir <DIR>`: 文件保存目录（默认: ./received_files）
- `-v, --verbose`: 启用详细输出
- `-L, --listen-addr <ADDR>`: 监听地址（默认: 0.0.0.0）
- `-P, --listen-port <PORT>`: 监听端口（默认: 0，自动分配）

#### 交互命令

启动后支持以下命令：

- **普通输入**: 广播消息到所有节点
- **`/s <peer> <msg>`**: 发送私聊消息
- **`/file <peer> <path>`**: 发送文件
- **`/list`**: 列出已连接节点
- **`/save <transfer_id> <filename>`**: 手动保存接收到的文件
- **`/quit`**: 退出程序

#### 文件自动保存

接收到的文件会自动保存到指定的目录中：

```bash
📁 文件保存目录: ./received_files
💾 文件已自动保存: document.pdf -> ./received_files/document.pdf (1024 bytes)
```

如果自动保存失败，可以使用 `/save` 命令手动保存：

```bash
/save 1 my_document.pdf
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

### 文件传输

文件传输支持两种模式：

#### 实时数据流模式（推荐用于前端集成）
```rust
// 接收数据块事件
AppEvent::FileChunkReceived { transfer_id, offset, data, is_last, .. } => {
    // 实时处理每个数据块，可传递给前端
    // 例如在 Tauri 中：window.emit("file-chunk", { transfer_id, offset, data, is_last })
}

// 接收完整文件
AppEvent::FileReceived { file_name, data, .. } => {
    // 获取完整文件数据，可传递给前端存储到 IDB
    // 例如在 Tauri 中：window.emit("file-received", { file_name, data })
}
```

#### 发送文件
```rust
cmd_tx.send(Command::SendFile {
    peer: peer_id,
    path: PathBuf::from("./document.pdf"),
})?;
```

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

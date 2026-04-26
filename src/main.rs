use clap::Parser;
use nodp2p::{start_swarm, AppEvent, Command};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{self, AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt};
use libp2p::PeerId;

/// P2P 网络节点命令行工具
#[derive(Parser)]
#[command(name = "nodp2p")]
#[command(about = "一个基于 libp2p 的 P2P 网络节点")]
#[command(version)]
struct Args {
    /// 密钥文件路径（可选）
    #[arg(short, long)]
    key_file: Option<PathBuf>,

    /// 文件保存目录
    #[arg(short, long, default_value = "./received_files")]
    save_dir: PathBuf,

    /// 是否启用详细输出
    #[arg(short, long)]
    verbose: bool,

    /// 监听地址
    #[arg(short = 'L', long, default_value = "0.0.0.0")]
    listen_addr: String,

    /// 监听端口（0表示自动分配）
    #[arg(short = 'P', long, default_value = "0")]
    listen_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 创建保存目录
    fs::create_dir_all(&args.save_dir).await?;
    println!("📁 文件保存目录: {}", args.save_dir.display());

    // 加载或生成密钥
    let key = if let Some(key_path) = &args.key_file {
        if key_path.exists() {
            println!("🔑 加载密钥文件: {}", key_path.display());
            let data = fs::read(key_path).await?;
            libp2p::identity::Keypair::from_protobuf_encoding(&data)?
        } else {
            println!("🔑 生成新密钥并保存到: {}", key_path.display());
            let key = libp2p::identity::Keypair::generate_ed25519();
            let data = key.to_protobuf_encoding()?;
            fs::write(key_path, data).await?;
            key
        }
    } else {
        println!("🔑 使用临时密钥（不保存）");
        libp2p::identity::Keypair::generate_ed25519()
    };

    let peer_id = key.public().to_peer_id();
    println!("🆔 本节点ID: {}", peer_id);

    let (cmd_tx, mut event_rx) = start_swarm(key).await?;
    println!("🚀 节点启动完成");
    println!("--------------------------------");
    println!("普通输入              = 群发消息");
    println!("/s <peer> <msg>       = 私聊消息");
    println!("/file <peer> <path>   = 发送文件");
    println!("/list                 = 列出已连接节点");
    println!("/quit                 = 退出程序");
    println!("--------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut incoming_files: HashMap<(PeerId, u64), IncomingFile> = HashMap::new();
    let mut next_transfer_id: u64 = 0;

    struct IncomingFile {
        file: tokio::fs::File,
        temp_path: PathBuf,
        final_path: PathBuf,
        original_file_name: String,
        total_size: u64,
    }

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match event {
                    AppEvent::PeerDiscovered(peer, addr) => {
                        if args.verbose {
                            println!("🔍 发现节点: {} @ {}", peer, addr);
                        }
                    }
                    AppEvent::PeerConnected(peer) => {
                        println!("🔗 节点已连接: {}", peer);
                    }
                    AppEvent::PeerDisconnected(peer) => {
                        println!("🔌 节点断开: {}", peer);
                    }
                    AppEvent::MessageReceived { peer, message } => {
                        println!("💬 广播消息 [{}]: {}", peer, message);
                    }
                    AppEvent::PrivateText(peer, text) => {
                        println!("🔒 [私聊] {}: {}", peer, text);
                    }
                    AppEvent::FileRequestReceived { peer, file_name, file_size, transfer_id } => {
                        println!("📥 收到文件请求: {} ({} bytes) 来自 {}", file_name, file_size, peer);
                        // 自动接受文件请求
                        println!("✅ 自动接受文件传输");

                        let safe_name = Path::new(&file_name)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let temp_path = args.save_dir.join(format!("{}_{}_{}.part", peer, transfer_id, safe_name));
                        let final_path = args.save_dir.join(&file_name);
                        if let Some(parent) = final_path.parent() {
                            fs::create_dir_all(parent).await?;
                        }
                        let file = fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(&temp_path)
                            .await?;

                        incoming_files.insert(
                            (peer, transfer_id),
                            IncomingFile {
                                file,
                                temp_path,
                                final_path,
                                original_file_name: file_name.clone(),
                                total_size: file_size,
                            },
                        );
                    }
                    AppEvent::FileTransferStarted { peer, file_name, transfer_id } => {
                        println!("📁 文件传输开始: {} -> {} [ID: {}]", peer, file_name, transfer_id);
                    }
                    AppEvent::FileTransferProgress { transfer_id, peer, received, total } => {
                        if args.verbose {
                            println!("📊 传输进度 [{}] {}: {}/{} ({:.1}%)",
                                transfer_id, peer, received, total,
                                (received as f64 / total as f64) * 100.0);
                        } else {
                            // 简单的进度条
                            let progress = (received as f64 / total as f64 * 20.0) as usize;
                            let bar = "█".repeat(progress) + &"░".repeat(20 - progress);
                            println!("📊 [{}] {}: [{}] {}/{}", transfer_id, peer, bar, received, total);
                        }
                    }
                    AppEvent::FileChunkReceived { peer, transfer_id, offset, data, is_last } => {
                        if args.verbose {
                            println!("📦 接收数据块 [{}] offset={} size={} is_last={}",
                                transfer_id, offset, data.len(), is_last);
                        }

                        if let Some(incoming) = incoming_files.get_mut(&(peer, transfer_id)) {
                            incoming.file.seek(std::io::SeekFrom::Start(offset)).await?;
                            incoming.file.write_all(&data).await?;
                            if is_last {
                                incoming.file.flush().await?;
                            }
                        } else {
                            println!("⚠️ 未找到接收文件记录: transfer_id={} peer={}", transfer_id, peer);
                        }
                    }
                    AppEvent::FileReceived { peer, transfer_id, file_name: _, data: _ } => {
                        if let Some(incoming) = incoming_files.remove(&(peer, transfer_id)) {
                            fs::rename(&incoming.temp_path, &incoming.final_path).await?;
                            println!("💾 文件已接收并保存: {} -> {} ({} bytes)",
                                incoming.original_file_name, incoming.final_path.display(), incoming.total_size);
                        } else {
                            println!("⚠️ 接收完成但未找到临时文件记录: transfer_id={} peer={}", transfer_id, peer);
                        }
                    }
                    AppEvent::FileSent { peer, transfer_id } => {
                        println!("📤 文件发送完成: transfer_id={} -> {}", transfer_id, peer);
                    }
                }
            }

            Ok(Some(line)) = stdin.next_line() => {
                if let Err(e) = handle_input(line, &cmd_tx, &mut next_transfer_id).await {
                    println!("❌ 命令处理错误: {}", e);
                }
            }
        }
    }
}

async fn handle_input(
    line: String,
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>,
    next_transfer_id: &mut u64,
) -> anyhow::Result<()> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(());
    }

    // 退出命令
    if line == "/quit" || line == "/q" {
        println!("👋 正在退出...");
        std::process::exit(0);
    }

    // 列出已连接节点
    if line == "/list" || line == "/l" {
        // 这里可以添加获取已连接节点列表的逻辑
        // 目前暂时显示帮助信息
        println!("📋 已连接节点列表功能待实现");
        return Ok(());
    }

    // 私聊消息
    if line.starts_with("/s ") {
        let mut parts = line.splitn(3, ' ');
        parts.next(); // 跳过 /s
        let peer_str = parts.next().ok_or_else(|| anyhow::anyhow!("用法: /s <peer_id> <消息>"))?;
        let text = parts.next().ok_or_else(|| anyhow::anyhow!("缺少消息内容"))?;

        let peer_id: PeerId = peer_str.parse()?;
        cmd_tx.send(Command::SendPrivateText {
            peer: peer_id,
            text: text.to_string(),
        })?;
        println!("✅ 私聊发送 -> {}: {}", peer_id, text);
        return Ok(());
    }

    // 发送文件
    if line.starts_with("/file ") {
        let mut parts = line.splitn(3, ' ');
        parts.next(); // 跳过 /file
        let peer_str = parts.next().ok_or_else(|| anyhow::anyhow!("用法: /file <peer_id> <文件路径>"))?;
        let path_str = parts.next().ok_or_else(|| anyhow::anyhow!("缺少文件路径"))?;

        let peer_id: PeerId = peer_str.parse()?;
        let path = PathBuf::from(path_str);

        if !path.exists() {
            println!("❌ 文件不存在: {}", path.display());
            return Ok(());
        }

        let data = fs::read(&path).await?;
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let file_size = data.len() as u64;
        let transfer_id = *next_transfer_id;
        *next_transfer_id += 1;

        cmd_tx.send(Command::SendFileRequest {
            peer: peer_id,
            transfer_id,
            file_name: file_name.clone(),
            file_size,
        })?;

        let chunk_size: usize = 256 * 1024;
        let mut offset: u64 = 0;
        while (offset as usize) < data.len() {
            let end = std::cmp::min(offset as usize + chunk_size, data.len());
            let chunk = data[offset as usize..end].to_vec();
            let is_last = end == data.len();
            cmd_tx.send(Command::SendFileChunk {
                transfer_id,
                peer: peer_id,
                offset,
                data: chunk,
                is_last,
            })?;
            offset = end as u64;
        }

        println!("📤 开始发送文件到: {} ({} bytes, transfer_id={})", peer_id, file_size, transfer_id);
        return Ok(());
    }

    // 广播消息（默认）
    cmd_tx.send(Command::Broadcast(line.to_string()))?;
    println!("📢 广播消息已发送");

    Ok(())
}
use clap::Parser;
use nodp2p::{start_swarm, AppEvent, Command};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{self, AsyncBufReadExt};
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
    println!("/save <transfer_id> <filename> = 手动保存文件");
    println!("/quit                 = 退出程序");
    println!("--------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut received_files: HashMap<u64, (String, Vec<u8>)> = HashMap::new();

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
                    AppEvent::FileRequestReceived { peer, file_name, file_size, transfer_id: _ } => {
                        println!("📥 收到文件请求: {} ({} bytes) 来自 {}", file_name, file_size, peer);
                        // 自动接受文件请求
                        println!("✅ 自动接受文件传输");
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
                    AppEvent::FileChunkReceived { peer: _, transfer_id, offset, data, is_last } => {
                        if args.verbose {
                            println!("📦 接收数据块 [{}] offset={} size={} is_last={}",
                                transfer_id, offset, data.len(), is_last);
                        }
                        // 存储数据块（实际应用中可能需要更复杂的组装逻辑）
                    }
                    AppEvent::FileReceived { peer: _, file_name, data } => {
                        let transfer_id = received_files.len() as u64 + 1;
                        received_files.insert(transfer_id, (file_name.clone(), data.clone()));

                        // 自动保存文件
                        let save_path = args.save_dir.join(&file_name);
                        match fs::write(&save_path, &data).await {
                            Ok(_) => {
                                println!("💾 文件已自动保存: {} -> {} ({} bytes)",
                                    file_name, save_path.display(), data.len());
                            }
                            Err(e) => {
                                println!("❌ 文件保存失败: {} - {}", save_path.display(), e);
                                println!("💡 可以使用 /save {} <filename> 手动保存", transfer_id);
                            }
                        }
                    }
                    AppEvent::FileSent { peer, transfer_id } => {
                        println!("📤 文件发送完成: transfer_id={} -> {}", transfer_id, peer);
                    }
                }
            }

            Ok(Some(line)) = stdin.next_line() => {
                if let Err(e) = handle_input(line, &cmd_tx, &received_files, &args.save_dir).await {
                    println!("❌ 命令处理错误: {}", e);
                }
            }
        }
    }
}

async fn handle_input(
    line: String,
    cmd_tx: &tokio::sync::mpsc::UnboundedSender<Command>,
    received_files: &HashMap<u64, (String, Vec<u8>)>,
    save_dir: &PathBuf,
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

    // 手动保存文件
    if line.starts_with("/save ") {
        let mut parts = line.splitn(3, ' ');
        parts.next(); // 跳过 /save
        let transfer_id_str = parts.next().ok_or_else(|| anyhow::anyhow!("缺少传输ID"))?;
        let filename = parts.next().unwrap_or("unknown_file");

        let transfer_id: u64 = transfer_id_str.parse()?;

        if let Some((original_name, data)) = received_files.get(&transfer_id) {
            let save_path = save_dir.join(filename);
            fs::write(&save_path, data).await?;
            println!("💾 文件已保存: {} -> {} ({} bytes)",
                original_name, save_path.display(), data.len());
        } else {
            println!("❌ 未找到传输ID: {}", transfer_id);
            println!("💡 可用的传输ID: {}", received_files.keys()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", "));
        }
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

        cmd_tx.send(Command::SendFile {
            peer: peer_id,
            path,
        })?;
        println!("📤 开始发送文件到: {}", peer_id);
        return Ok(());
    }

    // 广播消息（默认）
    cmd_tx.send(Command::Broadcast(line.to_string()))?;
    println!("📢 广播消息已发送");

    Ok(())
}
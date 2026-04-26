use libp2p::PeerId;
use std::path::PathBuf;

/// 网络命令，用于控制节点的操作
#[derive(Debug)]
pub enum Command {
    /// 广播消息到所有已连接的节点
    Broadcast(String),
    
    /// 向指定节点发送私聊文本消息
    SendPrivateText { 
        /// 目标节点ID
        peer: PeerId, 
        /// 消息内容
        text: String 
    },
    
    /// 向指定节点发送文件（路径方式，仅 CLI 版保留）
    SendFile { 
        /// 目标节点ID
        peer: PeerId, 
        /// 文件路径
        path: PathBuf 
    },

    /// 向指定节点发送文件请求（使用已经读取好的字节流）
    SendFileRequest {
        /// 目标节点ID
        peer: PeerId,
        /// 传输ID
        transfer_id: u64,
        /// 文件名
        file_name: String,
        /// 文件大小
        file_size: u64,
    },
    
    /// 发送文件块（直接传递字节内容）
    SendFileChunk {
        /// 传输ID
        transfer_id: u64,
        /// 目标节点ID
        peer: PeerId,
        /// 文件偏移
        offset: u64,
        /// 数据块内容
        data: Vec<u8>,
        /// 是否为最后一块
        is_last: bool,
    },
}
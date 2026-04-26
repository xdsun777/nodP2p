use libp2p::{Multiaddr, PeerId};
use serde::Serialize;

/// 网络事件，表示网络中发生的各种事件
#[derive(Debug, Serialize)]
pub enum AppEvent {
    /// 新的节点成功连接
    PeerConnected(PeerId),
    
    /// 节点连接已断开
    PeerDisconnected(PeerId),
    
    /// 通过 mDNS 发现了新节点
    PeerDiscovered(PeerId, Multiaddr),
    
    /// 接收到来自某个节点的广播消息
    MessageReceived { 
        /// 发送者节点ID
        peer: PeerId, 
        /// 消息内容
        message: String 
    },
    
    /// 接收到来自某个节点的私聊文本消息
    PrivateText(PeerId, String),
    
    /// 收到文件转移请求
    FileRequestReceived { 
        /// 发送者节点ID
        peer: PeerId, 
        /// 传输ID
        transfer_id: u64, 
        /// 文件名
        file_name: String, 
        /// 文件大小（字节）
        file_size: u64 
    },
    
    /// 文件传输已启动
    FileTransferStarted { 
        /// 目标节点ID
        peer: PeerId, 
        /// 传输ID
        transfer_id: u64, 
        /// 文件名
        file_name: String 
    },
    
    /// 文件传输进度更新
    FileTransferProgress { 
        /// 目标节点ID
        peer: PeerId, 
        /// 传输ID
        transfer_id: u64, 
        /// 已接收字节
        received: u64, 
        /// 总字节数
        total: u64 
    },
    
    /// 接收到文件数据块
    FileChunkReceived {
        /// 发送者节点ID
        peer: PeerId,
        /// 传输ID
        transfer_id: u64,
        /// 数据块偏移
        offset: u64,
        /// 数据块内容
        data: Vec<u8>,
        /// 是否为最后一块
        is_last: bool,
    },
    
    /// 文件接收完成（包含完整数据）
    FileReceived { 
        /// 发送者节点ID
        peer: PeerId, 
        /// 传输ID
        transfer_id: u64,
        /// 文件名
        file_name: String, 
        /// 完整文件数据（用于传递给前端）
        data: Vec<u8> 
    },
    
    /// 文件发送完成
    FileSent { 
        /// 目标节点ID
        peer: PeerId, 
        /// 传输ID
        transfer_id: u64 
    },
}
use serde::{Deserialize, Serialize};

/// 网络消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetMessage {
    /// 消息类型
    pub msg_type: MessageType,
    /// 发送者昵称
    pub nickname: String,
    /// 消息内容
    pub content: String,
}

/// 消息类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 用户加入网络
    Join,
    /// 聊天消息
    Chat,
    /// 用户离开网络
    Leave,
    /// 文件传输
    FileTransfer,
}

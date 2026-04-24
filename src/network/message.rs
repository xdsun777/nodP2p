use serde::{Serialize, Deserialize};

/// 所有网络消息统一结构（IM协议核心）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetMessage {
    /// 消息类型
    pub msg_type: MessageType,

    /// 昵称
    pub nickname: String,

    /// 内容
    pub content: String,
}

/// 消息类型（扩展点）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Join,
    Chat,
    Leave,
}
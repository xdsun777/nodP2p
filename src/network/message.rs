use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetMessage {
    pub msg_type: MessageType,
    pub nickname: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Join,
    Chat,
    Leave,
    FileTransfer,
}

//! 网络模块
//! 
//! 包含 P2P 网络的所有核心功能，包括 swarm 管理、对等体管理、消息传递等。

/// 网络行为及事件处理
pub mod behaviour;

/// 网络命令定义
pub mod command;

/// 网络配置选项
pub mod config;

/// 网络事件定义
pub mod event;

/// 节点身份和密钥管理
pub mod identity;

/// 网络消息类型
pub mod message;

/// 对等体管理器
pub mod peer;

/// P2P 网络 Swarm 管理
pub mod swarm;

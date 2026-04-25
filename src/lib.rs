#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// 网络相关模块
pub mod network;

// 核心公共 API
pub use network::command::Command;
pub use network::event::AppEvent;
pub use network::message::{MessageType, NetMessage};
pub use network::swarm::start_swarm;
pub use network::identity::{create_key, de_key, load_or_generate};
pub use network::config::AppConfig;
pub use network::peer::PeerManager;



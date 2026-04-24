use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppConfig {
    // 是否启用局域网发现（mDNS）
    pub enable_mdns: bool,

    // 是否持久化密钥
    pub identity_path: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            identity_path: None,
        }
    }
}

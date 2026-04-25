use std::path::PathBuf;

/// 网络应用配置选项
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// 是否启用局域网发现（mDNS）
    pub enable_mdns: bool,

    /// 密钥持久化存储路径，若为 None 则不持久化
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

impl AppConfig {
    /// 创建一个新的配置对象
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置密钥存储路径
    pub fn with_identity_path(mut self, path: PathBuf) -> Self {
        self.identity_path = Some(path);
        self
    }

    /// 设置是否启用 mDNS
    pub fn with_mdns(mut self, enabled: bool) -> Self {
        self.enable_mdns = enabled;
        self
    }
}

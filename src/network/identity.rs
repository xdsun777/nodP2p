use libp2p::identity;
use std::{fs, path::Path};
use base64::Engine;

/// 从指定路径加载密钥对，如果不存在则生成新的
///
/// # Arguments
/// * `path` - 密钥文件路径
///
/// # Example
/// ```no_run
/// let keypair = nodp2p::network::identity::load_or_generate(Some(Path::new("keypair.bin")));
/// ```
pub fn load_or_generate(path: Option<&Path>) -> identity::Keypair {
    if let Some(path) = path {
        if path.exists() {
            let data = fs::read(path).expect("读取密钥失败");
            return identity::Keypair::from_protobuf_encoding(&data).expect("解析密钥失败");
        }
    }

    // 不存在就生成
    let key = identity::Keypair::generate_ed25519();

    if let Some(path) = path {
        let data = key.to_protobuf_encoding().unwrap();
        fs::write(path, data).expect("写入密钥失败");
    }

    key
}

/// 生成新的 Ed25519 密钥对
///
/// # Returns
/// 返回 `(peer_id, base64_encoded_key)`
///
/// # Errors
/// 当密钥编码失败时返回错误
///
/// # Example
/// ```ignore
/// let (peer_id, key_base64) = nodp2p::network::identity::create_key()?;
/// println!("Peer ID: {}", peer_id);
/// ```
pub fn create_key() -> Result<(String, String), String> {
    // 生成 ed25519 密钥对
    let key = identity::Keypair::generate_ed25519();
    let peer_id = key.public().to_peer_id().to_string();
    let key_bytes = key.to_protobuf_encoding().map_err(|e| e.to_string())?;
    let key_base64 = base64::engine::general_purpose::STANDARD.encode(key_bytes);

    Ok((peer_id, key_base64))
}

/// 从 Base64 编码的字符串解码密钥对
///
/// # Arguments
/// * `key_base64` - Base64 编码的密钥
///
/// # Returns
/// 返回解码后的密钥对
///
/// # Panics
/// 当密钥格式无效时会panic
///
/// # Example
/// ```ignore
/// let keypair = nodp2p::network::identity::de_key(key_base64);
/// ```
pub fn de_key(key_base64: String) -> identity::Keypair {
    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&key_base64)
        .expect("密钥格式错误");
    identity::Keypair::from_protobuf_encoding(&key_bytes).expect("密钥解析失败")
}
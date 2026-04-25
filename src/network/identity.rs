use libp2p::identity;
use std::{fs, path::Path};

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

pub fn create_key() -> Result<(String, String), String> {
    // 生成 ed25519 密钥对
    let key = identity::Keypair::generate_ed25519();
    let peer_id = key.public().to_peer_id().to_string();
    let key_bytes = key.to_protobuf_encoding().map_err(|e| e.to_string())?;
    let key_base64 = base64::encode(key_bytes);

    Ok((peer_id, key_base64))
}

pub fn de_key(key_base64: String) -> identity::Keypair {
    let key_bytes = base64::decode(&key_base64).map_err(|_| "密钥格式错误").unwrap();
    let key = identity::Keypair::from_protobuf_encoding(&key_bytes).map_err(|e| e.to_string()).unwrap();
    key
}
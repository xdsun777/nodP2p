use libp2p::identity;
use std::{fs, path::Path};

pub fn load_or_generate(path: Option<&Path>) -> identity::Keypair {

    if let Some(path) = path {
        if path.exists() {
            let data = fs::read(path).expect("读取密钥失败");
            return identity::Keypair::from_protobuf_encoding(&data)
                .expect("解析密钥失败");
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
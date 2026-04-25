/// Tauri 应用中使用 nodp2p 的示例
///
/// 这个示例展示了如何在 Tauri 应用中集成 nodp2p 库，
/// 并将接收到的文件数据传递给前端页面。

use nodp2p::{start_swarm, AppEvent, Command};
use libp2p::identity::Keypair;
use tauri::Manager;

// 学习如何在 Tauri 中使用：
// https://tauri.app/v1/guides/features/command

#[tauri::command]
async fn start_p2p_node(app: tauri::AppHandle) -> Result<(), String> {
    let key = Keypair::generate_ed25519();
    let (cmd_tx, mut event_rx) = start_swarm(key).await
        .map_err(|e| format!("Failed to start swarm: {}", e))?;

    // 在后台处理 P2P 事件
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::FileChunkReceived { transfer_id, offset, data, is_last, .. } => {
                    // 将数据块发送给前端
                    let _ = app.emit("file-chunk", serde_json::json!({
                        "transfer_id": transfer_id,
                        "offset": offset,
                        "data": data,
                        "is_last": is_last
                    }));
                }
                AppEvent::FileReceived { file_name, data, .. } => {
                    // 将完整文件数据发送给前端
                    let _ = app.emit("file-received", serde_json::json!({
                        "file_name": file_name,
                        "data": data
                    }));
                }
                AppEvent::PeerConnected(peer) => {
                    let _ = app.emit("peer-connected", peer.to_string());
                }
                // 处理其他事件...
                _ => {}
            }
        }
    });

    Ok(())
}

#[tauri::command]
async fn send_file(cmd_tx: tauri::State<'_, tokio::sync::mpsc::UnboundedSender<Command>>,
                   peer_id: String,
                   file_path: String) -> Result<(), String> {
    let peer = peer_id.parse()
        .map_err(|_| "Invalid peer ID")?;

    cmd_tx.send(Command::SendFile {
        peer,
        path: std::path::PathBuf::from(file_path),
    }).map_err(|_| "Failed to send command")?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(tokio::sync::mpsc::unbounded_channel::<Command>().0)
        .invoke_handler(tauri::generate_handler![
            start_p2p_node,
            send_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
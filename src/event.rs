use libp2p::{mdns, ping, swarm::SwarmEvent, Swarm};

use crate::behaviour::NodBehaviour;

// =============================
//  事件统一处理
// =============================
pub async fn handle_event(
    event: SwarmEvent<
        <NodBehaviour as libp2p::swarm::NetworkBehaviour>::ToSwarm,
        impl std::error::Error,
    >,
    swarm: &mut Swarm<NodBehaviour>,
) {
    match event {
        // =============================
        // 新连接建立
        // =============================
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            println!("已连接到节点: {}", peer_id);
        }
        //  监听地址
        SwarmEvent::NewListenAddr { address, .. } => {
            println!("监听地址: {}", address);
        }
        // =============================
        // mDNS 发现节点
        // =============================
        SwarmEvent::Behaviour(event) => match event {
            // 处理 mDNS
            crate::behaviour::NodBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                for (peer, addr) in list {
                    println!("发现节点: {} {}", peer, addr);

                    // 自动连接
                    swarm.add_external_address(addr);
                }
            }

            crate::behaviour::NodBehaviourEvent::Mdns(mdns::Event::Expired(_)) => {
                // 节点过期（可处理断开逻辑）
            }

            // 处理 ping
            crate::behaviour::NodBehaviourEvent::Ping(ping::Event { peer, .. }) => {
                println!("Ping: {}", peer);
            }

            // =============================
            // TODO: 在这里扩展其他协议事件
            // =============================
            _ => {}
        },

        _ => {}
    }
}

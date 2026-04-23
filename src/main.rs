// 引入所需的 libp2p 组件
use libp2p::{
    core::upgrade,
    identity, noise, tcp, yamux, Swarm, Transport,
    swarm::{SwarmEvent, Config as SwarmConfig},
    mdns, ping, // 引入 mdns 和 ping 行为
    futures::StreamExt,
    Multiaddr,
};
use std::error::Error;

// 自定义网络行为：同时包含 mdns 和 ping
// 使用派生宏自动生成 NetworkBehaviour 实现
#[derive(libp2p::swarm::NetworkBehaviour)]
#[behaviour(out_event = "MyBehaviourEvent")]
struct MyBehaviour {
    // mdns 节点发现
    mdns: mdns::tokio::Behaviour,
    // ping 协议（保持连接活性，不涉及应用消息）
    ping: ping::Behaviour,
}

// 自定义行为产生的事件类型（用于在事件循环中处理 mdns 和 ping 产生的事件）
#[derive(Debug)]
enum MyBehaviourEvent {
    Mdns(mdns::Event),
    Ping(ping::Event),
}

// 将 mdns 的事件类型转换为自定义事件
impl From<mdns::Event> for MyBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        MyBehaviourEvent::Mdns(event)
    }
}

// 将 ping 的事件类型转换为自定义事件
impl From<ping::Event> for MyBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        MyBehaviourEvent::Ping(event)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. 生成 ed25519 密钥对和 PeerId
    let identity_key = identity::Keypair::generate_ed25519();
    let peer_id = identity_key.public().to_peer_id();
    println!("本地 PeerId: {}", peer_id);

    // 2. 创建 TCP 传输层（tokio 运行时）
    let tcp_transport = tcp::tokio::Transport::new(tcp::Config::default());

    // 3. Noise 加密配置（基于身份密钥）
    let noise_config = noise::Config::new(&identity_key)?;

    // 4. 组合传输层：TCP → Noise 加密 → Yamux 多路复用
    let transport = tcp_transport
        .upgrade(upgrade::Version::V1)          // 协议升级版本
        .authenticate(noise_config)             // 加密认证
        .multiplex(yamux::Config::default())    // 多路复用
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    // 5. 创建 mdns 行为（使用 tokio 运行时，默认配置）
    let mdns_behaviour = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
    
    // 6. 创建 ping 行为（默认配置）
    let ping_behaviour = ping::Behaviour::default();

    // 7. 组合成自定义行为
    let behaviour = MyBehaviour {
        mdns: mdns_behaviour,
        ping: ping_behaviour,
    };

    // 8. Swarm 配置（使用 tokio 执行器，设置空闲连接超时）
    let swarm_config = SwarmConfig::with_tokio_executor()
        .with_idle_connection_timeout(std::time::Duration::from_secs(60));

    // 9. 构建 Swarm
    let mut swarm = Swarm::new(transport, behaviour, peer_id, swarm_config);

    // 10. 监听所有网络接口的随机端口（用于 mdns 及后续连接）
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("正在监听，等待节点发现...");

    // 11. 事件循环
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("监听地址: {}", address);
            }
            // 处理自定义行为产生的事件
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns_event)) => {
                match mdns_event {
                    mdns::Event::Discovered(list) => {
                        // 发现新节点（list 可能包含多个）
                        for (peer_id, multiaddr) in list {
                            println!("发现新节点: {} (地址: {})", peer_id, multiaddr);
                        }
                    }
                    mdns::Event::Expired(list) => {
                        // 节点离线（到期）
                        for (peer_id, multiaddr) in list {
                            println!("节点离线: {} (地址: {})", peer_id, multiaddr);
                        }
                    }
                }
            }
            // 忽略 ping 事件（不输出，避免刷屏）
            SwarmEvent::Behaviour(MyBehaviourEvent::Ping(_)) => {}
            // 可选：处理其他 swarm 事件（如连接建立/关闭等，按需可取消注释）
            // SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            //     println!("连接已建立: {}", peer_id);
            // }
            // SwarmEvent::ConnectionClosed { peer_id, .. } => {
            //     println!("连接已关闭: {}", peer_id);
            // }
            _ => {}
        }
    }
}
use libp2p::{
    core::upgrade,
    identity, noise, tcp, yamux, Transport,
};
use std::time::Duration;

/// 构建 libp2p 传输层
///
/// TCP + Noise 加密 + Yamux 多路复用。
/// Noise 协议提供 TLS 级别的加密，使流量看起来像普通 TLS。
/// 这是"和光同尘"在传输层的体现。
pub fn build_transport(
    keypair: &identity::Keypair,
) -> std::io::Result<libp2p::core::transport::Boxed<(libp2p::PeerId, libp2p::core::muxing::StreamMuxerBox)>> {
    let noise = noise::Config::new(keypair)
        .expect("noise config");

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise)
        .multiplex(yamux::Config::default())
        .timeout(Duration::from_secs(20))
        .boxed();

    Ok(transport)
}

use libp2p::{identity, PeerId};
use libp2p::identity::PublicKey;

pub fn GetFixPeer() -> PublicKey {
    // 从固定的字节数组生成密钥
    let mut raw_key = [0u8; 32];
    let keypair = identity::Keypair::ed25519_from_bytes(&mut raw_key).unwrap();

// 从公钥生成 Peer ID
    let peer_id = PeerId::from(keypair.public());

    println!("Peer ID: {}", peer_id.to_base58());
    println!("hello");
    keypair.public()
}
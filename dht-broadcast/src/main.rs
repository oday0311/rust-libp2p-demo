mod util;

use std::error::Error;
use futures::prelude::*;
use libp2p::{
    core::{multiaddr::Multiaddr, upgrade::Version},
    noise,identity,identify,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};

use libp2p::kad::{Kademlia, KademliaConfig};
use libp2p::kad::store::MemoryStore;

//https://github.com/libp2p/rust-libp2p/blob/master/examples/identify/src/main.rs
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let other_peer_key = util::GetFixPeer();
    println!("hello");


    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {local_peer_id:?}");
    // 本地节点的 PeerId

    //Kademlia DHT 的默认配置
    let kad_config = KademliaConfig::default();

    // 创建 Kademlia DHT 协议
    // Create a Kademlia behaviour.
    let store = MemoryStore::new(local_peer_id.clone());
    let kad_proto = Kademlia::with_config(local_peer_id.clone(),store, kad_config);

    let transport = tcp::async_io::Transport::default()
        .upgrade(Version::V1Lazy)
        .authenticate(noise::Config::new(&local_key).unwrap())
        .multiplex(yamux::Config::default())
        .boxed();

    // Create a identify network behaviour.
    let behaviour = identify::Behaviour::new(identify::Config::new(
        "/ipfs/id/1.0.0".to_string(),
        local_key.public(),
    ));

    let mut swarm =
        SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build();

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
            // Prints peer id identify info is being sent to.
            SwarmEvent::Behaviour(identify::Event::Sent { peer_id, .. }) => {
                println!("Sent identify info to {peer_id:?}")
            }
            // Prints out the info received via the identify event
            SwarmEvent::Behaviour(identify::Event::Received { info, .. }) => {
                println!("Received {info:?}")
            }
            _ => {}
        }
    }


    //Ok(())
}
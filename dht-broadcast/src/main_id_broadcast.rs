mod util;

use futures::StreamExt;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{GetClosestPeersError, Kademlia, KademliaConfig, KademliaEvent, QueryResult};
use libp2p::{
     identity,
    swarm::{SwarmBuilder, SwarmEvent},
    PeerId,
};
use std::{env, error::Error, thread, time::Duration};
use std::str::FromStr;
use libp2p;
use libp2p::swarm::behaviour;
use multiaddr::Multiaddr;

const BOOTNODES: [&str; 5] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
    "QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
];

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    util::GetFixPeer();
    env_logger::init();

    let search_peerid = PeerId::from_str("12D3KooWRNw2pJC9748Fmq4WNV27HoSTcX3r37132FLkQMrbKAiC").unwrap();

    // Create a random key for ourselves.
    let local_key = identity::Keypair::generate_ed25519();
    let mut local_peer_id = PeerId::from(local_key.public());
    local_peer_id = search_peerid;

    println!("Local peer id: {:?}", local_peer_id);

    // Set up a an encrypted DNS-enabled TCP Transport over the yamux protocol
    let transport = libp2p::development_transport(local_key).await?;

    // Create a swarm to manage peers and events.
    let mut swarm = {
        // Create a Kademlia behaviour.
        let mut cfg = KademliaConfig::default();
        cfg.set_query_timeout(Duration::from_secs(15 * 60));
        let store = MemoryStore::new(local_peer_id);
        let mut behaviour = Kademlia::with_config(local_peer_id, store, cfg);

        // Add the bootnodes to the local routing table. `libp2p-dns` built
        // into the `transport` resolves the `dnsaddr` when Kademlia tries
        // to dial these nodes.
        for peer in &BOOTNODES {
            behaviour.add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
        }

        behaviour.add_address(&mut local_peer_id, "/ip4/127.0.0.1/tcp/8080".parse()?);


        SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build()
    };

    // Order Kademlia to search for a peer.
    let to_search = env::args()
        .nth(1)
        .map(|p| p.parse())
        .transpose()?
        .unwrap_or_else(PeerId::random);



    // Listen on all interfaces and whatever port the OS assigns.
    swarm.listen_on("/ip4/127.0.0.1/tcp/8080".parse()?)?;

    println!("Searching for the closest peers to {to_search}");
    swarm.behaviour_mut().get_closest_peers(to_search);

    loop {
        //println!("Waiting for next event");
        let event = swarm.select_next_some().await;
        if let SwarmEvent::Behaviour(KademliaEvent::OutboundQueryProgressed {
                                         result: QueryResult::GetClosestPeers(result),
                                         ..
                                     }) = event
        {

            match result {
                Ok(ok) => {
                    if !ok.peers.is_empty() {
                        println!("Query finished with closest peers: {:#?}", ok.peers)
                    } else {
                        // The example is considered failed as there
                        // should always be at least 1 reachable peer.
                        println!("Query finished with no closest peers.")
                    }
                }
                Err(GetClosestPeersError::Timeout { peers, .. }) => {
                    if !peers.is_empty() {
                        println!("Query timed out with closest peers: {peers:#?}")
                    } else {
                        // The example is considered failed as there
                        // should always be at least 1 reachable peer.
                        println!("Query timed out with no closest peers.");
                    }

                }
            };



            thread::sleep(Duration::from_secs(5));
            {
                swarm.behaviour_mut().get_closest_peers(to_search);
            }
            //break;


        }
    }

    Ok(())
}
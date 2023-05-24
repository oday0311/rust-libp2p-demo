

use async_std::io;
use futures::{prelude::*, select};
use libp2p::core::upgrade::Version;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{GetClosestPeersResult,GetClosestPeersOk,
                  record::Key, AddProviderOk, GetProvidersOk, GetRecordOk, Kademlia, KademliaEvent, PeerRecord,
                  PutRecordOk, QueryResult, Quorum, Record,
};
use libp2p::{identity, mdns, noise, swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent}, tcp, yamux, PeerId, Transport, Swarm, gossipsub};
use std::error::Error;
use std::str::FromStr;
use std::thread;
use std::time::Duration;


// We create a custom network behaviour that combines Kademlia and mDNS.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyBehaviourEvent")]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    kademlia: Kademlia<MemoryStore>,
    mdns: mdns::async_io::Behaviour,

}

#[allow(clippy::large_enum_variant)]
enum MyBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Kademlia(KademliaEvent),
    Mdns(mdns::Event),
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Create a random key for ourselves.
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("local_peer_id is {:?}", local_peer_id);

    let transport = tcp::async_io::Transport::default()
        .upgrade(Version::V1Lazy)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    // Create a Gosspipsub topic
    let gossipsub_topic = gossipsub::IdentTopic::new("chat");


    impl From<gossipsub::Event> for MyBehaviourEvent {
        fn from(event: gossipsub::Event) -> Self {
            MyBehaviourEvent::Gossipsub(event)
        }
    }
    impl From<KademliaEvent> for MyBehaviourEvent {
        fn from(event: KademliaEvent) -> Self {
            MyBehaviourEvent::Kademlia(event)
        }
    }

    impl From<mdns::Event> for MyBehaviourEvent {
        fn from(event: mdns::Event) -> Self {
            MyBehaviourEvent::Mdns(event)
        }
    }

    // Create a swarm to manage peers and events.
    let mut swarm = {
        // Create a Kademlia behaviour.
        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::new(local_peer_id, store);
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id)?;


        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .max_transmit_size(262144)
            .build()
            .expect("valid config");

        let mut  gossipub_instance =  gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config).expect("Valid configuration");
        gossipub_instance.subscribe(&gossipsub_topic).expect("TODO: panic message, subscribe");;

        let mut behaviour = MyBehaviour {
            gossipsub: gossipub_instance,
            kademlia:kademlia,
            mdns:mdns };

        SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build()
    };

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    // Listen on all interfaces and whatever port the OS assigns.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;


    // Kick it off.
    loop {
        println!("current msg is 1");
        select! {
        line = stdin.select_next_some() => handle_input_line(
                &mut swarm,
                line.expect("Stdin not to close")),
        event = swarm.select_next_some() => match event {


            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening in {address:?}");
            },
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                        println!("Discovered {:?} {:?}", peer_id, multiaddr);
                    //swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                }
            }

                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },

                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("Connection established to {}", peer_id);
                    //swarm.behaviour_mut().gossipsub.publish(gossipsub_topic.clone(), "Hello".as_bytes());
                    //swarm.send_message(peer_id, "Hello".to_string()).unwrap();
                             //thread::sleep(Duration::from_secs(1));

                }

                SwarmEvent::IncomingConnection { .. } => {
                    println!("Incoming Connection established");
                }

                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => {
                        println!(
                            "Got message: {} with id: {} from peer: {:?}",
                            String::from_utf8_lossy(&message.data),
                            id,
                            peer_id
                        )
                }
            SwarmEvent::Behaviour(MyBehaviourEvent::Kademlia(KademliaEvent::OutboundQueryProgressed { result, ..})) => {
                match result {
                    QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders { key, providers, .. })) => {
                        for peer in providers {
                            println!(
                                "Peer {peer:?} provides key {:?}",
                                std::str::from_utf8(key.as_ref()).unwrap()
                            );
                        }
                    }
                    QueryResult::GetProviders(Err(err)) => {
                        eprintln!("Failed to get providers: {err:?}");
                    }
                    QueryResult::GetRecord(Ok(
                        GetRecordOk::FoundRecord(PeerRecord {
                            record: Record { key, value, .. },
                            ..
                        })
                    )) => {
                        println!(
                            "Got record {:?} {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );
                    }
                    QueryResult::GetRecord(Ok(_)) => {}
                    QueryResult::GetRecord(Err(err)) => {
                        eprintln!("Failed to get record: {err:?}");
                    }
                    QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                        println!(
                            "Successfully put record {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    QueryResult::PutRecord(Err(err)) => {
                        eprintln!("Failed to put record: {err:?}");
                    }
                    QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                        println!(
                            "Successfully put provider record {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    QueryResult::StartProviding(Err(err)) => {
                        eprintln!("Failed to put provider record: {err:?}");
                    }
                        //QueryResult::GetClosestPeers(result
                    QueryResult::GetClosestPeers(Ok(GetClosestPeersOk { key, peers, .. })) => {
                        println!(
                            "Closest peers to key {:?}",
                            peers
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        }
    }
}

fn handle_input_line(swarm: &mut Swarm<MyBehaviour>,  line: String) {

    // Create a Gosspipsub topic
    let gossipsub_topic = gossipsub::IdentTopic::new("chat");

    let mut args = line.split(' ');
    let kademlia = &mut swarm.behaviour_mut().kademlia;
    match args.next() {
        Some("send") => {
            let msg = {
                match args.next() {
                    Some(msg) => msg,
                    None => {
                        eprintln!("Expected message: send msg peer");
                        return;
                    }
                }
            };
            let peer = {
                match args.next() {
                    Some(peer) => PeerId::from_str(peer).unwrap(),
                    None => {
                        eprintln!("Expected peer: send msg peer");
                        return;
                    }
                }
            };
            //kademlia.send_message(peer, msg.as_bytes());
            //swarm.dial(peer).expect("Failed to dial peer.");

            swarm.behaviour_mut().gossipsub.publish(gossipsub_topic.clone(), "hell0".as_bytes()).expect("TODO: panic message");


        }

        Some("query") => {
            let key = {
                match args.next() {
                    Some(key) => key,
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };

            let search_peerid = PeerId::from_str(key).unwrap();

            kademlia.get_closest_peers(search_peerid);

        }
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            kademlia.get_record(key);
        }
        Some("GET_PROVIDERS") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            kademlia.get_providers(key);
        }
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            let value = {
                match args.next() {
                    Some(value) => value.as_bytes().to_vec(),
                    None => {
                        eprintln!("Expected value");
                        return;
                    }
                }
            };
            let record = Record {
                key,
                value,
                publisher: None,
                expires: None,
            };
            kademlia
                .put_record(record, Quorum::One)
                .expect("Failed to store record locally.");
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };

            kademlia
                .start_providing(key)
                .expect("Failed to start providing key");
        }
        _ => {
            eprintln!("expected GET, GET_PROVIDERS, PUT or PUT_PROVIDER");
        }
    }
}

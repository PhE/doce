use std::{collections::HashMap, sync::Mutex, time::Duration};

use bevy::prelude::*;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubConfigBuilder, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    PeerId, Swarm,
};

use crate::player::Player;

pub struct Party {
    pub players: [Option<Player>; 4],
    pub peers: HashMap<PeerId, usize>,
    pub host_index: usize,
}

impl Party {
    pub fn new(host_player: Player) -> Self {
        Self {
            players: [Some(host_player), None, None, None],
            peers: HashMap::new(),
            host_index: 0,
        }
    }

    pub fn join(&mut self, player: Player) -> usize {
        for i in 0..4 {
            if let None = self.players[i] {
                self.players[i] = Some(player);
                return i;
            }
        }

        panic!("Maximum party limit reached!");
    }

    pub fn leave(&mut self, player_index: usize) {
        self.players[player_index].as_ref().unwrap();
        self.players[player_index] = None;

        for i in (player_index + 1)..(player_index + 5) {
            if let Some(_) = self.players[i % 4] {
                self.host_index = i;
                return;
            }
        }
    }
}

pub struct PartyNetwork {
    pub swarm: Mutex<Swarm<Gossipsub>>,
}

impl PartyNetwork {
    pub fn new() -> Self {
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from_public_key(local_key.public());
        info!("Local peer ID: {:?}", local_peer_id);

        let transport =
            libp2p::futures::executor::block_on(libp2p::development_transport(local_key.clone()))
                .unwrap();

        // Create a Gossipsub topic
        let topic = IdentTopic::new("test-net");

        // Create a Swarm to manage peers and events
        let mut swarm = {
            // Set a custom gossipsub
            let gossipsub_config = GossipsubConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                .build()
                .expect("Valid config");
            // build a gossipsub network behaviour
            let mut gossipsub: Gossipsub =
                Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                    .expect("Correct configuration");

            // subscribes to our topic
            gossipsub.subscribe(&topic).unwrap();

            // build the swarm
            libp2p::Swarm::new(transport, gossipsub, local_peer_id)
        };

        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
        let swarm = Mutex::new(swarm);

        Self { swarm }
    }
}

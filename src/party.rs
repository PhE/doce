use std::collections::HashMap;

use libp2p::PeerId;

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

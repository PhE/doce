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
}

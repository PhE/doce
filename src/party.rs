use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::player::{Player, PlayerId};

#[derive(Serialize, Deserialize)]
pub struct Party {
    pub players: HashMap<PlayerId, Player>,
    pub host_id: PlayerId,
}

impl Party {
    pub fn new(host_player: Player) -> Self {
        let host_id = host_player.id;
        let mut players = HashMap::new();
        players.insert(host_player.id, host_player);

        Self { players, host_id }
    }
}

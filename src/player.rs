use std::str::FromStr;

use libp2p::PeerId;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(PeerId);

impl PlayerId {
    pub fn new(peer_id: PeerId) -> Self {
        Self(peer_id)
    }
}

impl From<PeerId> for PlayerId {
    fn from(peer_id: PeerId) -> Self {
        Self(peer_id)
    }
}

impl Into<PeerId> for PlayerId {
    fn into(self) -> PeerId {
        self.0
    }
}

impl Serialize for PlayerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for PlayerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PlayerIdVisitor)
    }
}

struct PlayerIdVisitor;

impl de::Visitor<'_> for PlayerIdVisitor {
    type Value = PlayerId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Base58 string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match PeerId::from_str(v) {
            Ok(peer_id) => Ok(PlayerId::new(peer_id)),
            Err(error) => Err(E::custom(format!("cannot parse PeerId: {:?}", error))),
        }
    }
}

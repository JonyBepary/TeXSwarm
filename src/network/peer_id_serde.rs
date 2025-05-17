use libp2p::PeerId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// Serialize a PeerId as a string
pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let peer_id_str = peer_id.to_string();
    peer_id_str.serialize(serializer)
}

/// Deserialize a PeerId from a string
pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
where
    D: Deserializer<'de>,
{
    let peer_id_str = String::deserialize(deserializer)?;
    PeerId::from_str(&peer_id_str).map_err(serde::de::Error::custom)
}

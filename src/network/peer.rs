use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

mod peer_id_serde {
    include!("peer_id_serde.rs");
}

mod instant_serde {
    include!("instant_serde.rs");
}

/// Information about a peer in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID in the libp2p network
    #[serde(with = "peer_id_serde")]
    pub peer_id: PeerId,

    /// User ID associated with this peer
    pub user_id: Option<String>,

    /// Peer's displayable name
    pub display_name: Option<String>,

    /// Document IDs the peer is currently editing
    pub active_documents: Vec<String>,

    /// Multiaddresses where this peer can be reached
    #[serde(skip)]
    pub addresses: Vec<String>,

    /// When this peer was last seen
    #[serde(with = "instant_serde")]
    pub last_seen: Instant,
}

impl PeerInfo {
    /// Create a new PeerInfo
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            user_id: None,
            display_name: None,
            active_documents: Vec::new(),
            addresses: Vec::new(),
            last_seen: Instant::now(),
        }
    }

    /// Update the last_seen timestamp
    pub fn mark_seen(&mut self) {
        self.last_seen = Instant::now();
    }

    /// Check if the peer is considered active (seen within timeout)
    pub fn is_active(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() < timeout
    }
}

/// Collection of peers in the network
#[derive(Debug, Clone)]
pub struct PeerRegistry {
    peers: HashMap<PeerId, PeerInfo>,
    active_timeout: Duration,
}

impl PeerRegistry {
    /// Create a new peer registry with the given active timeout
    pub fn new(active_timeout: Duration) -> Self {
        Self {
            peers: HashMap::new(),
            active_timeout,
        }
    }

    /// Add or update a peer in the registry
    pub fn update_peer(&mut self, peer_id: PeerId) -> &mut PeerInfo {
        self.peers.entry(peer_id).or_insert_with(|| PeerInfo::new(peer_id))
    }

    /// Add a peer to the registry
    pub fn add_peer(&mut self, peer_id: PeerId) -> &mut PeerInfo {
        // Same implementation as update_peer
        self.update_peer(peer_id)
    }

    /// Get information about a peer
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    /// Get a mutable reference to a peer
    pub fn get_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerInfo> {
        self.peers.get_mut(peer_id)
    }

    /// Remove a peer from the registry
    pub fn remove_peer(&mut self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }

    /// Get all peers in the registry
    pub fn all_peers(&self) -> impl Iterator<Item = &PeerInfo> {
        self.peers.values()
    }

    /// Get all active peers in the registry
    pub fn active_peers(&self) -> impl Iterator<Item = &PeerInfo> {
        self.peers.values().filter(move |p| p.is_active(self.active_timeout))
    }

    /// Get peers currently editing a specific document
    pub fn document_peers(&self, document_id: &str) -> impl Iterator<Item = &PeerInfo> {
        self.peers.values().filter(move |p| p.active_documents.contains(&document_id.to_string()))
    }

    /// Mark a peer as editing a document
    pub fn mark_editing(&mut self, peer_id: &PeerId, document_id: &str) -> bool {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.mark_seen();
            if !peer.active_documents.contains(&document_id.to_string()) {
                peer.active_documents.push(document_id.to_string());
                return true;
            }
        }
        false
    }

    /// Clean up inactive peers
    pub fn cleanup_inactive(&mut self) {
        let inactive: Vec<PeerId> = self.peers.iter()
            .filter(|(_, info)| !info.is_active(self.active_timeout))
            .map(|(id, _)| id.clone())
            .collect();

        for peer_id in inactive {
            self.peers.remove(&peer_id);
        }
    }
}

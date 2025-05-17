use anyhow::Result;
use libp2p::{
    core::Multiaddr,
    kad,
    kad::{store::MemoryStore, QueryResult},
    mdns::{self},
    swarm::NetworkBehaviour,
    PeerId,
};
use std::collections::HashSet;
use std::time::Duration;

use crate::utils::errors::AppError;

/// Combined discovery behavior
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "DiscoveryEvent")]
pub struct DiscoveryBehavior {
    /// Kademlia DHT for peer discovery
    kademlia: kad::Kademlia<MemoryStore>,
    /// mDNS for local network discovery
    mdns: mdns::tokio::Behaviour,
}

/// Events from the discovery behavior
#[derive(Debug)]
pub enum DiscoveryEvent {
    /// Kademlia events
    Kademlia(kad::KademliaEvent),
    /// mDNS events
    Mdns(mdns::Event),
}

impl From<kad::KademliaEvent> for DiscoveryEvent {
    fn from(event: kad::KademliaEvent) -> Self {
        DiscoveryEvent::Kademlia(event)
    }
}

impl From<mdns::Event> for DiscoveryEvent {
    fn from(event: mdns::Event) -> Self {
        DiscoveryEvent::Mdns(event)
    }
}

/// Discovery service for finding peers
pub struct DiscoveryService {
    /// Local peer ID
    local_peer_id: PeerId,
    /// Discovered peers
    discovered_peers: HashSet<PeerId>,
}

impl DiscoveryService {
    /// Create a new discovery service
    pub async fn new(local_peer_id: PeerId) -> Result<Self> {
        Ok(Self {
            local_peer_id,
            discovered_peers: HashSet::new(),
        })
    }

    /// Create a new discovery behavior
    pub async fn create_behavior(&self) -> Result<DiscoveryBehavior> {
        // Set up Kademlia
        let mut kademlia_config = kad::KademliaConfig::default();
        kademlia_config.set_query_timeout(Duration::from_secs(30));
        let store = MemoryStore::new(self.local_peer_id);
        let kademlia = kad::Kademlia::with_config(self.local_peer_id, store, kademlia_config);

        // Set up mDNS
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), self.local_peer_id)
            .map_err(|e| AppError::NetworkError(format!("Failed to create mDNS: {}", e)))?;

        Ok(DiscoveryBehavior { kademlia, mdns })
    }

    /// Add a bootstrap node to Kademlia
    pub fn add_bootstrap_node(&self, behavior: &mut DiscoveryBehavior, peer_id: PeerId, addr: Multiaddr) {
        behavior.kademlia.add_address(&peer_id, addr);
    }

    /// Bootstrap the Kademlia DHT
    pub fn bootstrap(&self, behavior: &mut DiscoveryBehavior) -> Result<()> {
        behavior.kademlia.bootstrap()
            .map_err(|e| AppError::NetworkError(format!("Failed to bootstrap Kademlia: {}", e)))?;
        Ok(())
    }

    /// Handle a discovery event
    pub fn handle_event(&mut self, event: DiscoveryEvent) -> Vec<PeerId> {
        let mut new_peers = Vec::new();

        match event {
            DiscoveryEvent::Kademlia(event) => {
                if let kad::KademliaEvent::OutboundQueryProgressed { result, .. } = event {
                    match result {
                        QueryResult::Bootstrap(Ok(bootstrap_result)) => {
                            if !self.discovered_peers.contains(&bootstrap_result.peer) {
                                self.discovered_peers.insert(bootstrap_result.peer);
                                new_peers.push(bootstrap_result.peer);
                            }
                        }
                        QueryResult::GetClosestPeers(Ok(closest_peers_result)) => {
                            for peer in closest_peers_result.peers {
                                if !self.discovered_peers.contains(&peer) {
                                    self.discovered_peers.insert(peer);
                                    new_peers.push(peer);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            DiscoveryEvent::Mdns(event) => {
                if let mdns::Event::Discovered(list) = event {
                    for (peer, _) in list {
                        if !self.discovered_peers.contains(&peer) {
                            self.discovered_peers.insert(peer);
                            new_peers.push(peer);
                        }
                    }
                }
            }
        }

        new_peers
    }
}

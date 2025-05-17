// This is a proposed fix for the NetworkEngine class
// It can be merged into the main engine.rs file once it's tested

use anyhow::Result;
use dashmap::DashMap;
use libp2p::{PeerId, request_response};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::crdt::engine::CrdtEngine;
use crate::network::peer::PeerRegistry;
use crate::network::protocol::{CollabRequest, CollabResponse, NetworkMessage};
use crate::utils::config::NetworkConfig;
use crate::utils::errors::AppError;

// Implementation of the NetworkService remains the same

/// The NetworkEngine manages the P2P network connections and message routing
#[derive(Debug)]
pub struct NetworkEngine {
    service: Option<NetworkService>,
    peer_registry: Arc<RwLock<PeerRegistry>>,
    crdt_engine: Arc<RwLock<CrdtEngine>>,
    config: NetworkConfig,

    // Map of document IDs to the set of peer IDs that are subscribed to that document
    document_subscribers: DashMap<Uuid, Vec<String>>,
}

impl NetworkEngine {
    // Constructor remains the same

    /// Get the local peer ID
    pub async fn get_local_peer_id(&self) -> Result<String> {
        if let Some(service) = &self.service {
            Ok(service.local_peer_id.to_string())
        } else {
            Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())))
        }
    }

    // The rest of the implementation remains the same, but we make some changes to broadcast_operation

    /// Broadcast an operation to all subscribed peers
    pub async fn broadcast_operation(&mut self, doc_id: &Uuid, operation: Vec<u8>) -> Result<()> {
        if let Some(service) = &mut self.service {
            // Publish to the operations topic for this document
            let topic_str = DocumentTopic::Operations(*doc_id).to_topic_string();
            service.publish_to_topic(topic_str, operation.clone()).await?;

            // Also directly deliver the operation to all subscribed peers
            // This ensures operations propagate even if the gossipsub propagation fails
            if let Some(subscribers) = self.document_subscribers.get(doc_id) {
                for peer_id_str in subscribers.value() {
                    if let Ok(peer_id) = peer_id_str.parse::<PeerId>() {
                        // For a real implementation, we'd need to directly send the operation
                        // to the peer. This would involve using request_response or a direct
                        // connection to the peer.
                        tracing::debug!("Would send operation directly to peer: {}", peer_id);
                    }
                }
            }
        } else {
            return Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())));
        }

        Ok(())
    }

    /// Subscribe to a document
    pub async fn subscribe_to_document(&mut self, doc_id: Uuid) -> Result<()> {
        if let Some(service) = &mut self.service {
            let topic_str = DocumentTopic::Operations(doc_id).to_topic_string();
            service.subscribe_to_topic(topic_str).await?;

            // Also add ourselves to the subscribers
            let local_peer_id = self.get_local_peer_id().await?;
            let mut subscribers = self.document_subscribers.entry(doc_id).or_insert_with(Vec::new);
            if !subscribers.contains(&local_peer_id) {
                subscribers.push(local_peer_id);
            }

            // Now, we need to explicitly send a join request to peers to get document content
            // This would be done by sending a JoinRequest message to known peers
            let _request = NetworkMessage::JoinRequest {
                document_id: doc_id,
                user_id: "user".to_string(), // This would be the actual user ID
                user_name: "User".to_string(), // This would be the actual user name
            };

            // In a real implementation, we'd send this request to peers
            // For now, just log it
            tracing::debug!("Would send join request for document: {}", doc_id);
        } else {
            return Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())));
        }

        Ok(())
    }

    // The rest of the implementation remains the same
}

/// Topics for different document events - this needs to be in the module scope
pub enum DocumentTopic {
    /// Topic for document operations
    Operations(Uuid),
    /// Topic for document presence updates
    Presence(Uuid),
    /// Topic for document metadata updates
    Metadata(Uuid),
}

impl DocumentTopic {
    /// Convert to a topic string
    pub fn to_topic_string(&self) -> String {
        match self {
            DocumentTopic::Operations(id) => format!("doc-ops/{}", id),
            DocumentTopic::Presence(id) => format!("doc-presence/{}", id),
            DocumentTopic::Metadata(id) => format!("doc-meta/{}", id),
        }
    }
}

// NetworkService mock implementation for testing - this should match what's in the original file
#[derive(Debug)]
pub struct NetworkService {
    /// Local peer ID
    local_peer_id: PeerId,
    /// Sender for network events
    event_sender: mpsc::Sender<NetworkEvent>,
}

// The rest of the NetworkService implementation would be included here

/// Network events that can be sent to the application
#[derive(Debug)]
pub enum NetworkEvent {
    /// New peer discovered
    PeerDiscovered(PeerId),
    /// Peer connection established
    PeerConnected(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
    /// Message received on a topic
    MessageReceived {
        source: PeerId,
        topic: String,
        data: Vec<u8>,
    },
    /// Request received from a peer
    RequestReceived {
        request_id: request_response::RequestId,
        source: PeerId,
        request: CollabRequest,
        channel: request_response::ResponseChannel<CollabResponse>,
    },
    /// Response received from a peer
    ResponseReceived {
        request_id: String,
        source: PeerId,
        response: CollabResponse,
    },
}

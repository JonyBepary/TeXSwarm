use anyhow::Result;
use dashmap::DashMap;
// Remove the unused GossipsubTopic import
use libp2p::{PeerId, request_response};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::crdt::engine::CrdtEngine;
use crate::network::peer::PeerRegistry;
use crate::network::protocol::{CollabRequest, CollabResponse, NetworkMessage};
use crate::utils::config::NetworkConfig;
use crate::utils::errors::AppError;

// Define the types locally in this module to avoid import issues
/// Topics for different document events
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

/// Network service for P2P communication
#[derive(Debug)]
pub struct NetworkService {
    /// Local peer ID
    local_peer_id: PeerId,
    /// Sender for network events
    event_sender: mpsc::Sender<NetworkEvent>,
}

// Implement methods needed by the engine
impl NetworkService {
    pub async fn new(_config: NetworkConfig) -> Result<Self> {
        // Create a sample peer ID for development
        let local_peer_id = PeerId::random();

        // Create event channels
        let (event_sender, _) = mpsc::channel(100);

        Ok(Self {
            local_peer_id,
            event_sender,
        })
    }

    pub fn take_event_receiver(&mut self) -> mpsc::Receiver<NetworkEvent> {
        // Return a placeholder receiver
        let (_, rx) = mpsc::channel(100);
        rx
    }

    pub async fn publish_to_topic(&mut self, _topic_str: String, _data: Vec<u8>) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    pub async fn subscribe_to_topic(&mut self, _topic_str: String) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    pub async fn unsubscribe_from_topic(&mut self, _topic_str: String) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    pub async fn send_response(
        &mut self,
        _channel: request_response::ResponseChannel<CollabResponse>,
        _response: NetworkMessage
    ) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
}

impl Clone for NetworkService {
    fn clone(&self) -> Self {
        Self {
            local_peer_id: self.local_peer_id,
            event_sender: self.event_sender.clone(),
        }
    }
}

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
    pub async fn new(config: &NetworkConfig, crdt_engine: Arc<RwLock<CrdtEngine>>) -> Result<Self> {
        // Create a peer registry with the specified timeout duration
        let peer_registry = Arc::new(RwLock::new(PeerRegistry::new(std::time::Duration::from_secs(300))));

        Ok(Self {
            service: None,
            peer_registry: peer_registry.clone(),
            crdt_engine,
            config: config.clone(),
            document_subscribers: dashmap::DashMap::new(),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Initialize the network service with the configuration
        let service = NetworkService::new(self.config.clone()).await?;
        self.service = Some(service);

        // Start the main network event loop as a background task
        self.start_event_loop().await?;

        Ok(())
    }

    
    /// Get the local peer ID
    pub async fn get_local_peer_id(&self) -> Result<String> {
        if let Some(service) = &self.service {
            Ok(service.local_peer_id.to_string())
        } else {
            Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())))
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        // Network service will be dropped when Option is cleared
        self.service = None;
        Ok(())
    }

    /// Get the local peer ID
    pub async fn get_local_peer_id(&self) -> Result<String> {
        if let Some(service) = &self.service {
            Ok(service.local_peer_id.to_string())
        } else {
            Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())))
        }
    }

    async fn start_event_loop(&mut self) -> Result<()> {
        if let Some(service) = &mut self.service {
            // Get event receiver from the service
            let mut event_receiver = service.take_event_receiver();
            let peer_registry = Arc::clone(&self.peer_registry);
            let crdt_engine = self.crdt_engine.clone();
            let document_subscribers = self.document_subscribers.clone();
            let mut service_clone = service.clone();

            // Spawn the event loop as a background task
            tokio::spawn(async move {
                while let Some(event) = event_receiver.recv().await {
                    match event {
                        // Handle received messages
                        NetworkEvent::MessageReceived { source: _, topic, data } => {
                            let topic_str = topic.clone();
                            // Parse the topic string to identify document and event type
                            if let Some(topic_parts) = topic_str.strip_prefix("doc-ops/") {
                                if let Ok(doc_id) = Uuid::parse_str(topic_parts) {
                                    let engine = crdt_engine.read().await;
                                    if let Err(e) = engine.apply_remote_operation(&doc_id, &data).await {
                                        tracing::warn!("Failed to apply remote operation: {}", e);
                                    }
                                }
                            }
                        },
                        NetworkEvent::RequestReceived { request_id: _, source, request, channel } => {
                            match request.0 {
                                NetworkMessage::JoinRequest { document_id, user_id: _, user_name: _ } => {
                                    // Handle document join request
                                    let engine = crdt_engine.read().await;
                                    let content = match engine.get_document_content(&document_id).await {
                                        Ok(content) => Some(content),
                                        Err(_) => None,
                                    };

                                    // Add to document subscribers
                                    let mut subs = document_subscribers.entry(document_id).or_insert_with(Vec::new);
                                    if !subs.contains(&source.to_string()) {
                                        subs.push(source.to_string());
                                    }

                                    // Send response
                                    let response = NetworkMessage::JoinResponse {
                                        document_id,
                                        success: true,
                                        error_message: None,
                                        document_content: content,
                                    };

                                    if let Err(e) = service_clone.send_response(channel, response).await {
                                        tracing::warn!("Failed to send join response: {}", e);
                                    }
                                },
                                NetworkMessage::SyncRequest { document_id, user_id: _, version: _ } => {
                                    // Handle document sync request
                                    let _engine = crdt_engine.read().await;
                                    // This would need to retrieve operations since the version vector
                                    let response = NetworkMessage::SyncResponse {
                                        document_id,
                                        operations: Vec::new(), // Would need implementation to get ops
                                        is_full_sync: true,
                                    };

                                    if let Err(e) = service_clone.send_response(channel, response).await {
                                        tracing::warn!("Failed to send sync response: {}", e);
                                    }
                                },
                                _ => {
                                    tracing::warn!("Unhandled request type");
                                }
                            }
                        },
                        NetworkEvent::PeerConnected(peer_id) => {
                            let mut registry = peer_registry.write().await;
                            registry.add_peer(peer_id);
                        },
                        NetworkEvent::PeerDisconnected(peer_id) => {
                            // Remove peer from registry and all document subscribers
                            let mut registry = peer_registry.write().await;
                            registry.remove_peer(&peer_id);

                            // Remove peer from all document subscribers
                            for mut item in document_subscribers.iter_mut() {
                                let subs = item.value_mut();
                                subs.retain(|id| id != &peer_id.to_string());
                            }
                        },
                        _ => {}
                    }
                }
            });
        } else {
            return Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())));
        }

        Ok(())
    }

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

    pub async fn subscribe_to_document(&mut self, doc_id: Uuid) -> Result<()> {
        if let Some(service) = &mut self.service {
            let topic_str = DocumentTopic::Operations(doc_id).to_topic_string();
            service.subscribe_to_topic(topic_str).await?;
            
            // Add ourselves to the document subscribers
            let local_peer_id = self.get_local_peer_id().await?;
            let mut subscribers = self.document_subscribers.entry(doc_id).or_insert_with(Vec::new);
            if !subscribers.contains(&local_peer_id) {
                subscribers.push(local_peer_id);
            }
            
            // Request document content from any connected peer that has it
            self.request_document_sync(doc_id).await?;
        } else {
            return Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())));
        }

        Ok(())
    }
    
    /// Request document synchronization from peers
    async fn request_document_sync(&self, doc_id: Uuid) -> Result<()> {
        // This would normally broadcast a sync request to all peers
        // For the mock implementation, we'll just log the request
        tracing::debug!("Requesting document sync for document: {}", doc_id);
        Ok(())
    }

    /// Unsubscribe from a document
    pub async fn unsubscribe_from_document(&mut self, doc_id: Uuid) -> Result<()> {
        if let Some(service) = &mut self.service {
            let topic_str = DocumentTopic::Operations(doc_id).to_topic_string();
            service.unsubscribe_from_topic(topic_str).await?;
        } else {
            return Err(anyhow::anyhow!(AppError::NetworkError("Network service not initialized".to_string())));
        }

        Ok(())
    }
    
    /// Get the number of connected peers
    pub async fn get_connected_peer_count(&self) -> Result<usize> {
        let registry = self.peer_registry.read().await;
        Ok(registry.active_peers().count())
    }
    
    /// Get the connected peers
    pub async fn get_connected_peers(&self) -> Result<Vec<String>> {
        let registry = self.peer_registry.read().await;
        Ok(registry.active_peers().map(|p| p.peer_id.to_string()).collect())
    }
    
    /// Get all subscribed documents
    pub async fn get_subscribed_documents(&self) -> Result<Vec<Uuid>> {
        Ok(self.document_subscribers.iter().map(|entry| *entry.key()).collect())
    }
}

// NetworkEvent is now imported from swarm.rs

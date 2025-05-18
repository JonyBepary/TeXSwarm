//! This module provides a wrapper around different NetworkService implementations
//! It allows the NetworkEngine to use either the mock or real implementation

use anyhow::Result;
use libp2p::{PeerId, request_response};
use std::sync::Arc;
use tokio::sync::mpsc;
use super::protocol::{CollabResponse, NetworkMessage};

/// Wrapper around different NetworkService implementations
#[derive(Debug)]
pub enum NetworkServiceWrapper {
    /// Mock implementation for testing
    Mock(super::engine::NetworkService),

    /// Real implementation for production
    #[allow(dead_code)]
    Real(Arc<super::service::RealNetworkService>, mpsc::Receiver<super::service::NetworkEvent>),
}

impl NetworkServiceWrapper {
    /// Get the local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        match self {
            NetworkServiceWrapper::Mock(service) => service.local_peer_id,
            NetworkServiceWrapper::Real(service, _) => service.local_peer_id,
        }
    }

    /// Take the event receiver
    pub fn take_event_receiver(&mut self) -> mpsc::Receiver<super::engine::NetworkEvent> {
        match self {
            NetworkServiceWrapper::Mock(service) => service.take_event_receiver(),
            NetworkServiceWrapper::Real(_, receiver) => {
                // Replace the receiver with a dummy one
                let (_, dummy_rx) = mpsc::channel(1);
                let mut original_rx = std::mem::replace(receiver, dummy_rx);

                // Create a new channel for converting events
                let (tx, rx) = mpsc::channel(100);

                // Spawn a task to convert and forward events
                tokio::spawn(async move {
                    while let Some(event) = original_rx.recv().await {
                        // Convert from RealNetworkService::NetworkEvent to engine::NetworkEvent
                        let converted_event = match event {
                            super::service::NetworkEvent::PeerDiscovered(peer_id) =>
                                super::engine::NetworkEvent::PeerDiscovered(peer_id),
                            super::service::NetworkEvent::PeerConnected(peer_id) =>
                                super::engine::NetworkEvent::PeerConnected(peer_id),
                            super::service::NetworkEvent::PeerDisconnected(peer_id) =>
                                super::engine::NetworkEvent::PeerDisconnected(peer_id),
                            super::service::NetworkEvent::MessageReceived { source, topic, data } =>
                                super::engine::NetworkEvent::MessageReceived { source, topic, data },
                            super::service::NetworkEvent::RequestReceived { request_id, source, request, channel } =>
                                super::engine::NetworkEvent::RequestReceived { request_id, source, request, channel },
                            super::service::NetworkEvent::ResponseReceived { request_id, source, response } =>
                                super::engine::NetworkEvent::ResponseReceived { request_id, source, response },
                        };

                        if let Err(e) = tx.send(converted_event).await {
                            tracing::error!("Failed to forward converted event: {}", e);
                            break;
                        }
                    }
                });

                rx
            }
        }
    }

    /// Publish to a topic
    pub async fn publish_to_topic(&mut self, topic_str: String, data: Vec<u8>) -> Result<()> {
        match self {
            NetworkServiceWrapper::Mock(service) => service.publish_to_topic(topic_str, data).await,
            NetworkServiceWrapper::Real(service, _) => service.publish_to_topic(topic_str, data).await,
        }
    }

    /// Subscribe to a topic
    pub async fn subscribe_to_topic(&mut self, topic_str: String) -> Result<()> {
        match self {
            NetworkServiceWrapper::Mock(service) => service.subscribe_to_topic(topic_str).await,
            NetworkServiceWrapper::Real(service, _) => service.subscribe_to_topic(topic_str).await,
        }
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe_from_topic(&mut self, topic_str: String) -> Result<()> {
        match self {
            NetworkServiceWrapper::Mock(service) => service.unsubscribe_from_topic(topic_str).await,
            NetworkServiceWrapper::Real(service, _) => service.unsubscribe_from_topic(topic_str).await,
        }
    }

    /// Send a request to a peer
    pub async fn send_request(
        &mut self,
        peer_id: PeerId,
        request: NetworkMessage,
        request_id: String,
    ) -> Result<()> {
        match self {
            NetworkServiceWrapper::Mock(_service) => {
                // Mock implementation doesn't have this method yet
                Ok(())
            },
            NetworkServiceWrapper::Real(service, _) => service.send_request(peer_id, request, request_id).await,
        }
    }

    /// Send a response
    pub async fn send_response(
        &mut self,
        channel: request_response::ResponseChannel<CollabResponse>,
        response: NetworkMessage
    ) -> Result<()> {
        match self {
            NetworkServiceWrapper::Mock(service) => service.send_response(channel, response).await,
            NetworkServiceWrapper::Real(service, _) => service.send_response(channel, response).await,
        }
    }
}

impl Clone for NetworkServiceWrapper {
    fn clone(&self) -> Self {
        match self {
            NetworkServiceWrapper::Mock(service) => NetworkServiceWrapper::Mock(service.clone()),
            NetworkServiceWrapper::Real(service, _) => {
                // Create a dummy receiver since we can't clone the real one
                let (_, rx) = mpsc::channel(1);
                NetworkServiceWrapper::Real(Arc::clone(service), rx)
            }
        }
    }
}

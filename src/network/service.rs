use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub::{self, self as gossipsub_mod, MessageAuthenticity},
    identity, noise, yamux,
    request_response::{self, self as request_response_mod, ProtocolSupport},
    swarm::{self, SwarmEvent, keep_alive},
    tcp, Multiaddr, PeerId, Transport,
};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use super::protocol::{CollabCodec, CollabProtocol, CollabRequest, CollabResponse, NetworkMessage};
use crate::utils::config::NetworkConfig;
use crate::utils::errors::AppError;

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
pub struct RealNetworkService {
    /// libp2p Swarm
    swarm: Arc<Mutex<swarm::Swarm<MyBehaviour>>>,
    /// Local peer ID
    pub local_peer_id: PeerId,
    /// Subscribed topics
    subscribed_topics: Arc<Mutex<HashSet<String>>>,
    /// Sender for network events
    #[allow(dead_code)]
    event_sender: mpsc::Sender<NetworkEvent>,
    /// Mapping of request IDs to string IDs for tracking responses
    request_ids: Arc<Mutex<HashMap<request_response_mod::RequestId, String>>>,
}

impl std::fmt::Debug for RealNetworkService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealNetworkService")
            .field("local_peer_id", &self.local_peer_id)
            .field("subscribed_topics", &self.subscribed_topics)
            .field("request_ids", &self.request_ids)
            .finish()
    }
}

/// Combined network behavior
#[derive(swarm::NetworkBehaviour)]
pub struct MyBehaviour {
    /// Request-response protocol
    request_response: request_response_mod::Behaviour<CollabCodec>,
    /// Gossipsub for pub/sub messaging
    gossipsub: gossipsub_mod::Behaviour,
    /// Keep-alive to maintain connections
    keep_alive: keep_alive::Behaviour,
}

// From trait implementations for MyBehaviourEvent
impl From<request_response_mod::Event<CollabRequest, CollabResponse>> for MyBehaviourEvent {
    fn from(event: request_response_mod::Event<CollabRequest, CollabResponse>) -> Self {
        MyBehaviourEvent::RequestResponse(event)
    }
}

impl From<gossipsub_mod::Event> for MyBehaviourEvent {
    fn from(event: gossipsub_mod::Event) -> Self {
        MyBehaviourEvent::Gossipsub(event)
    }
}

impl From<void::Void> for MyBehaviourEvent {
    fn from(event: void::Void) -> Self {
        MyBehaviourEvent::KeepAlive(event)
    }
}

impl RealNetworkService {
    /// Create a new network service with the given configuration
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        // Create a keypair for the local node
        let local_key = if let Some(seed) = config.peer_id_seed {
            // Generate deterministic key from seed
            let mut seed_bytes = [0u8; 32];
            let seed_str = seed.as_bytes();
            seed_bytes[..seed_str.len().min(32)].copy_from_slice(&seed_str[..seed_str.len().min(32)]);
            identity::Keypair::ed25519_from_bytes(seed_bytes)?
        } else {
            // Generate random key
            identity::Keypair::generate_ed25519()
        };

        let local_peer_id = PeerId::from(local_key.public());

        // Create event channel
        let (event_sender, _event_receiver) = mpsc::channel(100);

        // Create request-response protocol
        let protocols = vec![(CollabProtocol, ProtocolSupport::Full)];
        let request_response = request_response_mod::Behaviour::new(
            CollabCodec,
            protocols.into_iter(),
            request_response::Config::default()
        );

        // Create gossipsub protocol
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid gossipsub config");

        let gossipsub = gossipsub_mod::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config
        ).expect("Valid gossipsub configuration");

        // Create the swarm
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key).expect("Valid noise config"))
            .multiplex(yamux::Config::default())
            .boxed();

        let behavior = MyBehaviour {
            request_response,
            gossipsub,
            keep_alive: keep_alive::Behaviour::default(),
        };

        let mut swarm = swarm::SwarmBuilder::with_tokio_executor(
            transport,
            behavior,
            local_peer_id
        ).build();

        // Configure the swarm based on the config
        for addr in &config.listen_addresses {
            if let Ok(addr_parsed) = addr.parse::<Multiaddr>() {
                if let Err(e) = swarm.listen_on(addr_parsed.clone()) {
                    tracing::warn!("Failed to listen on {}: {}", addr_parsed, e);
                }
            }
        }

        // Connect to bootstrap nodes
        for node in &config.bootstrap_nodes {
            if let Ok((peer_id, addr)) = parse_peer_and_addr(node) {
                if let Err(e) = swarm.dial(addr) {
                    tracing::warn!("Failed to dial bootstrap node {}: {}", peer_id, e);
                }
            }
        }

        Ok(Self {
            swarm: Arc::new(Mutex::new(swarm)),
            local_peer_id,
            subscribed_topics: Arc::new(Mutex::new(HashSet::new())),
            event_sender,
            request_ids: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start the network service event loop
    pub async fn start_event_loop(self: Arc<Self>) -> Result<mpsc::Receiver<NetworkEvent>> {
        let (event_sender, event_receiver) = mpsc::channel(100);
        let service_clone = self.clone();

        tokio::spawn(async move {
            loop {
                let event = {
                    let mut swarm = service_clone.swarm.lock().await;
                    swarm.select_next_some().await
                };

                match event {
                    SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(event)) => {
                        match event {
                            gossipsub_mod::Event::Message {
                                propagation_source: _,
                                message_id: _,
                                message,
                            } => {
                                if let Some(source_peer) = message.source {
                                    let topic_str = message.topic.to_string();
                                    if let Err(e) = event_sender.send(NetworkEvent::MessageReceived {
                                        source: source_peer,
                                        topic: topic_str,
                                        data: message.data,
                                    }).await {
                                        tracing::error!("Failed to send gossipsub message event: {}", e);
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(event)) => {
                        match event {
                            request_response_mod::Event::Message {
                                peer,
                                message: request_response_mod::Message::Request {
                                    request_id,
                                    request,
                                    channel,
                                }
                            } => {
                                if let Err(e) = event_sender.send(NetworkEvent::RequestReceived {
                                    request_id,
                                    source: peer,
                                    request,
                                    channel,
                                }).await {
                                    tracing::error!("Failed to send request event: {}", e);
                                }
                            },
                            request_response_mod::Event::Message {
                                peer,
                                message: request_response_mod::Message::Response {
                                    request_id,
                                    response
                                }
                            } => {
                                let string_id = {
                                    let request_ids = service_clone.request_ids.lock().await;
                                    request_ids.get(&request_id).cloned().unwrap_or_else(|| request_id.to_string())
                                };

                                if let Err(e) = event_sender.send(NetworkEvent::ResponseReceived {
                                    request_id: string_id,
                                    source: peer,
                                    response,
                                }).await {
                                    tracing::error!("Failed to send response event: {}", e);
                                }
                            },
                            _ => {}
                        }
                    },
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        if let Err(e) = event_sender.send(NetworkEvent::PeerConnected(peer_id)).await {
                            tracing::error!("Failed to send peer connected event: {}", e);
                        }
                    },
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        if let Err(e) = event_sender.send(NetworkEvent::PeerDisconnected(peer_id)).await {
                            tracing::error!("Failed to send peer disconnected event: {}", e);
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        tracing::info!("Listening on {}", address);
                    },
                    _ => {}
                }
            }
        });

        Ok(event_receiver)
    }

    /// Publish a message to a topic
    pub async fn publish_to_topic(&self, topic_str: String, data: Vec<u8>) -> Result<()> {
        // Create a topic hash from the string
        let topic = gossipsub_mod::Sha256Topic::new(topic_str);
        let mut swarm = self.swarm.lock().await;

        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
            return Err(anyhow::anyhow!(AppError::NetworkError(format!("Failed to publish to topic: {}", e))));
        }

        Ok(())
    }

    /// Subscribe to a topic
    pub async fn subscribe_to_topic(&self, topic_str: String) -> Result<()> {
        let topic = gossipsub_mod::Sha256Topic::new(topic_str.clone());
        let mut swarm = self.swarm.lock().await;

        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
            return Err(anyhow::anyhow!(AppError::NetworkError(format!("Failed to subscribe to topic: {}", e))));
        }

        // Add to subscribed topics
        let mut topics = self.subscribed_topics.lock().await;
        topics.insert(topic_str);

        Ok(())
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe_from_topic(&self, topic_str: String) -> Result<()> {
        let topic = gossipsub_mod::Sha256Topic::new(topic_str.clone());
        let mut swarm = self.swarm.lock().await;

        if let Err(e) = swarm.behaviour_mut().gossipsub.unsubscribe(&topic) {
            return Err(anyhow::anyhow!(AppError::NetworkError(format!("Failed to unsubscribe from topic: {}", e))));
        }

        // Remove from subscribed topics
        let mut topics = self.subscribed_topics.lock().await;
        topics.remove(&topic_str);

        Ok(())
    }

    /// Send a request to a peer
    pub async fn send_request(
        &self,
        peer_id: PeerId,
        request: NetworkMessage,
        request_id: String,
    ) -> Result<()> {
        let mut swarm = self.swarm.lock().await;

        let outbound_id = swarm.behaviour_mut().request_response.send_request(
            &peer_id,
            CollabRequest(request),
        );

        // Store the request ID mapping
        let mut request_ids = self.request_ids.lock().await;
        request_ids.insert(outbound_id, request_id);

        Ok(())
    }

    /// Send a response to a request
    pub async fn send_response(
        &self,
        channel: request_response::ResponseChannel<CollabResponse>,
        response: NetworkMessage,
    ) -> Result<()> {
        let mut swarm = self.swarm.lock().await;

        if let Err(e) = swarm.behaviour_mut().request_response.send_response(
            channel,
            CollabResponse(response),
        ) {
            return Err(anyhow::anyhow!(AppError::NetworkError(format!("Failed to send response: {:?}", e))));
        }

        Ok(())
    }
}

/// Parse a peer ID and multiaddress from a string like "/ip4/127.0.0.1/tcp/4001/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N"
fn parse_peer_and_addr(addr_str: &str) -> Result<(PeerId, Multiaddr)> {
    let mut addr = addr_str.parse::<Multiaddr>()?;

    // Extract peer ID from the multiaddress
    if let Some(libp2p::multiaddr::Protocol::P2p(peer_id_bytes)) = addr.pop() {
        let peer_id = PeerId::from_multihash(peer_id_bytes)
            .map_err(|_| anyhow::anyhow!(AppError::NetworkError("Invalid peer ID in multiaddress".to_string())))?;

        return Ok((peer_id, addr));
    }

    Err(anyhow::anyhow!(AppError::NetworkError("Multiaddress does not contain peer ID".to_string())))
}

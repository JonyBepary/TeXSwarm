use anyhow::Result;
use futures::{StreamExt, SinkExt};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use warp::Filter;
use warp::ws::Message as WarpMessage;

// We'll use Warp's WebSocket message type throughout the application
// and provide conversions when needed

use crate::api::protocol::ApiMessage;
use crate::crdt::engine::CrdtEngine;
use crate::crdt::operations::DocumentOperation;
use crate::utils::errors::AppError;

/// User client session information
#[derive(Debug, Clone)]
pub struct ClientSession {
    /// User ID
    pub user_id: String,
    /// Active document ID
    pub document_id: Option<Uuid>,
    /// Channel to send messages to the client
    pub sender: mpsc::Sender<WarpMessage>,
}

/// WebSocket server for real-time communication with clients
#[derive(Clone)]
pub struct WebSocketServer {
    /// CRDT engine
    crdt_engine: Arc<RwLock<CrdtEngine>>,
    /// Active client sessions
    sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(
        crdt_engine: Arc<RwLock<CrdtEngine>>,
    ) -> Self {
        Self {
            crdt_engine,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the WebSocket server
    pub async fn start(&self, config: &crate::utils::config::Config) -> Result<()> {
        // Get config values
        let addr = format!("{}:{}", config.server.ws_host, config.server.ws_port);
        let socket_addr: std::net::SocketAddr = addr.parse()
            .map_err(|e| AppError::ApiError(format!("Invalid WebSocket address: {}", e)))?;

        tracing::info!("WebSocket binding to socket address: {}", socket_addr);

        // Create a clone of relevant resources for the handler closure
        let server_ref = self.clone();

        // Create the WebSocket upgrader with CORS support
        let make_service = warp::serve(
            warp::path("ws")
                .and(warp::ws())
                .and(warp::any().map(move || server_ref.clone()))
                .map(|ws: warp::ws::Ws, server: WebSocketServer| {
                    ws.on_upgrade(move |websocket| handle_websocket_connection(websocket, server))
                })
                // Add CORS support for WebSocket handshake
                .with(warp::cors()
                    .allow_any_origin()
                    .allow_headers(vec!["content-type", "authorization"])
                    .allow_methods(vec!["GET", "POST", "OPTIONS"]))
        );

        // Start the server in a background task
        tokio::spawn(async move {
            make_service.run(socket_addr).await;
        });

        Ok(())
    }

    /// Clone the WebSocketServer
    pub fn clone(&self) -> Self {
        Self {
            crdt_engine: Arc::clone(&self.crdt_engine),
            sessions: Arc::clone(&self.sessions),
        }
    }

    /// Handle an incoming API message
    pub async fn handle_message(&self, session_id: &str, message: ApiMessage) -> Result<Option<ApiMessage>> {
        match message {
            ApiMessage::Authentication { user_id, token: _ } => {
                // In a real system, we would validate the token
                self.register_session(session_id, user_id.clone()).await?;

                // Return a positive authentication response
                Ok(Some(ApiMessage::Error {
                    code: "auth_success".to_string(),
                    message: format!("Authentication successful for user {}", user_id),
                }))
            },

            ApiMessage::DocumentOperation { operation } => {
                // Get the session
                let session = self.get_session(session_id).await?;

                // Convert API operation to CRDT operation
                let crdt_op = match operation {
                    crate::api::protocol::Operation::Insert { document_id, position, content } => {
                        DocumentOperation::Insert {
                            document_id,
                            user_id: session.user_id.clone(),
                            position,
                            content,
                        }
                    },
                    crate::api::protocol::Operation::Delete { document_id, range } => {
                        DocumentOperation::Delete {
                            document_id,
                            user_id: session.user_id.clone(),
                            range,
                        }
                    },
                    crate::api::protocol::Operation::Replace { document_id, range, content } => {
                        DocumentOperation::Replace {
                            document_id,
                            user_id: session.user_id.clone(),
                            range,
                            content,
                        }
                    },
                };

                // Apply the operation
                self.apply_operation(crdt_op).await?;

                Ok(None)
            },

            ApiMessage::OpenDocument { document_id } => {
                // Set the active document for this session
                self.set_active_document(session_id, document_id).await?;

                // Get the document content
                let engine = self.crdt_engine.read().await;
                let content = engine.get_document_content(&document_id).await?;

                // Return the document content
                Ok(Some(ApiMessage::DocumentUpdate {
                    document_id,
                    content,
                    version: "latest".to_string(), // Simplified version handling
                }))
            },

            ApiMessage::CreateDocument { title, repository_url: _ } => {
                // Get the session
                let session = self.get_session(session_id).await?;

                // Create the document
                let engine = self.crdt_engine.read().await;
                let document_id = engine.create_document(title, session.user_id.clone()).await?;

                // Set as active document
                self.set_active_document(session_id, document_id).await?;

                // Return the document ID
                Ok(Some(ApiMessage::DocumentUpdate {
                    document_id,
                    content: "".to_string(),
                    version: "initial".to_string(),
                }))
            },

            ApiMessage::ListDocuments => {
                // Get the session
                let session = self.get_session(session_id).await?;

                // Get the list of documents
                let engine = self.crdt_engine.read().await;
                let documents = engine.list_documents().await?;

                // Here we would typically filter documents by user or create DocumentSummary objects
                // For now we'll just return mock document summaries
                let doc_summaries = documents.iter().map(|_doc| {
                    crate::api::protocol::DocumentSummary {
                        id: Uuid::new_v4(), // Would actually extract from doc
                        title: "Document".to_string(), // Would extract from doc
                        owner: session.user_id.clone(),
                        updated_at: chrono::Utc::now().to_rfc3339(),
                        active_collaborators: 1,
                    }
                }).collect();

                // Return the document list
                Ok(Some(ApiMessage::DocumentList {
                    documents: doc_summaries,
                }))
            },

            ApiMessage::PresenceUpdate { document_id, presence } => {
                // Update the user's presence
                let engine = self.crdt_engine.read().await;
                engine.update_user_presence(document_id, presence).await?;

                // Broadcast to other users
                self.broadcast_presence(document_id).await?;

                Ok(None)
            },

            _ => {
                // Unhandled message type
                Err(AppError::ApiError(format!("Unhandled message type")).into())
            }
        }
    }

    /// Apply a document operation to the CRDT engine
    pub async fn apply_operation(&self, operation: DocumentOperation) -> Result<()> {
        // Get the CRDT engine
        let engine = self.crdt_engine.read().await;

        // Extract the document ID
        let document_id = match &operation {
            DocumentOperation::Insert { document_id, .. } => document_id,
            DocumentOperation::Delete { document_id, .. } => document_id,
            DocumentOperation::Replace { document_id, .. } => document_id,
        };

        // Apply the operation to the CRDT - don't need to recreate the operation
        engine.apply_local_operation(document_id, operation.clone()).await?;

        Ok(())
    }

    /// Register a new client session
    async fn register_session(&self, session_id: &str, user_id: String) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        // Check if this session already exists
        if sessions.contains_key(session_id) {
            // Instead of returning an error, update the existing session if the user_id is different
            if let Some(session) = sessions.get_mut(session_id) {
                if session.user_id != user_id {
                    // Update user ID if it changed
                    session.user_id = user_id;
                    tracing::info!("Updated session {} with new user ID", session_id);
                } else {
                    tracing::info!("Session {} already exists for user {}", session_id, user_id);
                }
            }
            return Ok(());
        }

        // Create a channel for sending messages to the client
        let (sender, _) = mpsc::channel(32);

        // Create the session
        let session = ClientSession {
            user_id,
            document_id: None,
            sender,
        };

        // Add the session
        sessions.insert(session_id.to_string(), session);
        tracing::info!("Registered new session: {}", session_id);

        Ok(())
    }

    /// Get a client session
    async fn get_session(&self, session_id: &str) -> Result<ClientSession> {
        let sessions = self.sessions.read().await;

        sessions.get(session_id)
            .cloned()
            .ok_or_else(|| AppError::ApiError("Session not found".to_string()).into())
    }

    /// Set the active document for a session
    async fn set_active_document(&self, session_id: &str, document_id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.document_id = Some(document_id);
            Ok(())
        } else {
            Err(AppError::ApiError("Session not found".to_string()).into())
        }
    }

    /// Broadcast presence updates for a document
    async fn broadcast_presence(&self, document_id: Uuid) -> Result<()> {
        // Get all sessions for this document
        let sessions = self.sessions.read().await;

        // Get presence information
        let engine = self.crdt_engine.read().await;
        let presences = engine.get_document_presences(&document_id).await?;

        // Create the message
        let message = serde_json::to_string(&ApiMessage::PresenceUpdate {
            document_id,
            presence: presences[0].clone(), // Simplified, would actually send all presences
        })?;

        // Send to all clients editing this document
        for session in sessions.values() {
            if let Some(doc_id) = session.document_id {
                if doc_id == document_id {
                    if let Err(e) = session.sender.send(WarpMessage::text(message.clone())).await {
                        eprintln!("Error sending presence update: {:?}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Broadcast a document update
    pub async fn broadcast_document_update(&self, document_id: Uuid, content: String) -> Result<()> {
        // Get all sessions for this document
        let sessions = self.sessions.read().await;

        // Create the message
        let message = serde_json::to_string(&ApiMessage::DocumentUpdate {
            document_id,
            content,
            version: "latest".to_string(),
        })?;

        // Send to all clients editing this document
        for session in sessions.values() {
            if let Some(doc_id) = session.document_id {
                if doc_id == document_id {
                    if let Err(e) = session.sender.send(WarpMessage::text(message.clone())).await {
                        eprintln!("Error sending document update: {:?}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the sender for a session
    pub async fn get_sender(&self, session_id: &str) -> Result<mpsc::Sender<WarpMessage>> {
        let sessions = self.sessions.read().await;

        sessions.get(session_id)
            .map(|session| session.sender.clone())
            .ok_or_else(|| AppError::ApiError("Session not found".to_string()).into())
    }

    /// Remove a client session
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        // Get the session to check if it's editing a document
        let document_id = if let Some(session) = sessions.get(session_id) {
            session.document_id
        } else {
            None
        };

        // Remove the session
        sessions.remove(session_id);

        // If the session was editing a document, broadcast presence update
        if let Some(doc_id) = document_id {
            // Drop the lock before making nested async calls
            drop(sessions);
            self.broadcast_presence(doc_id).await?;
        }

        tracing::info!("Session removed: {}", session_id);
        Ok(())
    }

    /// Send a heartbeat message to all connected clients
    pub async fn send_heartbeat(&self) -> Result<()> {
        // Get all sessions
        let sessions = self.sessions.read().await;

        // Create a heartbeat message
        let heartbeat = serde_json::to_string(&ApiMessage::Heartbeat {
            timestamp: chrono::Utc::now().to_rfc3339()
        })?;

        tracing::info!("Sending heartbeat to {} connected clients", sessions.len());

        // Send heartbeat to all connected clients
        for (session_id, session) in sessions.iter() {
            if let Err(e) = session.sender.send(WarpMessage::text(heartbeat.clone())).await {
                tracing::warn!("Error sending heartbeat to session {}: {:?}", session_id, e);
            }
        }

        Ok(())
    }
}

/// Handle a new WebSocket connection
async fn handle_websocket_connection(websocket: warp::ws::WebSocket, server: WebSocketServer) {
    // Generate a unique session ID
    let session_id = Uuid::new_v4().to_string();

    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = websocket.split();

    // Create a channel for sending messages to the WebSocket
    let (sender, mut receiver) = mpsc::channel::<WarpMessage>(32);

    // Forward messages from the channel to the WebSocket
    let forward_task = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                eprintln!("Error sending WebSocket message: {:?}", e);
                break;
            }
        }
    });

    // Register the session with a temporary user ID, will be updated on authentication
    if let Err(e) = server.register_session(&session_id, "anonymous".to_string()).await {
        eprintln!("Failed to register session: {:?}", e);
        return;
    }

    // Update the session's sender
    {
        let mut sessions = server.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.sender = sender;
        }
    }

    // Process incoming WebSocket messages
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(message) => {
                if message.is_text() {
                    let text = message.to_str().unwrap_or_default();
                // Try to parse the message as an ApiMessage
                tracing::info!("Received WebSocket message: {}", text);
                match serde_json::from_str::<ApiMessage>(text) {
                        Ok(api_message) => {
                            tracing::info!("Parsed WebSocket message: {:?}", api_message);
                            // Handle the message
                            match server.handle_message(&session_id, api_message).await {
                                Ok(Some(response)) => {
                                    // Send response if there is one
                                    let response_text = serde_json::to_string(&response).unwrap();
                                    tracing::info!("Sending WebSocket response: {}", response_text);
                                    if let Ok(sender) = server.get_sender(&session_id).await {
                                        let _ = sender.send(WarpMessage::text(response_text)).await;
                                    }
                                },
                                Ok(None) => {
                                    // No response needed
                                    tracing::info!("No response needed for this message");
                                },
                                Err(e) => {
                                    // Send error response
                                    let error_message = serde_json::to_string(&ApiMessage::Error {
                                        code: "error".to_string(),
                                        message: format!("Error: {}", e),
                                    }).unwrap();
                                    tracing::error!("WebSocket error: {}", e);

                                    if let Ok(sender) = server.get_sender(&session_id).await {
                                        let _ = sender.send(WarpMessage::text(error_message)).await;
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            // Send parse error response
                            let error_message = serde_json::to_string(&ApiMessage::Error {
                                code: "parse_error".to_string(),
                                message: format!("Failed to parse message: {}", e),
                            }).unwrap();

                            if let Ok(sender) = server.get_sender(&session_id).await {
                                let _ = sender.send(WarpMessage::text(error_message)).await;
                            }
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                break;
            }
        }
    }

    // Remove the session when connection is closed
    let _ = server.remove_session(&session_id).await;

    // Cancel the forward task
    forward_task.abort();
}

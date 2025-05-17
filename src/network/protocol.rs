use futures::prelude::*;
use libp2p::{request_response::{Codec}};
use serde::{Deserialize, Serialize};
use std::io;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

/// Protocol for P2P LaTeX collaboration
#[derive(Debug, Clone)]
pub struct CollabProtocol;

impl AsRef<[u8]> for CollabProtocol {
    fn as_ref(&self) -> &[u8] {
        b"/p2p-latex-collab/1.0.0"
    }
}

/// Network message types for the collaboration protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Request to join the network for a specific document
    JoinRequest {
        document_id: Uuid,
        user_id: String,
        user_name: String,
    },

    /// Response to a join request
    JoinResponse {
        document_id: Uuid,
        success: bool,
        error_message: Option<String>,
        document_content: Option<String>,
    },

    /// Document operation (insert, delete, etc.)
    Operation {
        document_id: Uuid,
        operations: Vec<u8>,  // Encoded operations
    },

    /// Request the full document state
    SyncRequest {
        document_id: Uuid,
        user_id: String,
        version: Option<Vec<u8>>, // Encoded version vector
    },

    /// Response to a sync request
    SyncResponse {
        document_id: Uuid,
        operations: Vec<u8>,      // Encoded operations
        is_full_sync: bool,
    },

    /// User presence information
    Presence {
        document_id: Uuid,
        user_id: String,
        user_name: String,
        cursor_position: Option<usize>,
        is_active: bool,
    },

    /// Document metadata update
    MetadataUpdate {
        document_id: Uuid,
        title: Option<String>,
        repository_url: Option<String>,
    },

    /// User leaving the document
    Leave {
        document_id: Uuid,
        user_id: String,
    },
}

/// Request type for the request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabRequest(pub NetworkMessage);

/// Response type for the request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabResponse(pub NetworkMessage);

/// Codec for encoding/decoding protocol messages
#[derive(Debug, Clone)]
pub struct CollabCodec;

impl Codec for CollabCodec {
    type Protocol = CollabProtocol;
    type Request = CollabRequest;
    type Response = CollabResponse;

    // Use the exact lifetime parameter names expected by the trait
    fn read_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T
    ) -> Pin<Box<dyn Future<Output = io::Result<Self::Request>> + Send + 'async_trait>>
    where
        T: AsyncRead + Unpin + Send + 'async_trait,
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
    {
        Box::pin(async move {
            let mut buffer = Vec::new();
            io.read_to_end(&mut buffer).await?;
            serde_json::from_slice(&buffer)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        })
    }

    // Use the exact lifetime parameter names expected by the trait
    fn read_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T
    ) -> Pin<Box<dyn Future<Output = io::Result<Self::Response>> + Send + 'async_trait>>
    where
        T: AsyncRead + Unpin + Send + 'async_trait,
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
    {
        Box::pin(async move {
            let mut buffer = Vec::new();
            io.read_to_end(&mut buffer).await?;
            serde_json::from_slice(&buffer)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        })
    }

    // Use the exact lifetime parameter names expected by the trait
    fn write_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T,
        req: Self::Request
    ) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'async_trait>>
    where
        T: AsyncWrite + Unpin + Send + 'async_trait,
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
    {
        Box::pin(async move {
            let bytes = serde_json::to_vec(&req)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            io.write_all(&bytes).await
        })
    }

    // Use the exact lifetime parameter names expected by the trait
    fn write_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _: &'life1 Self::Protocol,
        io: &'life2 mut T,
        res: Self::Response
    ) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'async_trait>>
    where
        T: AsyncWrite + Unpin + Send + 'async_trait,
        Self: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
    {
        Box::pin(async move {
            let bytes = serde_json::to_vec(&res)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            io.write_all(&bytes).await
        })
    }
}

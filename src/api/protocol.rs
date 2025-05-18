use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::ops::Range;

/// API protocol messages for communication with clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ApiMessage {
    /// Client authentication
    Authentication {
        user_id: String,
        token: Option<String>,
    },

    /// Document operations
    DocumentOperation {
        /// Operation details
        operation: Operation,
    },

    /// Document state update
    DocumentUpdate {
        /// Document ID
        document_id: Uuid,
        /// Full document content
        content: String,
        /// Document version
        version: String,
    },

    /// Create a new document
    CreateDocument {
        /// Document title
        title: String,
        /// Repository URL (optional)
        repository_url: Option<String>,
    },

    /// Open an existing document
    OpenDocument {
        /// Document ID
        document_id: Uuid,
    },

    /// User presence information
    PresenceUpdate {
        /// Document ID
        document_id: Uuid,
        /// User presence information
        presence: UserPresence,
    },

    /// List available documents
    ListDocuments,

    /// Document list response
    DocumentList {
        /// List of document summaries
        documents: Vec<DocumentSummary>,
    },

    /// Server heartbeat to check connection status
    Heartbeat {
        /// Current server timestamp
        timestamp: String,
    },

    /// Error message
    Error {
        /// Error code
        code: String,
        /// Error message
        message: String,
    },
}

/// Operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Insert text at a position
    Insert {
        /// Document ID
        document_id: Uuid,
        /// Position to insert at
        position: usize,
        /// Text to insert
        content: String,
    },

    /// Delete text in a range
    Delete {
        /// Document ID
        document_id: Uuid,
        /// Range to delete (start..end)
        range: Range<usize>,
    },

    /// Replace text in a range with new content
    Replace {
        /// Document ID
        document_id: Uuid,
        /// Range to replace (start..end)
        range: Range<usize>,
        /// New content
        content: String,
    },
}

/// Document summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    /// Document ID
    pub id: Uuid,
    /// Document title
    pub title: String,
    /// Document owner
    pub owner: String,
    /// Last modified time
    pub updated_at: String,
    /// Number of active collaborators
    pub active_collaborators: usize,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    /// User ID
    pub user_id: String,
    /// User display name
    pub display_name: String,
    /// User's cursor position
    pub cursor_position: Option<usize>,
    /// Selected text range
    pub selection: Option<Range<usize>>,
    /// Whether the user is actively editing
    pub is_active: bool,
    /// Timestamp of the last activity
    pub last_activity: String,
}

/// Response to document operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    /// Operation ID
    pub operation_id: String,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error: Option<String>,
}

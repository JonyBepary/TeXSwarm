use diamond_types::{list::OpLog};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use uuid::Uuid;

use crate::utils::errors::AppError;

/// Represents an operation on a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentOperation {
    /// Insert text at the specified position
    Insert {
        /// Document ID
        document_id: Uuid,
        /// User ID
        user_id: String,
        /// Position to insert at
        position: usize,
        /// Text to insert
        content: String,
    },
    /// Delete text in the specified range
    Delete {
        /// Document ID
        document_id: Uuid,
        /// User ID
        user_id: String,
        /// Range to delete (start..end)
        range: Range<usize>,
    },
    /// Replace text in the specified range with new content
    Replace {
        /// Document ID
        document_id: Uuid,
        /// User ID
        user_id: String,
        /// Range to replace (start..end)
        range: Range<usize>,
        /// New content
        content: String,
    },
}

impl DocumentOperation {
    /// Apply this operation to the given OpLog
    pub fn apply(&self, oplog: &mut OpLog) -> Result<(), AppError> {
        match self {
            DocumentOperation::Insert { user_id, position, content, .. } => {
                let agent_id = oplog.get_or_create_agent_id(&user_id);
                oplog.add_insert(agent_id, *position, &content);
                Ok(())
            }
            DocumentOperation::Delete { user_id, range, .. } => {
                let agent_id = oplog.get_or_create_agent_id(&user_id);
                oplog.add_delete_without_content(agent_id, range.clone());
                Ok(())
            }
            DocumentOperation::Replace { user_id, range, content, .. } => {
                let agent_id = oplog.get_or_create_agent_id(&user_id);

                // First delete the existing content
                oplog.add_delete_without_content(agent_id, range.clone());

                // Then insert the new content
                oplog.add_insert(agent_id, range.start, &content);

                Ok(())
            }
        }
    }
}

/// Interface for encoding and decoding operations for network transmission
#[derive(Debug)]
pub struct OperationEncoder;

impl OperationEncoder {
    pub fn new() -> Self {
        Self {}
    }

    /// Encode an operation for network transmission
    pub fn encode_operation(&self, operation: &DocumentOperation) -> anyhow::Result<Vec<u8>> {
        // For simplicity, we'll use serde_json to encode operations
        Ok(serde_json::to_vec(operation)?)
    }

    /// Decode an operation from network transmission
    pub fn decode_operation(&self, bytes: &[u8]) -> anyhow::Result<DocumentOperation> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

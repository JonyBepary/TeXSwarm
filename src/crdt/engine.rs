use anyhow::Result;
use diamond_types::list::{Branch, OpLog};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::document::Document;
use super::operations::{DocumentOperation, OperationEncoder};
use crate::utils::errors::AppError;
use crate::network::peer::PeerInfo;

/// The CrdtEngine manages all the documents and their corresponding CRDT data structures
#[derive(Debug)]
pub struct CrdtEngine {
    // Map of document IDs to their Document objects
    documents: dashmap::DashMap<Uuid, Arc<RwLock<Document>>>,

    // Map of document IDs to their operation logs
    oplogs: dashmap::DashMap<Uuid, Arc<RwLock<OpLog>>>,

    // Map of document IDs to their branches
    branches: dashmap::DashMap<Uuid, Arc<RwLock<Branch>>>,

    // Operation encoder for serialization/deserialization
    encoder: OperationEncoder,
}

impl CrdtEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            documents: dashmap::DashMap::new(),
            oplogs: dashmap::DashMap::new(),
            branches: dashmap::DashMap::new(),
            encoder: OperationEncoder::new(),
        })
    }

    /// Create a new document
    pub async fn create_document(&self, title: String, owner: String) -> Result<Uuid> {
        let doc_id = Uuid::new_v4();
        let doc = Document::new(doc_id, title, owner);

        // Create new OpLog for the document
        let oplog = OpLog::new();

        // Create a branch for viewing the document
        let branch = Branch::new_at_tip(&oplog);

        // Store the document and its CRDT structures
        self.documents.insert(doc_id, Arc::new(RwLock::new(doc)));
        self.oplogs.insert(doc_id, Arc::new(RwLock::new(oplog)));
        self.branches.insert(doc_id, Arc::new(RwLock::new(branch)));

        Ok(doc_id)
    }

    /// Get a document by ID
    pub async fn get_document(&self, doc_id: &Uuid) -> Result<Arc<RwLock<Document>>> {
        self.documents
            .get(doc_id)
            .map(|item| item.value().clone())
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document not found: {}", doc_id))))
    }

    /// Apply a local operation to a document
    pub async fn apply_local_operation(&self, doc_id: &Uuid, operation: DocumentOperation) -> Result<Vec<u8>> {
        let oplog = self
            .oplogs
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document not found: {}", doc_id))))?;

        let branch = self
            .branches
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document branch not found: {}", doc_id))))?;

        // Apply the operation to the oplog
        {
            let mut oplog_write = oplog.value().write().await;
            match &operation {
                DocumentOperation::Insert { user_id, position, content, .. } => {
                    let agent_id = oplog_write.get_or_create_agent_id(&user_id);
                    oplog_write.add_insert(agent_id, *position, &content);
                },
                DocumentOperation::Delete { user_id, range, .. } => {
                    let agent_id = oplog_write.get_or_create_agent_id(&user_id);
                    oplog_write.add_delete_without_content(agent_id, range.clone());
                },
                DocumentOperation::Replace { user_id, range, content, .. } => {
                    let agent_id = oplog_write.get_or_create_agent_id(&user_id);

                    // Delete then insert
                    oplog_write.add_delete_without_content(agent_id, range.clone());
                    oplog_write.add_insert(agent_id, range.start, &content);
                }
            }
        }

        // Update the branch
        {
            let mut branch_write = branch.value().write().await;
            let oplog_read = oplog.value().read().await;
            branch_write.merge(&oplog_read, oplog_read.local_version_ref());
        }

        // Encode the operation for broadcasting
        let encoded = self.encoder.encode_operation(&operation)?;

        Ok(encoded)
    }

    /// Apply a remote operation to a document (received from the network)
    pub async fn apply_remote_operation(&self, doc_id: &Uuid, encoded_operation: &[u8]) -> Result<()> {
        let oplog = self
            .oplogs
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document not found: {}", doc_id))))?;

        let branch = self
            .branches
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document branch not found: {}", doc_id))))?;

        // Decode the operation
        let operation = self.encoder.decode_operation(encoded_operation)?;

        // Apply the operation to the oplog
        {
            let mut oplog_write = oplog.value().write().await;
            match &operation {
                DocumentOperation::Insert { user_id, position, content, .. } => {
                    let agent_id = oplog_write.get_or_create_agent_id(&user_id);
                    oplog_write.add_insert(agent_id, *position, &content);
                },
                DocumentOperation::Delete { user_id, range, .. } => {
                    let agent_id = oplog_write.get_or_create_agent_id(&user_id);
                    oplog_write.add_delete_without_content(agent_id, range.clone());
                },
                DocumentOperation::Replace { user_id, range, content, .. } => {
                    let agent_id = oplog_write.get_or_create_agent_id(&user_id);

                    // Delete then insert
                    oplog_write.add_delete_without_content(agent_id, range.clone());
                    oplog_write.add_insert(agent_id, range.start, &content);
                }
            }
        }

        // Update the branch
        {
            let mut branch_write = branch.value().write().await;
            let oplog_read = oplog.value().read().await;
            branch_write.merge(&oplog_read, oplog_read.local_version_ref());
        }

        Ok(())
    }

    /// Get the current content of a document
    pub async fn get_document_content(&self, doc_id: &Uuid) -> Result<String> {
        let branch = self
            .branches
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document branch not found: {}", doc_id))))?;

        let branch_read = branch.value().read().await;

        // Extract the content as a string
        let content = branch_read.content().to_string();

        Ok(content)
    }

    /// Update a document's content from external source (e.g., Git)
    pub async fn update_document_content(&self, doc_id: &Uuid, content: String) -> Result<()> {
        let oplog = self
            .oplogs
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document not found: {}", doc_id))))?;

        let branch = self
            .branches
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document branch not found: {}", doc_id))))?;

        // Clear the current content and add the new content
        {
            let mut branch_write = branch.value().write().await;
            let mut oplog_write = oplog.value().write().await;

            // Create a system agent for this operation
            let agent_id = oplog_write.get_or_create_agent_id("system");

            // Delete all content and insert the new content
            let content_len = branch_write.len();
            if content_len > 0 {
                oplog_write.add_delete_without_content(agent_id, 0..content_len);
            }

            oplog_write.add_insert(agent_id, 0, &content);

            // Update the branch
            branch_write.merge(&oplog_write, oplog_write.local_version_ref());
        }

        Ok(())
    }

    /// List all documents
    pub async fn list_documents(&self) -> Result<Vec<Arc<RwLock<Document>>>> {
        let mut docs = Vec::new();
        for item in self.documents.iter() {
            docs.push(item.value().clone());
        }
        Ok(docs)
    }

    /// Import a document from an OpLog binary representation
    pub async fn import_document(&self, title: String, owner: String, encoded_oplog: &[u8]) -> Result<Uuid> {
        let doc_id = Uuid::new_v4();
        let doc = Document::new(doc_id, title, owner);

        // Create a new OpLog and decode the binary data into it
        let mut oplog = OpLog::new();
        oplog.decode_and_add(encoded_oplog)?;

        // Create a branch for viewing the document
        let branch = Branch::new_at_tip(&oplog);

        // Store the document and its CRDT structures
        self.documents.insert(doc_id, Arc::new(RwLock::new(doc)));
        self.oplogs.insert(doc_id, Arc::new(RwLock::new(oplog)));
        self.branches.insert(doc_id, Arc::new(RwLock::new(branch)));

        Ok(doc_id)
    }

    /// Export a document to an OpLog binary representation
    pub async fn export_document(&self, doc_id: &Uuid) -> Result<Vec<u8>> {
        let oplog = self
            .oplogs
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document not found: {}", doc_id))))?;

        let oplog_read = oplog.value().read().await;
        let encoded = oplog_read.encode(diamond_types::list::encoding::EncodeOptions::default());

        Ok(encoded)
    }

    /// Synchronize with another peer by exchanging oplogs
    pub async fn sync_document(&self, doc_id: &Uuid, encoded_oplog: &[u8]) -> Result<Vec<u8>> {
        let oplog = self
            .oplogs
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document not found: {}", doc_id))))?;

        let branch = self
            .branches
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!(AppError::CrdtError(format!("Document branch not found: {}", doc_id))))?;

        // Apply the remote oplog to our local oplog
        {
            let mut oplog_write = oplog.value().write().await;
            oplog_write.decode_and_add(encoded_oplog)?;
        }

        // Update the branch
        {
            let mut branch_write = branch.value().write().await;
            let oplog_read = oplog.value().read().await;
            branch_write.merge(&oplog_read, oplog_read.local_version_ref());
        }

        // Export our oplog to send back
        let oplog_read = oplog.value().read().await;
        let encoded = oplog_read.encode(diamond_types::list::encoding::EncodeOptions::default());

        Ok(encoded)
    }

    /// Get user presence information for a document (dummy implementation)
    pub async fn get_document_presences(&self, _doc_id: &Uuid) -> Result<Vec<crate::api::protocol::UserPresence>> {
        // Dummy implementation returning a single user presence
        Ok(vec![crate::api::protocol::UserPresence {
            user_id: "user1".to_string(),
            display_name: "User 1".to_string(),
            cursor_position: Some(0),
            selection: None,
            is_active: true,
            last_activity: chrono::Utc::now().to_rfc3339(),
        }])
    }

    /// Update user presence in a document (dummy implementation)
    pub async fn update_user_presence(&self, doc_id: Uuid, presence: crate::api::protocol::UserPresence) -> Result<()> {
        // Just log that we received the presence update
        tracing::info!("User {} presence updated in document {}", presence.user_id, doc_id);
        Ok(())
    }

    /// Get the peers for a document
    pub async fn get_document_peers(&self, _doc_id: &Uuid) -> Result<Vec<PeerInfo>> {
        // This would normally be implemented to get peers from the document's subscribers
        // For now, we'll return an empty list
        Ok(Vec::new())
    }

    /// Get all document IDs
    pub async fn get_all_documents(&self) -> Result<Vec<Uuid>> {
        let mut doc_ids = Vec::new();
        for item in self.documents.iter() {
            doc_ids.push(*item.key());
        }
        Ok(doc_ids)
    }
}

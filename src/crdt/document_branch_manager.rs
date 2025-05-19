use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::crdt::engine::CrdtEngine;
use crate::utils::errors::AppError;

/// Manages document branches across instances to prevent "Document branch not found" errors
pub struct DocumentBranchManager {
    crdt_engine: Arc<RwLock<CrdtEngine>>,
    /// Tracks documents that have been requested but don't exist yet
    pending_documents: RwLock<HashMap<Uuid, String>>,
}

impl DocumentBranchManager {
    pub fn new(crdt_engine: Arc<RwLock<CrdtEngine>>) -> Self {
        Self {
            crdt_engine,
            pending_documents: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a document exists and create it if it doesn't
    pub async fn ensure_document_exists(&self, document_id: &Uuid, title: &str) -> Result<bool> {
        let engine = self.crdt_engine.read().await;

        // Check if document exists
        match engine.get_document(document_id).await {
            Ok(_) => {
                // Document exists
                return Ok(true);
            }
            Err(e) => {
                if let Some(app_error) = e.downcast_ref::<AppError>() {
                    match app_error {
                        AppError::DocumentNotFound(_) | AppError::CrdtError(_) => {
                            // Document doesn't exist, we'll create it
                            drop(engine); // Drop the read lock before acquiring a write lock

                            // Create the document
                            let engine = self.crdt_engine.write().await;
                            let _ = engine.create_document(title.to_string(), "system".to_string()).await?;

                            // Register as created
                            let mut pending = self.pending_documents.write().await;
                            pending.remove(document_id);

                            return Ok(false);
                        }
                        _ => return Err(e),
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }

    /// Register a pending document request
    pub async fn register_pending_document(&self, document_id: &Uuid, title: &str) -> Result<()> {
        let mut pending = self.pending_documents.write().await;
        pending.insert(*document_id, title.to_string());
        Ok(())
    }

    /// Create any registered pending documents
    pub async fn create_pending_documents(&self) -> Result<usize> {
        let mut pending = self.pending_documents.write().await;
        let pending_docs: Vec<(Uuid, String)> = pending.drain().collect();

        let mut created_count = 0;

        for (id, title) in pending_docs {
            let engine = self.crdt_engine.write().await;
            match engine.create_document(title, "system".to_string()).await {
                Ok(_) => {
                    created_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to create pending document {}: {:?}", id, e);
                }
            }
        }

        Ok(created_count)
    }

    /// Process an error to check if it's a "Document branch not found" error
    pub async fn process_document_error(&self, error: &anyhow::Error, document_id: &Uuid, title: &str) -> Result<bool> {
        let error_message = error.to_string();

        if error_message.contains("Document branch not found") {
            // Register the document for creation
            self.register_pending_document(document_id, title).await?;

            // Try to create it immediately
            self.ensure_document_exists(document_id, title).await?;

            return Ok(true);
        }

        Ok(false)
    }
}

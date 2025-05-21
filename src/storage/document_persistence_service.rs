use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, Instant};
use uuid::Uuid;

use crate::crdt::engine::CrdtEngine;
use crate::crdt::document_branch_manager::DocumentBranchManager;
use crate::git::manager::GitManager;

/// Service responsible for persisting documents to both local storage and remote Git repositories
pub struct DocumentPersistenceService {
    crdt_engine: Arc<RwLock<CrdtEngine>>,
    git_manager: Arc<RwLock<GitManager>>,
    branch_manager: Arc<DocumentBranchManager>,
    /// How often to auto-save documents (in seconds)
    auto_save_interval: u64,
    /// Last time documents were saved
    last_save: RwLock<std::collections::HashMap<Uuid, Instant>>,
}

impl DocumentPersistenceService {
    pub fn new(
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        git_manager: Arc<RwLock<GitManager>>,
        auto_save_interval: u64,
    ) -> Self {
        let branch_manager = Arc::new(DocumentBranchManager::new(crdt_engine.clone()));

        Self {
            crdt_engine,
            git_manager,
            branch_manager,
            auto_save_interval,
            last_save: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Start the auto-save service
    pub async fn start(self: Arc<Self>) {
        // Run auto-save every 30 seconds
        let mut tick_interval = interval(Duration::from_secs(30));

        loop {
            tick_interval.tick().await;
            if let Err(e) = self.auto_save_all_documents().await {
                eprintln!("Error during auto-save: {:?}", e);
            }
        }
    }

    /// Get the document branch manager
    pub fn branch_manager(&self) -> Arc<DocumentBranchManager> {
        self.branch_manager.clone()
    }

    /// Manually save a specific document
    pub async fn save_document(&self, document_id: &Uuid) -> Result<()> {
        // Get the document content
        let content = {
            let engine = self.crdt_engine.read().await;
            engine.get_document_content(document_id).await?
        };

        // Save locally first (this happens automatically in the CRDT engine)

        // Then attempt to save to Git if available
        let mut git = self.git_manager.write().await;
        match git.sync_document_blocking(document_id, content) {
            Ok(_) => {
                // Update the last save time
                let mut last_save = self.last_save.write().await;
                last_save.insert(*document_id, Instant::now());
                Ok(())
            }
            Err(e) => {
                // Check if the error is due to missing Git repository - this is fine for local-only docs
                if e.to_string().contains("No repository found") {
                    // Local-only document, just update the save time
                    let mut last_save = self.last_save.write().await;
                    last_save.insert(*document_id, Instant::now());
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Auto-save all documents that need saving
    async fn auto_save_all_documents(&self) -> Result<()> {
        // Get all documents
        let documents = {
            let engine = self.crdt_engine.read().await;
            engine.get_all_documents().await?
        };

        let mut save_count = 0;

        // Check each document
        for doc_id in documents {
            // Check if this document needs saving
            let needs_save = {
                let last_save = self.last_save.read().await;
                match last_save.get(&doc_id) {
                    Some(time) => time.elapsed().as_secs() >= self.auto_save_interval,
                    None => true, // Never saved before
                }
            };

            if needs_save {
                if let Err(e) = self.save_document(&doc_id).await {
                    eprintln!("Error saving document {}: {:?}", doc_id, e);
                } else {
                    save_count += 1;
                }
            }
        }

        if save_count > 0 {
            println!("Auto-saved {} documents", save_count);
        }

        Ok(())
    }

    /// Create a new document and ensure it persists
    pub async fn create_document(&self, title: &str, owner: &str) -> Result<Uuid> {
        // Create the document in the CRDT engine
        let doc_id = {
            let engine = self.crdt_engine.write().await;
            engine.create_document(title.to_string(), owner.to_string()).await?
        };

        // Save it immediately
        self.save_document(&doc_id).await?;

        Ok(doc_id)
    }

    /// Integrate with Git - create a new Git repository for a document
    pub async fn create_document_with_git(&self, title: &str, owner: &str, repo_name: &str) -> Result<(Uuid, String)> {
        // Create document first
        let doc_id = self.create_document(title, owner).await?;

        // Create Git repository
        let repo_url = {
            let mut git = self.git_manager.write().await;
            git.create_repository(&doc_id, repo_name).await?
        };

        // Save document to the repository
        self.save_document(&doc_id).await?;

        Ok((doc_id, repo_url))
    }

    /// Import a document from a Git repository
    pub async fn import_document_from_git(&self, title: &str, owner: &str, repo_url: &str) -> Result<Uuid> {
        // First create an empty document
        let doc_id = self.create_document(title, owner).await?;

        // Clone the repository
        {
            let mut git = self.git_manager.write().await;
            git.clone_repository(&doc_id, repo_url).await?;
        }

        // Pull content from the repository
        {
            let mut git = self.git_manager.write().await;
            git.pull_changes(&doc_id).await?;
        }

        Ok(doc_id)
    }

    /// Handle a "Document branch not found" error by ensuring the document exists
    pub async fn handle_branch_not_found_error(&self, document_id: &Uuid, title: &str) -> Result<bool> {
        self.branch_manager.ensure_document_exists(document_id, title).await
    }
}

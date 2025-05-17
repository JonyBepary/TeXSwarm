use anyhow::Result;
use git2::Repository;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time;
use uuid::Uuid;

use super::repository::RepositoryManager;
use crate::crdt::engine::CrdtEngine;
use crate::utils::errors::AppError;

/// Manages synchronization between the CRDT and Git repository
#[derive(Clone)]
pub struct GitSync {
    /// Repository manager
    pub repo_manager: RepositoryManager,
    /// CRDT engine
    crdt_engine: Arc<RwLock<CrdtEngine>>,
    /// Synchronization interval
    sync_interval: Duration,
    /// Last synchronization time for each document
    last_sync: Arc<RwLock<std::collections::HashMap<Uuid, Instant>>>,
}

impl GitSync {
    /// Create a new git sync manager
    pub fn new(
        repo_manager: RepositoryManager,
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        sync_interval: Duration,
    ) -> Self {
        Self {
            repo_manager,
            crdt_engine,
            sync_interval,
            last_sync: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Start the periodic synchronization task
    pub async fn start_sync_task(self) {
        let mut interval = time::interval(Duration::from_secs(30)); // Check every 30 seconds

        loop {
            interval.tick().await;
            if let Err(e) = self.check_for_sync().await {
                eprintln!("Error during git sync: {:?}", e);
            }
        }
    }

    /// Check if any documents need to be synchronized
    async fn check_for_sync(&self) -> Result<()> {
        let engine = self.crdt_engine.read().await;
        let documents = engine.get_all_documents().await?;
        let last_sync_read = self.last_sync.read().await;

        for doc_id in documents {
            // Check if document needs synchronization
            let needs_sync = match last_sync_read.get(&doc_id) {
                Some(last) => last.elapsed() >= self.sync_interval,
                None => true,
            };

            if needs_sync {
                if let Err(e) = self.sync_document(&doc_id).await {
                    eprintln!("Error syncing document {}: {:?}", doc_id, e);
                }
            }
        }

        Ok(())
    }

    /// Synchronize a specific document
    pub async fn sync_document(&self, document_id: &Uuid) -> Result<()> {
        // Get the document data
        let engine = self.crdt_engine.read().await;
        let document = engine.get_document(document_id).await?;
        let doc = document.read().await;

        // Check if the document has a repository URL
        let repo_url = doc.repository_url.clone().ok_or_else(||
            AppError::GitError(format!("Document {} has no repository URL", document_id)))?;

        // Open or clone the repository
        let repo = self.repo_manager.clone_or_open(&repo_url, document_id)?;

        // Get the document content
        let content = engine.get_document_content(document_id).await?;

        // Save the document to the repository
        let filename = format!("{}.tex", doc.title.replace(" ", "_"));
        self.repo_manager.save_document(
            &repo,
            &content,
            &filename,
            &format!("Update document {}", doc.title),
        )?;

        // Update the bootstrap file
        self.update_bootstrap_file(&repo, document_id).await?;

        // Update the last sync time
        {
            let mut last_sync = self.last_sync.write().await;
            last_sync.insert(*document_id, Instant::now());
        }

        Ok(())
    }

    /// Synchronize a document's content to the repository
    pub async fn sync_document_to_repo(&self, repo: &Repository, content: String, message: &str) -> Result<()> {
        // Get the main document filename
        let filename = "document.tex"; // Using a standard filename

        // Save the document to the repository
        self.repo_manager.save_document(
            repo,
            &content,
            filename,
            message,
        )?;

        Ok(())
    }

    /// Synchronize a document's content to the repository - blocking version
    pub fn sync_document_to_repo_blocking(&self, repo: &Repository, content: String, message: &str) -> Result<()> {
        // Get the main document filename
        let filename = "document.tex"; // Using a standard filename

        // Save the document to the repository
        self.repo_manager.save_document(
            repo,
            &content,
            filename,
            message,
        )?;

        // Push the changes to the remote
        self.repo_manager.push(repo)?;

        Ok(())
    }

    /// Push changes to the remote repository
    pub async fn push_changes(&self, repo: &Repository) -> Result<()> {
        // Push the changes to the remote
        self.repo_manager.push(repo)?;

        Ok(())
    }

    /// Pull changes from the remote repository
    pub async fn pull_changes(&self, repo: &Repository) -> Result<()> {
        // Pull the changes from the remote
        self.repo_manager.pull(repo)?;

        Ok(())
    }

    /// Update the bootstrap file with current peers
    async fn update_bootstrap_file(&self, repo: &Repository, document_id: &Uuid) -> Result<()> {
        // Get the list of connected peers
        let engine = self.crdt_engine.read().await;
        let peers = engine.get_document_peers(document_id).await?;

        // Format the peer addresses
        let peer_addresses: Vec<String> = peers.iter()
            .map(|p| format!("{},{}", p.peer_id, p.addresses.join(";")))
            .collect();

        // Update the bootstrap file
        self.repo_manager.create_bootstrap_file(
            repo,
            &peer_addresses,
            "bootstrap.txt",
        )?;

        Ok(())
    }

    /// Load a document from the repository
    pub async fn load_document_from_repository(
        &self,
        repo_url: &str,
        document_id: &Uuid,
        filename: &str,
    ) -> Result<String> {
        // Open or clone the repository
        let repo = self.repo_manager.clone_or_open(repo_url, document_id)?;

        // Get the repository path
        let repo_path = repo.path().parent().ok_or_else(||
            AppError::GitError("Could not get repository path".to_string()))?;

        let file_path = repo_path.join(filename);

        // Read the file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| AppError::IoError(e))?;

        Ok(content)
    }

    /// Read bootstrap peers from a repository
    pub async fn get_bootstrap_peers(&self, repo_url: &str, document_id: &Uuid) -> Result<Vec<(String, Vec<String>)>> {
        // Open or clone the repository
        let repo = self.repo_manager.clone_or_open(repo_url, document_id)?;

        // Read the bootstrap file
        let peers = self.repo_manager.read_bootstrap_file(&repo, "bootstrap.txt")?;

        // Parse the peer addresses
        let mut result = Vec::new();
        for peer in peers {
            let parts: Vec<&str> = peer.split(',').collect();
            if parts.len() == 2 {
                let peer_id = parts[0].to_string();
                let addresses: Vec<String> = parts[1].split(';')
                    .map(|addr| addr.to_string())
                    .collect();

                result.push((peer_id, addresses));
            }
        }

        Ok(result)
    }

    /// Get document content from the repository
    pub async fn get_document_from_repo(&self, repo: &Repository) -> Result<String> {
        // Get the repository path
        let repo_path = repo.path().parent().ok_or_else(||
            AppError::GitError("Could not get repository path".to_string()))?;

        // Try to find the main document file
        let filename = "document.tex"; // Using the standard filename
        let file_path = repo_path.join(filename);

        // Check if the file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!(AppError::GitError(format!("Document file not found: {}", filename))));
        }

        // Read the file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| AppError::IoError(e))?;

        Ok(content)
    }
}

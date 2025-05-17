use anyhow::Result;
use git2::Repository;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::crdt::engine::CrdtEngine;
use crate::git::repository::RepositoryManager;
use crate::git::sync::GitSync;
use crate::utils::config::Config;
use crate::utils::errors::AppError;

/// The GitManager handles Git repository operations and document synchronization
#[derive(Clone)]
pub struct GitManager {
    config: Config,
    repositories: HashMap<Uuid, RepositoryManager>,
    crdt_engine: Arc<RwLock<CrdtEngine>>,
    git_synchronizer: GitSync,
}

impl GitManager {
    pub fn new(config: &Config, crdt_engine: Arc<RwLock<CrdtEngine>>) -> Result<Self> {
        // Ensure the repositories directory exists
        std::fs::create_dir_all(&config.git.repositories_path)?;

        // Create the Git synchronizer
        let repo_manager = RepositoryManager::new(config.git.clone());

        // Use Arc<RwLock<CrdtEngine>> for the synchronizer
        let git_synchronizer = GitSync::new(
            repo_manager.clone(),
            crdt_engine.clone(),
            std::time::Duration::from_secs(config.git.sync_interval_secs)
        );

        Ok(Self {
            config: config.clone(),
            repositories: HashMap::new(),
            crdt_engine,
            git_synchronizer,
        })
    }

    /// Create a new repository for a document
    pub async fn create_repository(&mut self, doc_id: &Uuid, name: &str) -> Result<String> {
        // Get the document to verify it exists
        let engine = self.crdt_engine.read().await;
        let document = engine.get_document(doc_id).await?;
        let doc = document.read().await;

        // This is a placeholder implementation since we're replacing GitRepository
        // In a real implementation, we'd initialize a Git repository here
        // and get its remote URL
        let repo_url = format!("https://example.com/repos/{}.git", name);

        // Store the repository
        self.repositories.insert(*doc_id, self.git_synchronizer.repo_manager.clone());

        // Update the document with the repository URL
        drop(doc); // Drop the read lock before acquiring a write lock
        let mut doc = document.write().await;
        doc.set_repository_url(repo_url.clone());

        return Ok(repo_url);
    }

    /// Clone an existing repository for a document
    pub async fn clone_repository(&mut self, doc_id: &Uuid, url: &str) -> Result<()> {
        // Get the document to verify it exists
        let engine = self.crdt_engine.read().await;
        let document = engine.get_document(doc_id).await?;
        let doc = document.read().await;

        // Create a repository path (not used directly, but keeping for documentation)
        let _repo_path = self.get_repository_path(doc_id);

        // Clone the repository
        let _repo = self.git_synchronizer.repo_manager.clone_or_open(url, doc_id)?;

        // Store the repository
        self.repositories.insert(*doc_id, self.git_synchronizer.repo_manager.clone());

        // Update the document with the repository URL
        drop(doc); // Drop the read lock before acquiring a write lock
        let mut doc = document.write().await;
        doc.set_repository_url(url.to_string());

        Ok(())
    }

    /// Synchronize a document with its Git repository
    pub async fn sync_document(&mut self, doc_id: &Uuid) -> Result<()> {
        // Get the document
        let repo_url_opt;
        let doc_title;
        let content;

        {
            let engine = self.crdt_engine.read().await;
            let document = engine.get_document(doc_id).await?;
            let doc = document.read().await;
            repo_url_opt = doc.repository_url.clone();
            doc_title = doc.title.clone();

            // Get the content from the CRDT document before dropping locks
            content = engine.get_document_content(doc_id).await?;
        } // All locks are dropped here

        // Get the repository for this document
        let repo = match self.repositories.get(doc_id) {
            Some(repo) => repo.clone(),
            None => {
                // If the repository doesn't exist locally but the document has a URL,
                // try to clone it
                if let Some(url) = repo_url_opt {
                    self.clone_repository(doc_id, &url).await?;
                    self.repositories.get(doc_id).unwrap().clone()
                } else {
                    return Err(anyhow::anyhow!(AppError::GitError(format!("No repository found for document: {}", doc_id))));
                }
            }
        };

        // Use the synchronizer to update the repository
        let repo_path = repo.path.clone();
        let repo_obj = Repository::open(&repo_path)
            .map_err(|e| AppError::GitError(format!("Failed to open repository at {}: {}", repo_path.display(), e)))?;
        self.git_synchronizer.sync_document_to_repo(&repo_obj, content, &format!("Update document {}", doc_title)).await?;

        // Push changes to remote if available
        match repo_obj.find_remote("origin") {
            Ok(_) => {
                self.git_synchronizer.push_changes(&repo_obj).await?;
            },
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                // No remote, continue without pushing
            },
            Err(e) => {
                return Err(anyhow::anyhow!(AppError::GitError(format!("Failed to check remote: {}", e))));
            }
        }

        Ok(())
    }

    /// Synchronize a document with its Git repository, using a blocking approach
    /// that avoids using async code with git2 (which isn't Send/Sync)
    pub fn sync_document_blocking(&mut self, doc_id: &Uuid, content: String) -> Result<()> {
        // Get the repository URL from the document database or configuration
        let repo_url = match self.get_repository_url(doc_id) {
            Some(url) => url,
            None => return Err(anyhow::anyhow!(AppError::RepositoryNotFound(*doc_id))),
        };

        // Get the document title
        let doc_title = match self.get_document_title(doc_id) {
            Some(title) => title,
            None => return Err(anyhow::anyhow!(AppError::DocumentNotFound(*doc_id))),
        };

        // Construct a repository manager
        let repo_manager = self.git_synchronizer.repo_manager.clone();

        // This is a blocking call that creates/opens a repository
        let repo = repo_manager.clone_or_open(&repo_url, doc_id)?;

        // Update the repository
        self.git_synchronizer.sync_document_to_repo_blocking(
            &repo,
            content,
            &format!("Update {}", doc_title)
        )?;

        Ok(())
    }

    // Helper methods to get document information without async
    fn get_repository_url(&self, doc_id: &Uuid) -> Option<String> {
        // Since we don't have direct access to documents, we need to use the repositories map
        self.repositories.get(doc_id).and_then(|_| {
            // Check if we have a repository for this document
            // If we do, we need to get the URL from somewhere else
            // For now, just return a placeholder value
            Some("https://github.com/example/placeholder.git".to_string())
        })
    }

    fn get_document_title(&self, doc_id: &Uuid) -> Option<String> {
        // Since we don't have direct access to document titles, return a placeholder
        Some(format!("Document-{}", doc_id))
    }

    /// Pull changes from a remote repository and update the document
    pub async fn pull_changes(&mut self, doc_id: &Uuid) -> Result<()> {
        // Get the document URL
        let repo_url_opt;

        {
            let engine = self.crdt_engine.read().await;
            let document = engine.get_document(doc_id).await?;
            let doc = document.read().await;
            repo_url_opt = doc.repository_url.clone();
        } // All locks are dropped here

        // Get the repository for this document
        let repo = match self.repositories.get(doc_id) {
            Some(repo) => repo.clone(),
            None => {
                // If the repository doesn't exist locally but the document has a URL,
                // try to clone it
                if let Some(url) = repo_url_opt {
                    self.clone_repository(doc_id, &url).await?;
                    self.repositories.get(doc_id).unwrap().clone()
                } else {
                    return Err(anyhow::anyhow!(AppError::GitError(format!("No repository found for document: {}", doc_id))));
                }
            }
        };

        // Pull changes from the remote
        let repo_path = repo.path.clone();
        let repo_obj = Repository::open(&repo_path)
            .map_err(|e| AppError::GitError(format!("Failed to open repository at {}: {}", repo_path.display(), e)))?;
                match repo_obj.find_remote("origin") {
            Ok(_) => {
                self.git_synchronizer.pull_changes(&repo_obj).await?;

                // Get the updated content from the repository
                let content = self.git_synchronizer.get_document_from_repo(&repo_obj).await?;

                // Update the CRDT document with the new content
                let engine = self.crdt_engine.write().await;
                engine.update_document_content(doc_id, content).await?;
            },
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                // No remote, continue without pulling
            },
            Err(e) => {
                return Err(anyhow::anyhow!(AppError::GitError(format!("Failed to check remote: {}", e))));
            }
        }

        Ok(())
    }

    /// Get the path for storing a document's Git repository
    fn get_repository_path(&self, doc_id: &Uuid) -> PathBuf {
        self.config.git.repositories_path.join(doc_id.to_string())
    }
}

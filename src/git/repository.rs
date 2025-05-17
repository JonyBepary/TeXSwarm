use anyhow::Result;
use git2::{Repository, Signature, PushOptions, RemoteCallbacks};
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;

use crate::utils::config::GitConfig;
use crate::utils::errors::AppError;

/// Repository manager for handling git operations
#[derive(Clone)]
pub struct RepositoryManager {
    /// Configuration for git
    config: GitConfig,
    /// Base path for repositories
    pub path: PathBuf,
}

impl RepositoryManager {
    /// Create a new repository manager
    pub fn new(config: GitConfig) -> Self {
        Self {
            config: config.clone(),
            path: config.repositories_path.clone()
        }
    }

    /// Clone a repository if it doesn't exist
    pub fn clone_or_open(&self, repo_url: &str, document_id: &Uuid) -> Result<Repository> {
        let repo_path = self.get_repo_path(document_id);

        // Check if repository directory exists
        if repo_path.exists() {
            // Open existing repository
            let repo = Repository::open(&repo_path)
                .map_err(|e| AppError::GitError(format!("Failed to open repository: {}", e)))?;

            // Pull latest changes
            self.pull(&repo)?;

            Ok(repo)
        } else {
            // Create parent directory if it doesn't exist
            if let Some(parent) = repo_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| AppError::IoError(e))?;
            }

            // Clone the repository
            let repo = Repository::clone(repo_url, &repo_path)
                .map_err(|e| AppError::GitError(format!("Failed to clone repository: {}", e)))?;

            Ok(repo)
        }
    }

    /// Get the path to a repository
    fn get_repo_path(&self, document_id: &Uuid) -> PathBuf {
        self.config.repositories_path.join(document_id.to_string())
    }

    /// Pull the latest changes from the remote repository
    pub fn pull(&self, repo: &Repository) -> Result<()> {
        // Get the default remote
        let mut remote = repo.find_remote("origin")
            .map_err(|e| AppError::GitError(format!("Failed to find remote: {}", e)))?;

        // Fetch from the remote
        remote.fetch(&["main", "master"], None, None)
            .map_err(|e| AppError::GitError(format!("Failed to fetch from remote: {}", e)))?;

        // Get the current branch
        let head = repo.head()
            .map_err(|e| AppError::GitError(format!("Failed to get HEAD: {}", e)))?;

        let branch_name = head.shorthand().unwrap_or("master");

        // Get the remote reference
        let remote_ref = format!("refs/remotes/origin/{}", branch_name);
        let remote_branch = repo.find_reference(&remote_ref)
            .map_err(|e| AppError::GitError(format!("Failed to find remote reference: {}", e)))?;

        let remote_commit = remote_branch.peel_to_commit()
            .map_err(|e| AppError::GitError(format!("Failed to get remote commit: {}", e)))?;

        // Create a merge from the remote commit
        let mut index = repo.index()
            .map_err(|e| AppError::GitError(format!("Failed to get index: {}", e)))?;

        let head_commit = head.peel_to_commit()
            .map_err(|e| AppError::GitError(format!("Failed to get HEAD commit: {}", e)))?;

        // Only merge if there are changes
        if head_commit.id() != remote_commit.id() {
            let mut merge_result = repo.merge_commits(&head_commit, &remote_commit, None)
                .map_err(|e| AppError::GitError(format!("Failed to merge commits: {}", e)))?;

            if merge_result.has_conflicts() {
                return Err(AppError::GitError("Merge conflicts detected".to_string()).into());
            }

            // Update the index
            index.write()
                .map_err(|e| AppError::GitError(format!("Failed to write index: {}", e)))?;

            // Create the commit
            let tree = repo.find_tree(merge_result.write_tree_to(repo)
                .map_err(|e| AppError::GitError(format!("Failed to write tree: {}", e)))?)
                .map_err(|e| AppError::GitError(format!("Failed to find tree: {}", e)))?;

            let signature = self.create_signature()?;

            let message = format!("Merge remote-tracking branch 'origin/{}'", branch_name);
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &[&head_commit, &remote_commit],
            )
            .map_err(|e| AppError::GitError(format!("Failed to create merge commit: {}", e)))?;
        }

        Ok(())
    }

    /// Save a document to the repository
    pub fn save_document(&self, repo: &Repository, content: &str, filename: &str, message: &str) -> Result<()> {
        // Get the repository path
        let repo_path = repo.path().parent().ok_or_else(|| AppError::GitError("Could not get repository path".to_string()))?;
        let file_path = repo_path.join(filename);

        // Write the content to the file
        fs::write(&file_path, content)
            .map_err(|e| AppError::IoError(e))?;

        // Create a signature
        let signature = self.create_signature()?;

        // Add the file to the index
        let mut index = repo.index()
            .map_err(|e| AppError::GitError(format!("Failed to get index: {}", e)))?;

        index.add_path(Path::new(filename))
            .map_err(|e| AppError::GitError(format!("Failed to add file to index: {}", e)))?;

        index.write()
            .map_err(|e| AppError::GitError(format!("Failed to write index: {}", e)))?;

        // Create a tree from the index
        let tree_id = index.write_tree()
            .map_err(|e| AppError::GitError(format!("Failed to write tree: {}", e)))?;

        let tree = repo.find_tree(tree_id)
            .map_err(|e| AppError::GitError(format!("Failed to find tree: {}", e)))?;

        // Get the parent commit
        let head = repo.head()
            .map_err(|e| AppError::GitError(format!("Failed to get HEAD: {}", e)))?;

        let parent_commit = head.peel_to_commit()
            .map_err(|e| AppError::GitError(format!("Failed to get HEAD commit: {}", e)))?;

        // Create the commit
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )
        .map_err(|e| AppError::GitError(format!("Failed to create commit: {}", e)))?;

        // Push the changes
        self.push(repo)?;

        Ok(())
    }

    /// Push changes to the remote repository
    pub fn push(&self, repo: &Repository) -> Result<()> {
        // Get the default remote
        let mut remote = repo.find_remote("origin")
            .map_err(|e| AppError::GitError(format!("Failed to find remote: {}", e)))?;

        // Set up callbacks
        let mut callbacks = RemoteCallbacks::new();

        // Add authentication if GitHub token is provided
        if let Some(token) = &self.config.github_token {
            callbacks.credentials(move |_url, _username, _allowed| {
                git2::Cred::userpass_plaintext("x-access-token", token)
            });
        }

        // Create push options
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Get the current branch
        let head = repo.head()
            .map_err(|e| AppError::GitError(format!("Failed to get HEAD: {}", e)))?;

        let branch_name = head.shorthand().unwrap_or("master");
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // Push to the remote
        remote.push(&[&refspec], Some(&mut push_options))
            .map_err(|e| AppError::GitError(format!("Failed to push to remote: {}", e)))?;

        Ok(())
    }

    /// Create a signature for commits
    fn create_signature(&self) -> Result<Signature> {
        let name = self.config.github_username.clone()
            .unwrap_or_else(|| "P2P LaTeX Collaborator".to_string());
        let email = self.config.github_email.clone()
            .unwrap_or_else(|| "collab@example.com".to_string());

        Signature::now(&name, &email)
            .map_err(|e| AppError::GitError(format!("Failed to create signature: {}", e)).into())
    }

    /// Create a bootstrap file in the repository
    pub fn create_bootstrap_file(&self, repo: &Repository, peers: &[String], filename: &str) -> Result<()> {
        let content = peers.join("\n");
        self.save_document(repo, &content, filename, "Update bootstrap peers")
    }

    /// Read the bootstrap file from the repository
    pub fn read_bootstrap_file(&self, repo: &Repository, filename: &str) -> Result<Vec<String>> {
        // Get the repository path
        let repo_path = repo.path().parent().ok_or_else(|| AppError::GitError("Could not get repository path".to_string()))?;
        let file_path = repo_path.join(filename);

        // Check if the file exists
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        // Read the file content
        let content = fs::read_to_string(file_path)
            .map_err(|e| AppError::IoError(e))?;

        // Split the content by lines and filter out empty lines
        let peers: Vec<String> = content.lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(peers)
    }
}

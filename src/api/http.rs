use anyhow::Result;
// Remove unused import: futures::future
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};

use crate::crdt::engine::CrdtEngine;
use crate::crdt::operations::DocumentOperation;
use crate::git::manager::GitManager;
use crate::network::engine::NetworkEngine;
use crate::utils::config::Config;
use crate::utils::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentResponse {
    pub document_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub id: Uuid,
    pub title: String,
    pub owner: String,
    pub collaborators: Vec<String>,
    pub repository_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertOperationRequest {
    pub user_id: String,
    pub position: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOperationRequest {
    pub user_id: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    pub status: String,
    pub server_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// The HTTP API server
pub struct HttpApi {
    crdt_engine: Arc<RwLock<CrdtEngine>>,
    network_engine: Arc<RwLock<NetworkEngine>>,
    git_manager: Arc<RwLock<GitManager>>,
}

impl HttpApi {
    pub fn new(
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        network_engine: Arc<RwLock<NetworkEngine>>,
        git_manager: Arc<RwLock<GitManager>>,
    ) -> Self {
        Self {
            crdt_engine,
            network_engine,
            git_manager,
        }
    }

    pub async fn start(&self, config: &Config) -> Result<()> {
        // Clone the dependencies to avoid borrowing `self`
        let crdt_engine = self.crdt_engine.clone();
        let network_engine = self.network_engine.clone();
        let git_manager = self.git_manager.clone();

        let addr = format!("{}:{}", config.server.api_host, config.server.api_port)
            .parse::<std::net::SocketAddr>()
            .map_err(|e| anyhow::anyhow!("Failed to parse API address: {}", e))?;

        tracing::info!("HTTP API binding to socket address: {}", addr);

        // Move all dependencies into the tokio::spawn
        tokio::spawn(async move {
            // Create routes directly inside the async block to avoid lifetime issues
            let routes = Self::create_routes(crdt_engine, network_engine, git_manager);
            warp::serve(routes).run(addr).await;
        });

        Ok(())
    }

    // Static method to create routes without borrowing self
    fn create_routes(
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        network_engine: Arc<RwLock<NetworkEngine>>,
        git_manager: Arc<RwLock<GitManager>>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let ping = warp::path("api")
            .and(warp::path("ping"))
            .and(warp::get())
            .map(|| {
                warp::reply::json(&PingResponse {
                    status: "ok".to_string(),
                    server_version: env!("CARGO_PKG_VERSION").to_string(),
                })
            });

        let create_document = warp::path("documents")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_crdt_engine(crdt_engine.clone()))
            .and_then(Self::handle_create_document);

        let list_documents = warp::path("documents")
            .and(warp::get())
            .and(with_crdt_engine(crdt_engine.clone()))
            .and_then(Self::handle_list_documents);

        let get_document = warp::path!("documents" / String)
            .and(warp::get())
            .and(with_crdt_engine(crdt_engine.clone()))
            .and_then(Self::handle_get_document);

        let insert_operation = warp::path!("documents" / String / "insert")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_crdt_engine(crdt_engine.clone()))
            .and(with_network_engine(network_engine.clone()))
            .and_then(Self::handle_insert_operation);

        let delete_operation = warp::path!("documents" / String / "delete")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_crdt_engine(crdt_engine.clone()))
            .and(with_network_engine(network_engine.clone()))
            .and_then(Self::handle_delete_operation);

        let git_sync = warp::path!("documents" / String / "sync")
            .and(warp::post())
            .and(with_crdt_engine(crdt_engine.clone()))
            .and(with_git_manager(git_manager.clone()))
            .map(|id: String, crdt: Arc<RwLock<CrdtEngine>>, git: Arc<RwLock<GitManager>>| {
                // Clone the ID for use in the spawned blocking task
                let id_clone = id.clone();
                let crdt_clone = crdt.clone();
                let git_clone = git.clone();

                // Return a response immediately
                let resp = warp::reply::json(&OperationResponse { success: true });

                // Use tokio::task::spawn_blocking instead of tokio::spawn
                // This allows us to run blocking git operations safely
                tokio::task::spawn_blocking(move || {
                    // Convert to a blocking sync operation
                    tokio::runtime::Handle::current().block_on(async {
                        if let Err(e) = Self::process_git_sync(id_clone, crdt_clone, git_clone).await {
                            eprintln!("Error in git sync: {}", e);
                        }
                    });
                });

                resp
            });

        // Combine all routes
        let api = create_document
            .or(list_documents)
            .or(get_document)
            .or(insert_operation)
            .or(delete_operation)
            .or(git_sync)
            .or(ping);

        // Add CORS to the combined API
        api.with(warp::cors().allow_any_origin().allow_methods(vec!["GET", "POST"]))
    }

    /// Creates routes for this HTTP API instance - kept for backwards compatibility
    #[allow(dead_code)]
    fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        Self::create_routes(
            Arc::clone(&self.crdt_engine),
            Arc::clone(&self.network_engine),
            Arc::clone(&self.git_manager)
        )
    }

    async fn handle_create_document(
        req: CreateDocumentRequest,
        crdt_engine: Arc<RwLock<CrdtEngine>>,
    ) -> Result<impl Reply, Infallible> {
        let result: Result<warp::reply::Json, anyhow::Error> = async {
            let engine = crdt_engine.read().await;
            let document_id = engine.create_document(req.title, req.owner).await?;
            Ok(warp::reply::json(&CreateDocumentResponse { document_id }))
        }
        .await;

        Ok(match result {
            Ok(response) => response,
            Err(e) => warp::reply::json(&ErrorResponse {
                error: e.to_string(),
            }),
        })
    }

    async fn handle_list_documents(
        crdt_engine: Arc<RwLock<CrdtEngine>>,
    ) -> Result<impl Reply, Infallible> {
        let result: Result<warp::reply::Json, anyhow::Error> = async {
            let engine = crdt_engine.read().await;
            let documents = engine.list_documents().await?;

            // Create a vector of futures for each document
            let doc_info_futures = documents
                .into_iter()
                .map(|doc| async move {
                    let doc = doc.read().await;
                    DocumentInfo {
                        id: doc.id,
                        title: doc.title.clone(),
                        owner: doc.owner.clone(),
                        collaborators: doc.collaborators.iter().cloned().collect(),
                        repository_url: doc.repository_url.clone(),
                        created_at: doc.created_at.to_rfc3339(),
                        updated_at: doc.updated_at.to_rfc3339(),
                    }
                })
                .collect::<Vec<_>>();

            // Await all futures concurrently
            let doc_infos = futures::future::join_all(doc_info_futures).await;

            Ok(warp::reply::json(&DocumentListResponse {
                documents: doc_infos,
            }))
        }
        .await;

        Ok(match result {
            Ok(response) => response,
            Err(e) => warp::reply::json(&ErrorResponse {
                error: e.to_string(),
            }),
        })
    }

    async fn handle_get_document(
        id: String,
        crdt_engine: Arc<RwLock<CrdtEngine>>,
    ) -> Result<impl Reply, Infallible> {
        let result: Result<warp::reply::Json, anyhow::Error> = async {
            let doc_id = Uuid::parse_str(&id)
                .map_err(|_| anyhow::anyhow!(AppError::InvalidUuid(id.clone())))?;

            let engine = crdt_engine.read().await;
            let document = engine.get_document(&doc_id).await?;
            let doc = document.read().await;

            let doc_info = DocumentInfo {
                id: doc.id,
                title: doc.title.clone(),
                owner: doc.owner.clone(),
                collaborators: doc.collaborators.iter().cloned().collect(),
                repository_url: doc.repository_url.clone(),
                created_at: doc.created_at.to_rfc3339(),
                updated_at: doc.updated_at.to_rfc3339(),
            };

            Ok(warp::reply::json(&doc_info))
        }
        .await;

        Ok(match result {
            Ok(response) => response,
            Err(e) => warp::reply::json(&ErrorResponse {
                error: e.to_string(),
            }),
        })
    }

    async fn handle_insert_operation(
        id: String,
        req: InsertOperationRequest,
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        network_engine: Arc<RwLock<NetworkEngine>>,
    ) -> Result<impl Reply, Infallible> {
        let result: Result<warp::reply::Json, anyhow::Error> = async {
            let doc_id = Uuid::parse_str(&id)
                .map_err(|_| anyhow::anyhow!(AppError::InvalidUuid(id.clone())))?;

            let engine = crdt_engine.read().await;

            // Create the operation
            let operation = DocumentOperation::Insert {
                document_id: doc_id,
                user_id: req.user_id,
                position: req.position,
                content: req.content,
            };

            // Apply locally
            let encoded = engine.apply_local_operation(&doc_id, operation).await?;

            // Broadcast to network
            let mut network = network_engine.write().await;
            network.broadcast_operation(&doc_id, encoded).await?;

            Ok(warp::reply::json(&OperationResponse { success: true }))
        }
        .await;

        Ok(match result {
            Ok(response) => response,
            Err(e) => warp::reply::json(&ErrorResponse {
                error: e.to_string(),
            }),
        })
    }

    async fn handle_delete_operation(
        id: String,
        req: DeleteOperationRequest,
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        network_engine: Arc<RwLock<NetworkEngine>>,
    ) -> Result<impl Reply, Infallible> {
        let result: Result<warp::reply::Json, anyhow::Error> = async {
            let doc_id = Uuid::parse_str(&id)
                .map_err(|_| anyhow::anyhow!(AppError::InvalidUuid(id.clone())))?;

            let engine = crdt_engine.read().await;

            // Create the operation
            let operation = DocumentOperation::Delete {
                document_id: doc_id,
                user_id: req.user_id,
                range: req.start..req.end,
            };

            // Apply locally
            let encoded = engine.apply_local_operation(&doc_id, operation).await?;

            // Broadcast to network
            let mut network = network_engine.write().await;
            network.broadcast_operation(&doc_id, encoded).await?;

            Ok(warp::reply::json(&OperationResponse { success: true }))
        }
        .await;

        Ok(match result {
            Ok(response) => response,
            Err(e) => warp::reply::json(&ErrorResponse {
                error: e.to_string(),
            }),
        })
    }

    async fn process_git_sync(
        id: String,
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        git_manager: Arc<RwLock<GitManager>>,
    ) -> Result<(), anyhow::Error> {
        let doc_id = Uuid::parse_str(&id)
            .map_err(|_| anyhow::anyhow!(AppError::InvalidUuid(id.clone())))?;

        // Get document data first
        let doc_content = {
            let engine = crdt_engine.read().await;
            engine.get_document_content(&doc_id).await?
        };

        // The key point here is to avoid holding a lock on git_manager while performing Git operations
        // that could prevent the Send trait from being implemented
        {
            let mut manager = git_manager.write().await;
            // This is a blocking operation that should be safe to use with tokio::task::spawn_blocking
            // Instead of using async code with git2, which isn't Send/Sync
            manager.sync_document_blocking(&doc_id, doc_content)?;
        }

        Ok(())
    }
}

// Helper functions to extract dependencies
fn with_crdt_engine(
    crdt_engine: Arc<RwLock<CrdtEngine>>,
) -> impl Filter<Extract = (Arc<RwLock<CrdtEngine>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || crdt_engine.clone())
}

fn with_network_engine(
    network_engine: Arc<RwLock<NetworkEngine>>,
) -> impl Filter<Extract = (Arc<RwLock<NetworkEngine>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || network_engine.clone())
}

fn with_git_manager(
    git_manager: Arc<RwLock<GitManager>>,
) -> impl Filter<Extract = (Arc<RwLock<GitManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || git_manager.clone())
}

use anyhow::Result;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::document_persistence_service::DocumentPersistenceService;

/// Request to save a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDocumentRequest {
    pub content: Option<String>,
}

/// Response from saving a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDocumentResponse {
    pub success: bool,
    pub message: String,
}

/// Enhanced API routes for document persistence
pub struct DocumentPersistenceApi {
    persistence_service: Arc<DocumentPersistenceService>,
}

impl DocumentPersistenceApi {
    pub fn new(persistence_service: Arc<DocumentPersistenceService>) -> Self {
        Self {
            persistence_service,
        }
    }

    /// Clone the persistence service to avoid borrowing issues
    pub fn clone_persistence_service(&self) -> Arc<DocumentPersistenceService> {
        Arc::clone(&self.persistence_service)
    }

    /// Create enhanced document persistence routes
    pub fn routes(persistence_service: Arc<DocumentPersistenceService>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        // Route for saving a document
        let save_document = warp::path!("api" / "documents" / String / "save")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_persistence_service(persistence_service.clone()))
            .and_then(Self::handle_save_document);

        // Route for checking if document exists
        let check_document = warp::path!("api" / "documents" / String / "check")
            .and(warp::get())
            .and(with_persistence_service(persistence_service.clone()))
            .and_then(Self::handle_check_document);

        // Combine routes
        save_document.or(check_document)
            .with(warp::cors()
                .allow_any_origin()
                .allow_headers(vec!["content-type", "x-user-id", "authorization"])
                .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]))
    }

    /// Save a document
    async fn handle_save_document(
        id: String,
        req: SaveDocumentRequest,  // Removing underscore to allow for future use
        persistence_service: Arc<DocumentPersistenceService>,
    ) -> Result<impl Reply, Rejection> {
        tracing::info!("Saving document: {}", id);

        // Log request content if available
        if let Some(content) = &req.content {
            tracing::debug!("Document content length: {} bytes", content.len());

            // TODO: In a future implementation, we would save the content here
            // For now, we'll just check if content exists but not actually use it
            tracing::debug!("Content provided for document: {}", id);
        } else {
            tracing::debug!("No content provided for document: {}", id);
        }

        let document_id = match Uuid::parse_str(&id) {
            Ok(id) => id,
            Err(_) => return Ok(warp::reply::json(&SaveDocumentResponse {
                success: false,
                message: "Invalid document ID format".to_string(),
            })),
        };

        // Attempt to save the document
        match persistence_service.save_document(&document_id).await {
            Ok(_) => {
                tracing::info!("Document saved successfully: {}", id);
                Ok(warp::reply::json(&SaveDocumentResponse {
                    success: true,
                    message: "Document saved successfully".to_string(),
                }))
            }
            Err(e) => {
                tracing::error!("Error saving document {}: {:?}", id, e);
                Ok(warp::reply::json(&SaveDocumentResponse {
                    success: false,
                    message: format!("Error saving document: {}", e),
                }))
            }
        }
    }

    /// Check if a document exists and can be accessed
    async fn handle_check_document(
        id: String,
        persistence_service: Arc<DocumentPersistenceService>,
    ) -> Result<impl Reply, Rejection> {
        tracing::info!("Checking document existence: {}", id);

        let document_id = match Uuid::parse_str(&id) {
            Ok(id) => id,
            Err(_) => return Ok(warp::reply::json(&SaveDocumentResponse {
                success: false,
                message: "Invalid document ID format".to_string(),
            })),
        };

        // Use the branch manager to check if the document exists
        let branch_manager = persistence_service.branch_manager();
        match branch_manager.ensure_document_exists(&document_id, &format!("Document {}", id)).await {
            Ok(existed) => {
                let message = if existed {
                    "Document exists".to_string()
                } else {
                    "Document was created".to_string()
                };

                Ok(warp::reply::json(&SaveDocumentResponse {
                    success: true,
                    message,
                }))
            }
            Err(e) => {
                tracing::error!("Error checking document {}: {:?}", id, e);
                Ok(warp::reply::json(&SaveDocumentResponse {
                    success: false,
                    message: format!("Error checking document: {}", e),
                }))
            }
        }
    }
}

// Helper function to include the persistence service in route handlers
fn with_persistence_service(
    persistence_service: Arc<DocumentPersistenceService>,
) -> impl Filter<Extract = (Arc<DocumentPersistenceService>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || persistence_service.clone())
}

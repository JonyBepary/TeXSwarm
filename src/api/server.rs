use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::api::http::HttpApi;
use crate::api::websocket::WebSocketServer;
use crate::api::document_persistence_api::DocumentPersistenceApi;
use crate::crdt::engine::CrdtEngine;
use crate::git::manager::GitManager;
use crate::network::engine::NetworkEngine;
use crate::storage::document_persistence_service::DocumentPersistenceService;
use crate::utils::config::Config;

pub struct ApiServer {
    http_api: HttpApi,
    websocket_server: WebSocketServer,
    document_persistence_api: Option<DocumentPersistenceApi>,
    config: Config,
    // Optional heartbeat task handle
    heartbeat_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl ApiServer {
    pub fn new(
        config: &Config,
        crdt_engine: Arc<RwLock<CrdtEngine>>,
        network_engine: Arc<RwLock<NetworkEngine>>,
        git_manager: Arc<RwLock<GitManager>>,
    ) -> Result<Self> {
        let http_api = HttpApi::new(
            Arc::clone(&crdt_engine),
            Arc::clone(&network_engine),
            Arc::clone(&git_manager),
        );

        let websocket_server = WebSocketServer::new(
            Arc::clone(&crdt_engine),
        );

        // Document persistence API is initialized later when the persistence service is available
        let document_persistence_api = None;

        Ok(Self {
            http_api,
            websocket_server,
            document_persistence_api,
            config: config.clone(),
            heartbeat_task: Arc::new(RwLock::new(None)),
        })
    }

    /// Set the document persistence service
    pub fn set_persistence_service(&mut self, persistence_service: Arc<DocumentPersistenceService>) {
        self.document_persistence_api = Some(DocumentPersistenceApi::new(persistence_service));
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting HTTP API server...");
        info!("Binding HTTP API to {}:{}", self.config.server.api_host, self.config.server.api_port);
        self.http_api.start(&self.config).await?;
        info!("HTTP API server started successfully on {}:{}", self.config.server.api_host, self.config.server.api_port);

        info!("Starting WebSocket server...");
        info!("Binding WebSocket to {}:{}", self.config.server.ws_host, self.config.server.ws_port);
        self.websocket_server.start(&self.config).await?;
        info!("WebSocket server started successfully on {}:{}", self.config.server.ws_host, self.config.server.ws_port);

        // Start document persistence API if available
        if let Some(persistence_api) = &self.document_persistence_api {
            info!("Starting Document Persistence API...");

            // Get the routes from the document persistence API
            let persistence_routes = persistence_api.routes();

            // Combine with the existing routes (this is done separately since we can't modify HTTP API)
            let addr = format!("{}:{}", self.config.server.api_host, self.config.server.api_port)
                .parse::<std::net::SocketAddr>()
                .map_err(|e| anyhow::anyhow!("Failed to parse API address: {}", e))?;

            tokio::spawn(async move {
                info!("Document Persistence API routes mounted");
                warp::serve(persistence_routes).run(addr).await;
            });

            info!("Document Persistence API started successfully");
        }

        // Start the heartbeat task
        let websocket_server = self.websocket_server.clone();
        let heartbeat_task = tokio::spawn(async move {
            let heartbeat_interval = tokio::time::Duration::from_secs(30); // 30 seconds

            loop {
                tokio::time::sleep(heartbeat_interval).await;

                if let Err(e) = websocket_server.send_heartbeat().await {
                    tracing::error!("Error sending heartbeat: {:?}", e);
                }
            }
        });

        {
            let mut task_handle = self.heartbeat_task.write().await;
            *task_handle = Some(heartbeat_task);
        }

        info!("WebSocket heartbeat task started");

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Shutting down API servers...");

        // Stop the heartbeat task if it's running
        {
            let mut task_handle = self.heartbeat_task.write().await;
            if let Some(handle) = task_handle.take() {
                info!("Stopping WebSocket heartbeat task");
                handle.abort();
            }
        }

        // Currently, we don't have explicit stop methods for our servers as they
        // run in Tokio tasks. In a more complex application, we might use shutdown
        // signals or channels to gracefully terminate these services.

        Ok(())
    }
}

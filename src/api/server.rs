use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::api::http::HttpApi;
use crate::api::websocket::WebSocketServer;
use crate::crdt::engine::CrdtEngine;
use crate::git::manager::GitManager;
use crate::network::engine::NetworkEngine;
use crate::utils::config::Config;

pub struct ApiServer {
    http_api: HttpApi,
    websocket_server: WebSocketServer,
    config: Config,
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

        Ok(Self {
            http_api,
            websocket_server,
            config: config.clone(),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting HTTP API server...");
        self.http_api.start(&self.config).await?;
        info!("HTTP API server started successfully on {}:{}", self.config.server.api_host, self.config.server.api_port);

        info!("Starting WebSocket server...");
        self.websocket_server.start(&self.config).await?;
        info!("WebSocket server started successfully on {}:{}", self.config.server.ws_host, self.config.server.ws_port);

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Shutting down API servers...");

        // Currently, we don't have explicit stop methods for our servers as they
        // run in Tokio tasks. In a more complex application, we might use shutdown
        // signals or channels to gracefully terminate these services.

        Ok(())
    }
}

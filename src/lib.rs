pub mod api;
pub mod crdt;
pub mod git;
pub mod network;
pub mod protocol;
pub mod utils;
#[cfg(test)]
pub mod tests;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct P2PLatexCollab {
    pub crdt_engine: Arc<RwLock<crdt::engine::CrdtEngine>>,
    pub network_engine: Arc<RwLock<network::engine::NetworkEngine>>,
    pub git_manager: Arc<RwLock<git::manager::GitManager>>,
    pub api_server: Arc<api::server::ApiServer>,
}

impl P2PLatexCollab {
    pub async fn new(config: &utils::config::Config) -> anyhow::Result<Self> {
        let crdt_engine = Arc::new(RwLock::new(crdt::engine::CrdtEngine::new()?));
        let network_engine = Arc::new(RwLock::new(network::engine::NetworkEngine::new(&config.network, Arc::clone(&crdt_engine)).await?));
        let git_manager = Arc::new(RwLock::new(git::manager::GitManager::new(config, Arc::clone(&crdt_engine))?));
        let api_server = Arc::new(api::server::ApiServer::new(
            config,
            Arc::clone(&crdt_engine),
            Arc::clone(&network_engine),
            Arc::clone(&git_manager),
        )?);

        Ok(Self {
            crdt_engine,
            network_engine,
            git_manager,
            api_server,
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // Start the network engine
        {
            let mut network = self.network_engine.write().await;
            network.start().await?;
        }

        // Start the API server
        self.api_server.start().await?;

        Ok(())
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        // Stop the API server
        self.api_server.stop().await?;

        // Stop the network engine
        {
            let mut network = self.network_engine.write().await;
            network.stop().await?;
        }

        Ok(())
    }
}

// This script can be used to migrate from the mock NetworkService to the real implementation
// Run with: cargo run --bin migrate_to_real_network

use anyhow::Result;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use p2p_latex_collab::crdt::engine::CrdtEngine;
use p2p_latex_collab::network::engine::NetworkEngine;
use p2p_latex_collab::network::service::RealNetworkService;
use p2p_latex_collab::utils::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let _config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.json".to_string());
    let config = Config::load()?;

    println!("Migrating to real network implementation...");

    // Create CRDT engine
    let crdt_engine = Arc::new(RwLock::new(CrdtEngine::new()?));
    println!("Created CRDT engine");

    // Create network engine with the mock service
    let mut network_engine = NetworkEngine::new(&config.network, Arc::clone(&crdt_engine)).await?;
    println!("Created network engine with mock service");

    // Start the engine with the mock service to test it
    network_engine.start().await?;
    println!("Started network engine with mock service");

    // Get the local peer ID from the mock service
    let mock_peer_id = network_engine.get_local_peer_id().await?;
    println!("Mock service local peer ID: {}", mock_peer_id);

    // Stop the engine to clean up resources
    network_engine.stop().await?;
    println!("Stopped network engine");

    // Create the real network service
    let real_service = Arc::new(RealNetworkService::new(config.network.clone()).await?);
    println!("Created real network service");

    // Start the real service and get the event receiver
    let _event_receiver = Arc::clone(&real_service).start_event_loop().await?;
    println!("Started real network service event loop");

    // Get the local peer ID from the real service
    let real_peer_id = real_service.local_peer_id.to_string();
    println!("Real service local peer ID: {}", real_peer_id);

    println!("The real network service is ready to use!");
    println!("You can now replace the mock service with the real one in your application");

    Ok(())
}

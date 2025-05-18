use anyhow::Result;
use p2p_latex_collab::{utils::config::Config, P2PLatexCollab};
use tracing::{info, Level, debug};
use tracing_subscriber::FmtSubscriber;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let level = match log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting P2P LaTeX Collaboration Server");

    // Load configuration
    let mut config = Config::load()?;

    // Force binding to all interfaces, overriding any configuration
    config.server.api_host = "0.0.0.0".to_string();
    config.server.ws_host = "0.0.0.0".to_string();

    debug!("Server configuration: {:?}", config.server);
    info!("Configuration loaded successfully with hosts set to 0.0.0.0");

    // Initialize and start the application
    let app = P2PLatexCollab::new(&config).await?;
    info!("Application initialized successfully");

    // Start the application components
    app.start().await?;
    info!("Application started successfully");

    // Wait for termination signal
    tokio::signal::ctrl_c().await?;
    info!("Received termination signal, shutting down...");

    // Stop the application components
    app.stop().await?;
    info!("Application stopped successfully");

    Ok(())
}

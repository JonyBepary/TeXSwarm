use anyhow::Result;
use p2p_latex_collab::{utils::config::Config, P2PLatexCollab};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting P2P LaTeX Collaboration Server");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

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

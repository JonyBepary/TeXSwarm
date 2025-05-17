use anyhow::Result;
use p2p_latex_collab::{
    P2PLatexCollab,
    utils::config::{Config, GitConfig, NetworkConfig, ServerConfig, StorageConfig},
    crdt::operations::DocumentOperation
};
use std::time::Duration;
// Remove the unused Uuid import
// use uuid::Uuid;

/// This test verifies basic end-to-end functionality of the system, including:
/// - Document creation
/// - Applying operations
/// - Network message propagation
/// - WebSocket functionality
#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Starting end-to-end test...");

    // Create two instances of the application with different configurations
    let config1 = create_test_config(1);
    let config2 = create_test_config(2);

    // Ensure the configs have different network ports
    assert_ne!(config1.network.listen_addresses, config2.network.listen_addresses);

    // Start the first instance
    let app1 = P2PLatexCollab::new(&config1).await?;
    app1.start().await?;

    println!("First instance started");

    // Start the second instance
    let app2 = P2PLatexCollab::new(&config2).await?;
    app2.start().await?;

    println!("Second instance started");

    // Give some time for the network to initialize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Create a document in the first instance
    let engine1 = app1.crdt_engine.clone();
    let document_id = {
        let engine = engine1.read().await;
        engine.create_document("Test Document".to_string(), "user1".to_string()).await?
    };

    println!("Document created with ID: {}", document_id);

    // Insert some text into the document
    {
        let engine = engine1.read().await;
        let operation = DocumentOperation::Insert {
            document_id,
            user_id: "user1".to_string(),
            position: 0,
            content: "Hello, collaborative world!".to_string(),
        };
        engine.apply_local_operation(&document_id, operation).await?;
    }

    println!("Text inserted into the document");

    // Tell the second instance to join the document
    let engine2 = app2.crdt_engine.clone();
    {
        let network_engine = app2.network_engine.clone();
        let mut net_engine = network_engine.write().await;
        net_engine.subscribe_to_document(document_id).await?;
    }

    println!("Second instance subscribed to the document");

    // Give some time for synchronization
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check if the document content is the same in both instances
    let content1 = {
        let engine = engine1.read().await;
        engine.get_document_content(&document_id).await?
    };

    let content2 = {
        let engine = engine2.read().await;
        engine.get_document_content(&document_id).await?
    };

    println!("Content in first instance: {}", content1);
    println!("Content in second instance: {}", content2);

    assert_eq!(content1, content2, "Document content should be the same in both instances");

    // Apply another operation from the second instance
    {
        let engine = engine2.read().await;
        let operation = DocumentOperation::Insert {
            document_id,
            user_id: "user2".to_string(),
            position: content2.len(),
            content: " This is a collaborative edit!".to_string(),
        };
        engine.apply_local_operation(&document_id, operation).await?;
    }

    println!("Text inserted from second instance");

    // Give some time for synchronization
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check if the updates propagated correctly
    let content1_after = {
        let engine = engine1.read().await;
        engine.get_document_content(&document_id).await?
    };

    let content2_after = {
        let engine = engine2.read().await;
        engine.get_document_content(&document_id).await?
    };

    println!("Updated content in first instance: {}", content1_after);
    println!("Updated content in second instance: {}", content2_after);

    assert_eq!(content1_after, content2_after, "Document content should be synchronized after updates");

    // Shut down both instances
    app1.stop().await?;
    app2.stop().await?;

    println!("End-to-end test completed successfully!");

    Ok(())
}

/// Create a test configuration with unique ports
fn create_test_config(instance_id: u16) -> Config {
    let api_port = 3000 + instance_id;
    let ws_port = 3100 + instance_id;
    let p2p_port = 4000 + instance_id;

    Config {
        server: ServerConfig {
            api_host: "127.0.0.1".to_string(),
            api_port,
            ws_host: "127.0.0.1".to_string(),
            ws_port,
        },
        network: NetworkConfig {
            listen_addresses: vec![
                format!("/ip4/127.0.0.1/tcp/{}", p2p_port)
            ],
            bootstrap_nodes: vec![
                // For the second instance, use the first instance's address as bootstrap
                if instance_id > 1 {
                    format!("/ip4/127.0.0.1/tcp/{}/p2p/PLACEHOLDER", 4000 + 1)
                } else {
                    "".to_string()
                }
            ].into_iter().filter(|s| !s.is_empty()).collect(),
            peer_id_seed: Some(format!("test-seed-{}", instance_id)),
            enable_mdns: true,
            enable_kad: true,
            external_addresses: vec![],
        },
        git: GitConfig {
            repositories_path: std::path::PathBuf::from(format!("./tmp/test-repos-{}", instance_id)),
            sync_interval_secs: 60,
            github_token: None,
            github_username: Some("test-user".to_string()),
            github_email: Some("test@example.com".to_string()),
        },
        storage: StorageConfig {
            documents_path: std::path::PathBuf::from(format!("./tmp/test-docs-{}", instance_id)),
            max_document_size_mb: 10,
            enable_autosave: true,
            autosave_interval_seconds: 60,
        },
    }
}

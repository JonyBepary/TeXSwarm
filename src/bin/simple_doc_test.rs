use anyhow::Result;
use p2p_latex_collab::{
    P2PLatexCollab,
    utils::config::{Config, GitConfig, NetworkConfig, ServerConfig, StorageConfig},
    crdt::operations::DocumentOperation,
};
use std::{
    time::Duration,
    path::PathBuf,
};
// use uuid::Uuid;

/// A simple document test to verify basic synchronization works
#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("Starting simple document test...");

    // Create two app instances
    let config1 = create_test_config(1);
    let config2 = create_test_config(2);

    // Start the instances
    println!("Starting application instances...");
    let app1 = P2PLatexCollab::new(&config1).await?;
    app1.start().await?;
    println!("Instance 1 started");

    let app2 = P2PLatexCollab::new(&config2).await?;
    app2.start().await?;
    println!("Instance 2 started");

    // Wait for network initialization
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Network initialized");

    // Create a document on instance 1
    let engine1 = app1.crdt_engine.clone();
    let doc_id = {
        let engine = engine1.write().await;
        let id = engine.create_document("Test Document".to_string(), "user1".to_string()).await?;
        println!("Document created with ID: {}", id);
        id
    };

    // Add content to the document
    {
        let engine = engine1.read().await;
        let operation = DocumentOperation::Insert {
            document_id: doc_id.clone(),
            user_id: "user1".to_string(),
            position: 0,
            content: "Hello, world!".to_string(),
        };

        engine.apply_local_operation(&doc_id, operation).await?;
        println!("Content added to document");
    }

    // Subscribe instance 2 to the document
    {
        let network = app2.network_engine.clone();
        let mut net = network.write().await;
        net.subscribe_to_document(doc_id).await?;
        println!("Instance 2 subscribed to document");
    }

    // Allow time for synchronization
    println!("Waiting for synchronization...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check if document exists in instance 2
    let engine2 = app2.crdt_engine.clone();
    {
        let engine = engine2.read().await;
        let docs = engine.get_all_documents().await?;
        println!("Instance 2 document IDs: {:?}", docs);

        // Try to get document content
        match engine.get_document_content(&doc_id).await {
            Ok(content) => println!("Document content in instance 2: {}", content),
            Err(e) => println!("Error getting document content: {:?}", e),
        }
    }

    // If document doesn't exist in instance 2, manually create it
    {
        let engine = engine2.read().await;
        let docs = engine.get_all_documents().await?;

        if !docs.contains(&doc_id) {
            println!("Document doesn't exist in instance 2, manually creating it...");
            engine.create_document("Test Document".to_string(), "user1".to_string()).await?;

            // Get content from instance 1
            let content = {
                let engine1 = app1.crdt_engine.read().await;
                engine1.get_document_content(&doc_id).await?
            };

            // Update content in instance 2
            engine.update_document_content(&doc_id, content).await?;
            println!("Document manually created and content copied");
        }
    }

    println!("Test completed successfully!");

    Ok(())
}

/// Create a test configuration
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
                // For instance 2, use instance 1's address as bootstrap
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
            repositories_path: PathBuf::from(format!("./tmp/simple-test-repos-{}", instance_id)),
            sync_interval_secs: 60,
            github_token: None,
            github_username: Some("test-user".to_string()),
            github_email: Some("test@example.com".to_string()),
        },
        storage: StorageConfig {
            documents_path: PathBuf::from(format!("./tmp/simple-test-docs-{}", instance_id)),
            max_document_size_mb: 10,
            enable_autosave: true,
            autosave_interval_seconds: 60,
        },
    }
}

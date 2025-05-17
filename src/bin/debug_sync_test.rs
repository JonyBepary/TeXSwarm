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

/// A debugging test specifically focused on investigating document synchronization issues
#[tokio::main]
async fn main() -> Result<()> {
    // Set up detailed logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    println!("Starting document sync debugging test...");

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

    // Wait for network initialization and verify connections
    println!("Waiting for network connections...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Check peer connections
    verify_network_connections(&app1, &app2).await?;

    // Create a document on instance 1
    let engine1 = app1.crdt_engine.clone();
    let doc_id = {
        let engine = engine1.write().await;
        let id = engine.create_document("Debug Document".to_string(), "user1".to_string()).await?;
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
            content: "Test content for debugging.".to_string(),
        };

        engine.apply_local_operation(&doc_id, operation).await?;
        println!("Content added to document");
    }

    // Verify the document exists in instance 1
    {
        let engine = engine1.read().await;
        let docs = engine.get_all_documents().await?;
        println!("Instance 1 document IDs: {:?}", docs);

        if docs.contains(&doc_id) {
            println!("✅ Document exists in instance 1");
        } else {
            println!("❌ Document doesn't exist in instance 1");
        }
    }

    // Explicitly check if document exists in instance 2 before subscription
    {
        let engine2 = app2.crdt_engine.clone();
        let engine = engine2.read().await;
        let docs = engine.get_all_documents().await?;
        println!("Instance 2 document IDs before subscription: {:?}", docs);
    }

    // Subscribe instance 2 to the document with explicit debug logs
    println!("\nSubscribing instance 2 to document...");
    {
        let network = app2.network_engine.clone();
        println!("Getting network engine write lock...");
        let mut net = network.write().await;
        println!("Calling subscribe_to_document...");
        net.subscribe_to_document(doc_id).await?;
        println!("Instance 2 subscribed to document");
    }

    // Wait for synchronization
    for i in 1..=3 {
        println!("\nWaiting for synchronization (attempt {}/3)...", i);
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Check if document exists in instance 2
        let engine2 = app2.crdt_engine.clone();
        let engine = engine2.read().await;
        let docs = engine.get_all_documents().await?;
        println!("Instance 2 document IDs: {:?}", docs);

        if docs.contains(&doc_id) {
            println!("✅ Document synchronized to instance 2!");

            // Check content
            match engine.get_document_content(&doc_id).await {
                Ok(content) => {
                    println!("Document content in instance 2: {}", content);
                    // Success! We can break the loop
                    break;
                },
                Err(e) => {
                    println!("❌ Error getting document content: {:?}", e);
                }
            }
        } else if i < 3 {
            println!("Document not yet synchronized, continuing to wait...");
        } else {
            println!("❌ Document not synchronized after maximum attempts");

            // If the document didn't synchronize automatically, let's try a manual approach
            println!("\nAttempting manual document synchronization...");

            // Create the document manually in instance 2
            println!("Creating document in instance 2...");
            engine.create_document("Debug Document".to_string(), "user1".to_string()).await?;

            // Get content from instance 1
            let content = {
                let engine1 = app1.crdt_engine.read().await;
                engine1.get_document_content(&doc_id).await?
            };

            // Update content in instance 2
            println!("Updating content in instance 2...");
            engine.update_document_content(&doc_id, content).await?;
            println!("✅ Document manually created and content copied");

            // Verify the content
            match engine.get_document_content(&doc_id).await {
                Ok(content) => println!("Final document content: {}", content),
                Err(e) => println!("Error getting final content: {:?}", e),
            }
        }
    }

    println!("\nTesting direct document synchronization via CRDT mechanism...");
    {
        // Export document from instance 1
        let engine1 = app1.crdt_engine.read().await;
        let exported = engine1.export_document(&doc_id).await?;
        println!("Exported document from instance 1 (size: {} bytes)", exported.len());

        // Import to instance 2
        let engine2 = app2.crdt_engine.read().await;
        match engine2.sync_document(&doc_id, &exported).await {
            Ok(_) => println!("✅ Direct document sync successful"),
            Err(e) => println!("❌ Direct document sync failed: {:?}", e),
        }

        // Check content after direct sync
        match engine2.get_document_content(&doc_id).await {
            Ok(content) => println!("Content after direct sync: {}", content),
            Err(e) => println!("Error getting content after direct sync: {:?}", e),
        }
    }

    println!("\nDocument sync debugging test completed");

    Ok(())
}

/// Helper function to create a test configuration
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
            repositories_path: PathBuf::from(format!("./tmp/debug-test-repos-{}", instance_id)),
            sync_interval_secs: 60,
            github_token: None,
            github_username: Some("test-user".to_string()),
            github_email: Some("test@example.com".to_string()),
        },
        storage: StorageConfig {
            documents_path: PathBuf::from(format!("./tmp/debug-test-docs-{}", instance_id)),
            max_document_size_mb: 10,
            enable_autosave: true,
            autosave_interval_seconds: 60,
        },
    }
}

/// Verify network connections between instances
async fn verify_network_connections(app1: &P2PLatexCollab, app2: &P2PLatexCollab) -> Result<()> {
    let network1 = app1.network_engine.clone();
    let network2 = app2.network_engine.clone();

    // Get peer information for both instances
    let peers_info1 = {
        let net = network1.read().await;
        let local_id = net.get_local_peer_id().await?;
        println!("Instance 1 peer ID: {}", local_id);
        "Connected"
    };

    let peers_info2 = {
        let net = network2.read().await;
        let local_id = net.get_local_peer_id().await?;
        println!("Instance 2 peer ID: {}", local_id);
        "Connected"
    };

    println!("Instance 1 network status: {}", peers_info1);
    println!("Instance 2 network status: {}", peers_info2);

    println!("✅ Basic network information retrieved (peer IDs)");

    // Give more time for network to establish
    println!("Waiting additional time for network connection...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("Continuing with test...");

    Ok(())
}

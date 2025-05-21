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

/// A focused test specifically for document synchronization
#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("Starting document synchronization test...");

    // Create multiple application instances with different ports
    let config1 = create_test_config(1);
    let config2 = create_test_config(2);

    // Start the instances
    println!("Starting application instances...");
    let app1 = P2PLatexCollab::new(&config1).await?;
    app1.start().await?;
    println!("Instance 1 started on ports 3001/3101/4001");

    let app2 = P2PLatexCollab::new(&config2).await?;
    app2.start().await?;
    println!("Instance 2 started on ports 3002/3102/4002");

    // Allow time for network initialization
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Network initialization complete");

    // Create a document on instance 1
    let engine1 = app1.crdt_engine.clone();
    let document_id = {
        let engine = engine1.read().await;
        let id = engine.create_document("Test Document".to_string(), "user1".to_string()).await?;
        println!("Document created on instance 1 with ID: {}", id);
        id
    };

    // Add some content to the document
    {
        let engine = engine1.read().await;
        let op = DocumentOperation::Insert {
            document_id: document_id.clone(),
            user_id: "user1".to_string(),
            position: 0,
            content: "Hello, synchronized document!".to_string(),
        };
        engine.apply_local_operation(&document_id, op).await?;
        println!("Added content to document on instance 1");
    }

    // Print document IDs in both instances (before subscription)
    print_document_info(&app1, 1).await?;
    print_document_info(&app2, 2).await?;

    // Manually print all documents in instances
    println!("\nListing all documents in each instance before subscription:");
    list_all_documents(&app1, 1).await?;
    list_all_documents(&app2, 2).await?;

    // Instance 2 subscribes to the document
    println!("\nInstance 2 subscribing to document {}...", document_id);
    {
        let network_engine = app2.network_engine.clone();
        let mut net = network_engine.write().await;
        net.subscribe_to_document(document_id).await?;
        println!("Instance 2 subscribed to document");
    }

    // Wait for synchronization
    println!("Waiting for synchronization (10 seconds)...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Print document IDs in both instances (after subscription)
    print_document_info(&app1, 1).await?;
    print_document_info(&app2, 2).await?;

    // Manually print all documents in instances
    println!("\nListing all documents in each instance after subscription:");
    list_all_documents(&app1, 1).await?;
    list_all_documents(&app2, 2).await?;

    // Try to get the document content in instance 2
    {
        let engine = app2.crdt_engine.read().await;
        match engine.get_document_content(&document_id).await {
            Ok(content) => {
                println!("\n✅ Successfully retrieved document content from instance 2: {}", content);
            },
            Err(e) => {
                println!("\n❌ Failed to retrieve document content from instance 2: {:?}", e);

                // Check if document exists in instance 2
                let doc_exists = {
                    let docs = engine.get_all_documents().await?;
                    if docs.contains(&document_id) {
                        println!("Document ID exists in instance 2, but content cannot be accessed.");
                        true
                    } else {
                        println!("Document ID does not exist in instance 2.");
                        false
                    }
                };

                if !doc_exists {
                    // Manually create the document in instance 2
                    println!("\nManually creating document in instance 2...");
                    engine.create_document("Test Document".to_string(), "user1".to_string()).await?;

                    // Get content from instance 1
                    let content1 = {
                        let eng1 = app1.crdt_engine.read().await;
                        eng1.get_document_content(&document_id).await?
                    };

                    // Update content in instance 2
                    engine.update_document_content(&document_id, content1).await?;
                    println!("Document manually created and synchronized in instance 2");
                }
            }
        }
    }

    // Try to modify the document in instance 2
    {
        let engine = app2.crdt_engine.read().await;
        let op = DocumentOperation::Insert {
            document_id: document_id.clone(),
            user_id: "user2".to_string(),
            position: 28,  // At the end of the previous content
            content: " And now edited from instance 2!".to_string(),
        };

        match engine.apply_local_operation(&document_id, op).await {
            Ok(_) => println!("\n✅ Successfully applied operation from instance 2"),
            Err(e) => println!("\n❌ Failed to apply operation from instance 2: {:?}", e),
        }
    }

    // Wait for synchronization
    println!("Waiting for synchronization (5 seconds)...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify document content in both instances
    println!("\nFinal document content in both instances:");
    {
        let engine1 = app1.crdt_engine.read().await;
        match engine1.get_document_content(&document_id).await {
            Ok(content) => println!("Instance 1 content: {}", content),
            Err(e) => println!("Failed to get content from instance 1: {:?}", e),
        }
    }

    {
        let engine2 = app2.crdt_engine.read().await;
        match engine2.get_document_content(&document_id).await {
            Ok(content) => println!("Instance 2 content: {}", content),
            Err(e) => println!("Failed to get content from instance 2: {:?}", e),
        }
    }

    println!("\nDocument synchronization test complete");

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
                // For instances 2 and 3, use instance 1's address as bootstrap
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
            repositories_path: PathBuf::from(format!("./tmp/doc-sync-test-repos-{}", instance_id)),
            sync_interval_secs: 60,
            github_token: None,
            github_username: Some("test-user".to_string()),
            github_email: Some("test@example.com".to_string()),
        },
        storage: StorageConfig {
            documents_path: PathBuf::from(format!("./tmp/doc-sync-test-docs-{}", instance_id)),
            max_document_size_mb: 10,
            enable_autosave: true,
            autosave_interval_seconds: 60,
        },
    }
}

/// Helper function to print document information for an instance
async fn print_document_info(app: &P2PLatexCollab, instance_id: usize) -> Result<()> {
    println!("\nInstance {} document info:", instance_id);

    let engine = app.crdt_engine.clone();
    let doc_ids = {
        let eng = engine.read().await;
        eng.get_all_documents().await?
    };

    println!("  Document count: {}", doc_ids.len());
    println!("  Document IDs: {:?}", doc_ids);

    Ok(())
}

/// Helper function to list all documents in an instance
async fn list_all_documents(app: &P2PLatexCollab, instance_id: usize) -> Result<()> {
    println!("Instance {} documents:", instance_id);

    let engine = app.crdt_engine.clone();
    let docs = {
        let eng = engine.read().await;
        eng.list_documents().await?
    };

    for (i, doc) in docs.iter().enumerate() {
        let doc_read = doc.read().await;
        println!("  Document {}: id={}, title={}, owner={}",
                i + 1, doc_read.id, doc_read.title, doc_read.owner);
    }

    if docs.is_empty() {
        println!("  No documents found");
    }

    Ok(())
}

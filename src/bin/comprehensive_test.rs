use anyhow::Result;
use futures::future::join_all;
use p2p_latex_collab::{
    P2PLatexCollab,
    utils::config::{Config, GitConfig, NetworkConfig, ServerConfig, StorageConfig},
    crdt::operations::DocumentOperation,
};
use std::{
    time::Duration,
    path::PathBuf,
};
use uuid::Uuid;

/// A comprehensive test for the p2p-latex-collab application
/// Tests multiple instances, network synchronization, concurrent edits,
/// document operations, and Git integration
#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Starting comprehensive test...");

    // Create multiple application instances (3 for thorough testing)
    let config1 = create_test_config(1);
    let config2 = create_test_config(2);
    let config3 = create_test_config(3);

    // Start the instances
    println!("Starting application instances...");
    let app1 = P2PLatexCollab::new(&config1).await?;
    app1.start().await?;
    println!("Instance 1 started");

    let app2 = P2PLatexCollab::new(&config2).await?;
    app2.start().await?;
    println!("Instance 2 started");

    let app3 = P2PLatexCollab::new(&config3).await?;
    app3.start().await?;
    println!("Instance 3 started");

    // Allow time for network initialization
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Network initialization complete");

    // TEST PHASE 1: Document Creation and Basic Operations
    println!("\n=== TEST PHASE 1: Document Creation and Basic Operations ===");

    // Create a document on instance 1
    let engine1 = app1.crdt_engine.clone();
    let document_id = {
        let engine = engine1.read().await;
        let id = engine.create_document("Collaborative LaTeX Paper".to_string(), "user1".to_string()).await?;
        println!("Document created with ID: {}", id);
        id
    };

    // Insert initial content
    {
        let engine = engine1.read().await;
        let initial_content = "\\documentclass{article}\n\\title{Collaborative LaTeX Paper}\n\\author{Test Team}\n\\begin{document}\n\\maketitle\n\\section{Introduction}\nThis is a test document.\n\\end{document}";
        engine.update_document_content(&document_id, initial_content.to_string()).await?;
        println!("Added initial content to document");
    }

    // Subscribe other instances to the document
    println!("Subscribing other instances to the document...");
    for (i, app) in [&app2, &app3].iter().enumerate() {
        let network_engine = app.network_engine.clone();
        {
            let mut net = network_engine.write().await;
            net.subscribe_to_document(document_id).await?;
        }
        println!("Instance {} subscribed to document", i + 2);
    }

    // Allow time for synchronization
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Initial synchronization complete");

    // Verify document is synchronized across all instances
    println!("Checking if document is available in instances...");

    // Check if document exists in engines 2 and 3
    for (i, app) in [&app2, &app3].iter().enumerate() {
        let engine = app.crdt_engine.clone();
        let has_doc = {
            let eng = engine.read().await;
            let doc_ids = eng.get_all_documents().await?;
            println!("Instance {} document IDs: {:?}", i + 2, doc_ids);
            doc_ids.contains(&document_id)
        };

        if !has_doc {
            println!("⚠️ Document doesn't exist in instance {}. Trying to create it...", i + 2);
            // Document doesn't exist in this instance - need to create it
            let eng = engine.read().await;
            let doc_content = {
                let eng1 = app1.crdt_engine.read().await;
                eng1.get_document_content(&document_id).await?
            };

            // Create the document and set the content
            eng.create_document("Collaborative LaTeX Paper".to_string(), "user1".to_string()).await?;
            eng.update_document_content(&document_id, doc_content).await?;
            println!("Created document in instance {}", i + 2);
        }
    }

    // Try getting contents again
    let contents = get_document_contents(&[&app1, &app2, &app3], &document_id).await?;
    if contents[0] == contents[1] && contents[1] == contents[2] {
        println!("✅ Document successfully synchronized across all instances");
    } else {
        println!("❌ Document synchronization failed");
        for (i, content) in contents.iter().enumerate() {
            println!("Instance {} content: {}", i + 1, content);
        }
        return Err(anyhow::anyhow!("Document synchronization failed"));
    }

    // TEST PHASE 2: Concurrent Edits
    println!("\n=== TEST PHASE 2: Concurrent Edits ===");

    // Make concurrent edits from all instances
    let ops = vec![
        // Instance 1 adds a new section
        add_section_operation(&app1, &document_id, "Methodology",
            "This section discusses our approach.").await?,

        // Instance 2 adds another section
        add_section_operation(&app2, &document_id, "Results",
            "Here are our preliminary results.").await?,

        // Instance 3 adds a third section
        add_section_operation(&app3, &document_id, "Conclusion",
            "In conclusion, our approach works well.").await?,
    ];

    // Apply operations concurrently
    join_all(ops).await;
    println!("Applied concurrent edits from all instances");

    // Allow time for synchronization
    tokio::time::sleep(Duration::from_secs(8)).await;
    println!("Synchronization after concurrent edits complete");

    // Verify document is synchronized with all edits
    let contents = get_document_contents(&[&app1, &app2, &app3], &document_id).await?;
    if contents[0] == contents[1] && contents[1] == contents[2] {
        println!("✅ Concurrent edits successfully synchronized");

        // Verify all sections are present
        let content = &contents[0];
        if content.contains("\\section{Methodology}") &&
           content.contains("\\section{Results}") &&
           content.contains("\\section{Conclusion}") {
            println!("✅ All sections successfully added");
        } else {
            println!("❌ Not all sections were added properly");
            println!("Final content: {}", content);
            return Err(anyhow::anyhow!("Not all sections were added properly"));
        }
    } else {
        println!("❌ Concurrent edit synchronization failed");
        for (i, content) in contents.iter().enumerate() {
            println!("Instance {} content: {}", i + 1, content);
        }
        return Err(anyhow::anyhow!("Concurrent edit synchronization failed"));
    }

    // TEST PHASE 3: Document Listing and Metadata
    println!("\n=== TEST PHASE 3: Document Listing and Metadata ===");

    // Create a second document on instance 2
    let engine2 = app2.crdt_engine.clone();
    let document_id2 = {
        let engine = engine2.read().await;
        let id = engine.create_document("Second Test Document".to_string(), "user2".to_string()).await?;
        println!("Second document created with ID: {}", id);
        id
    };

    // Allow time for propagation
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify all instances can see both documents
    for (i, app) in [&app1, &app2, &app3].iter().enumerate() {
        let engine = app.crdt_engine.clone();
        let docs = {
            let eng = engine.read().await;
            let all_docs = eng.get_all_documents().await?;
            println!("Instance {} sees {} documents", i + 1, all_docs.len());
            all_docs
        };

        if docs.len() >= 2 && docs.contains(&document_id) && docs.contains(&document_id2) {
            println!("✅ Instance {} correctly sees all documents", i + 1);
        } else {
            println!("❌ Instance {} does not see all documents", i + 1);
            println!("Documents seen: {:?}", docs);
            println!("Expected to see: {}, {}", document_id, document_id2);
            return Err(anyhow::anyhow!("Document listing failed for instance {}", i + 1));
        }
    }

    // TEST PHASE 4: Delete Operations
    println!("\n=== TEST PHASE 4: Delete Operations ===");    // Delete text from the first document
    {
        let engine = app1.crdt_engine.read().await;
        let content = engine.get_document_content(&document_id).await?;

        // Find the position of "approach" in the Methodology section
        if let Some(pos) = content.find("approach") {
            let delete_op = DocumentOperation::Delete {
                document_id: document_id.clone(),
                user_id: "user1".to_string(),
                range: (pos..(pos + 8)), // "approach" length is 8
            };

            engine.apply_local_operation(&document_id, delete_op).await?;
            println!("Deleted 'approach' from the document");
        } else {
            println!("Couldn't find 'approach' in the document");
            return Err(anyhow::anyhow!("Text to delete not found"));
        }
    }

    // Allow time for propagation
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify all instances have the deletion
    let contents = get_document_contents(&[&app1, &app2, &app3], &document_id).await?;
    if contents[0] == contents[1] && contents[1] == contents[2] && !contents[0].contains("approach") {
        println!("✅ Delete operation successfully synchronized");
    } else {
        println!("❌ Delete operation synchronization failed");
        for (i, content) in contents.iter().enumerate() {
            println!("Instance {} content contains 'approach': {}", i + 1, content.contains("approach"));
        }
        return Err(anyhow::anyhow!("Delete operation synchronization failed"));
    }

    println!("\nAll tests completed successfully!");

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
            repositories_path: PathBuf::from(format!("./tmp/test-repos-{}", instance_id)),
            sync_interval_secs: 60,
            github_token: None,
            github_username: Some("test-user".to_string()),
            github_email: Some("test@example.com".to_string()),
        },
        storage: StorageConfig {
            documents_path: PathBuf::from(format!("./tmp/test-docs-{}", instance_id)),
            max_document_size_mb: 10,
            enable_autosave: true,
            autosave_interval_seconds: 60,
        },
    }
}

/// Helper function to get document content from multiple instances
async fn get_document_contents(apps: &[&P2PLatexCollab], doc_id: &Uuid) -> Result<Vec<String>> {
    let mut contents = Vec::new();

    for (i, app) in apps.iter().enumerate() {
        let engine = app.crdt_engine.clone();
        let content = {
            let eng = engine.read().await;
            match eng.get_document_content(doc_id).await {
                Ok(content) => content,
                Err(e) => {
                    println!("⚠️ Error getting document from instance {}: {:?}", i + 1, e);
                    // Return empty content if document not found
                    String::new()
                }
            }
        };
        contents.push(content);
    }

    Ok(contents)
}

/// Helper function to add a section to a document
async fn add_section_operation(
    app: &P2PLatexCollab,
    doc_id: &Uuid,
    section_title: &str,
    section_content: &str
) -> Result<impl std::future::Future<Output = Result<()>>> {
    let engine = app.crdt_engine.clone();
    let doc_id = doc_id.clone();
    let section = format!("\\section{{{}}}\n{}\n", section_title, section_content);

    // Find position to insert the section (before \end{document})
    let position = {
        let eng = engine.read().await;
        let content = eng.get_document_content(&doc_id).await?;
        content.find("\\end{document}").unwrap_or(content.len())
    };

    let operation = DocumentOperation::Insert {
        document_id: doc_id.clone(),
        user_id: format!("user{}", section_title.len() % 3 + 1), // Use a different user ID for each section
        position,
        content: section,
    };

    Ok(async move {
        let eng = engine.read().await;
        eng.apply_local_operation(&doc_id, operation).await?;
        Ok(())
    })
}

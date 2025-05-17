use anyhow::Result;
use p2p_latex_collab::{P2PLatexCollab, utils::config::Config};
use std::path::PathBuf;
// Remove unused import
// use uuid::Uuid;

/// A simple end-to-end test for the application
#[tokio::main]
async fn main() -> Result<()> {
    // Create a basic configuration
    let mut config = Config::default();

    // Set up paths for this test
    config.git.repositories_path = PathBuf::from("/tmp/p2p-latex-test/repos");

    // Clean up any previous test directories
    let _ = std::fs::remove_dir_all(&config.git.repositories_path);
    std::fs::create_dir_all(&config.git.repositories_path)?;

    println!("Starting P2P LaTeX Collaboration tool test...");

    // Initialize the application
    let app = P2PLatexCollab::new(&config).await?;

    println!("Application initialized successfully");

    // Start the application
    app.start().await?;

    println!("Application started successfully");

    // Create a document
    let doc_id = {
        let engine = app.crdt_engine.read().await;
        let document_id = engine.create_document("Test Document".to_string(), "user1".to_string()).await?;
        println!("Created document with ID: {}", document_id);
        document_id
    };

    // Add some content to the document
    {
        let engine = app.crdt_engine.read().await;
        let content = "\\documentclass{article}\n\\begin{document}\nHello, world!\n\\end{document}";
        engine.update_document_content(&doc_id, content.to_string()).await?;
        println!("Updated document content");
    }

    // Verify document content
    {
        let engine = app.crdt_engine.read().await;
        let content = engine.get_document_content(&doc_id).await?;
        println!("Document content: {}", content);
    }

    // List all documents
    {
        let engine = app.crdt_engine.read().await;
        let doc_ids = engine.get_all_documents().await?;
        println!("Found {} documents", doc_ids.len());
        for id in doc_ids {
            println!(" - Document ID: {}", id);
        }
    }

    // Apply an operation to the document
    {
        use p2p_latex_collab::crdt::operations::DocumentOperation;

        let engine = app.crdt_engine.read().await;
        let operation = DocumentOperation::Insert {
            document_id: doc_id,
            user_id: "user1".to_string(),
            position: 46, // Position after "Hello, world!"
            content: " This is a collaborative edit.".to_string(),
        };

        let encoded = engine.apply_local_operation(&doc_id, operation).await?;
        println!("Applied operation to document (encoded size: {} bytes)", encoded.len());

        // Verify updated content
        let content = engine.get_document_content(&doc_id).await?;
        println!("Updated document content: {}", content);
    }

    // Simulate another user joining and making an edit
    {
        use p2p_latex_collab::crdt::operations::DocumentOperation;

        let engine = app.crdt_engine.read().await;
        let operation = DocumentOperation::Insert {
            document_id: doc_id,
            user_id: "user2".to_string(),
            position: 79, // Position at the end of the collaborative edit
            content: " Edited by user2.".to_string(),
        };

        let encoded = engine.apply_local_operation(&doc_id, operation).await?;
        println!("Applied second operation to document (encoded size: {} bytes)", encoded.len());

        // Verify updated content
        let content = engine.get_document_content(&doc_id).await?;
        println!("Final document content: {}", content);
    }

    // Stop the application
    app.stop().await?;

    println!("Application stopped successfully");
    println!("All tests completed successfully!");

    Ok(())
}

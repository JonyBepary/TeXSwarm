use anyhow::Result;
use crate::crdt::operations::DocumentOperation;
use crate::utils::config::Config;

#[tokio::test]
async fn test_crdt_operations() -> Result<()> {
    // Initialize a CRDT engine
    let engine = crate::crdt::engine::CrdtEngine::new()?;

    // Create a test document
    let doc_id = engine.create_document("Test Document".to_string(), "test-user".to_string()).await?;

    // Insert text
    let insert_op = DocumentOperation::Insert {
        document_id: doc_id.clone(),
        user_id: "test-user".to_string(),
        position: 0,
        content: "Hello, world!".to_string(),
    };

    let encoded = engine.apply_local_operation(&doc_id, insert_op).await?;

    // Apply the same operation as if it came from the network
    engine.apply_remote_operation(&doc_id, &encoded).await?;

    // Get the document content
    let content = engine.get_document_content(&doc_id).await?;

    // Verify that the text was inserted correctly (twice)
    assert_eq!(content, "Hello, world!Hello, world!");

    Ok(())
}

#[tokio::test]
async fn test_document_lifecycle() -> Result<()> {
    // Load config
    let config = Config::default();

    // Create app instance
    let app = crate::P2PLatexCollab::new(&config).await?;

    // Create a document through the CRDT engine
    let doc_id = {
        let engine = app.crdt_engine.write().await;
        engine.create_document("Test Document".to_string(), "test-user".to_string()).await?
    };

    // Apply an operation to the document
    {
        let engine = app.crdt_engine.read().await;
        let insert_op = DocumentOperation::Insert {
            document_id: doc_id.clone(),
            user_id: "test-user".to_string(),
            position: 0,
            content: "Hello, collaborative LaTeX!".to_string(),
        };

        engine.apply_local_operation(&doc_id, insert_op).await?;
    }

    // Get the document content
    let content = {
        let engine = app.crdt_engine.read().await;
        engine.get_document_content(&doc_id).await?
    };

    // Verify content
    assert_eq!(content, "Hello, collaborative LaTeX!");

    // List all documents
    let documents = {
        let engine = app.crdt_engine.read().await;
        engine.list_documents().await?
    };

    // Verify document count
    assert_eq!(documents.len(), 1);

    // Verify document properties
    let doc = documents[0].read().await;
    assert_eq!(doc.id, doc_id);
    assert_eq!(doc.title, "Test Document");
    assert_eq!(doc.owner, "test-user");

    Ok(())
}

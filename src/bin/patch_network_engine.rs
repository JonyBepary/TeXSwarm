use anyhow::Result;
use std::path::Path;
use std::fs;

/// A utility script to patch the NetworkEngine with the missing get_local_peer_id method
/// This is a temporary solution until the main code is properly fixed
#[tokio::main]
async fn main() -> Result<()> {
    println!("Patching NetworkEngine with missing get_local_peer_id method...");

    let engine_path = Path::new("/home/jony/rust-pract/p2p-latex-collab/src/network/engine.rs");

    if !engine_path.exists() {
        println!("NetworkEngine file not found at expected path!");
        return Ok(());
    }

    // Read the current file
    let content = fs::read_to_string(engine_path)?;

    // Check if method already exists
    if content.contains("get_local_peer_id") {
        println!("Method already exists, no patching needed.");
        return Ok(());
    }

    // Find the right location to add our method - after the start method
    let with_patch = if let Some(pos) = content.find("pub async fn stop(&mut self)") {
        let (before, after) = content.split_at(pos);
        format!("{}
    /// Get the local peer ID
    pub async fn get_local_peer_id(&self) -> Result<String> {{
        if let Some(service) = &self.service {{
            Ok(service.local_peer_id.to_string())
        }} else {{
            Err(anyhow::anyhow!(AppError::NetworkError(\"Network service not initialized\".to_string())))
        }}
    }}

    {}", before, after)
    } else {
        println!("Could not find insertion point, manual patching required.");
        return Ok(());
    };

    // Write back the patched file
    fs::write(engine_path, with_patch)?;

    println!("✅ NetworkEngine successfully patched with get_local_peer_id method!");
    println!("You may need to rebuild the project for changes to take effect.");

    Ok(())
}

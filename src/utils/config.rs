use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub network: NetworkConfig,
    pub git: GitConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub api_host: String,
    pub api_port: u16,
    pub ws_host: String,
    pub ws_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub peer_id_seed: Option<String>,
    pub bootstrap_nodes: Vec<String>,
    pub listen_addresses: Vec<String>,
    pub external_addresses: Vec<String>,
    pub enable_mdns: bool,
    pub enable_kad: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub repositories_path: PathBuf,
    pub github_token: Option<String>,
    pub github_username: Option<String>,
    pub github_email: Option<String>,
    pub sync_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub documents_path: PathBuf,
    pub max_document_size_mb: u64,
    pub enable_autosave: bool,
    pub autosave_interval_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                api_host: "127.0.0.1".to_string(),
                api_port: 8080,
                ws_host: "127.0.0.1".to_string(),
                ws_port: 8081,
            },
            network: NetworkConfig {
                peer_id_seed: None,
                bootstrap_nodes: vec![],
                listen_addresses: vec!["/ip4/0.0.0.0/tcp/9000".to_string()],
                external_addresses: vec![],
                enable_mdns: true,
                enable_kad: true,
            },
            git: GitConfig {
                repositories_path: PathBuf::from("./repositories"),
                github_token: None,
                github_username: None,
                github_email: None,
                sync_interval_secs: 300,
            },
            storage: StorageConfig {
                documents_path: PathBuf::from("./documents"),
                max_document_size_mb: 50,
                enable_autosave: true,
                autosave_interval_seconds: 60,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        // Ensure the directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_str)?;

        Ok(())
    }

    fn config_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("org", "p2p-latex", "p2p-latex-collab") {
            proj_dirs.config_dir().join("config.json")
        } else {
            PathBuf::from("./config.json")
        }
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Document metadata and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub owner: String,
    pub collaborators: HashSet<String>,
    pub repository_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Document {
    pub fn new(id: Uuid, title: String, owner: String) -> Self {
        let now = chrono::Utc::now();

        Self {
            id,
            title,
            owner,
            collaborators: HashSet::new(),
            repository_url: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_collaborator(&mut self, user_id: String) -> bool {
        self.collaborators.insert(user_id)
    }

    pub fn remove_collaborator(&mut self, user_id: &str) -> bool {
        self.collaborators.remove(user_id)
    }

    pub fn is_collaborator(&self, user_id: &str) -> bool {
        user_id == self.owner || self.collaborators.contains(user_id)
    }

    pub fn set_repository_url(&mut self, url: String) {
        self.repository_url = Some(url);
        self.updated_at = chrono::Utc::now();
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = chrono::Utc::now();
    }
}

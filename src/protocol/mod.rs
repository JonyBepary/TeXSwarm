// All protocol message types in a common module
pub use crate::api::protocol::ApiMessage;
pub use crate::api::protocol::Operation;
pub use crate::network::protocol::{NetworkMessage, CollabRequest, CollabResponse};

// Re-export common types used in protocols
pub use uuid::Uuid;

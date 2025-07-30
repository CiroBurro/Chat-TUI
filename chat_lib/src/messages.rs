/// Messages module
// Necessary imports
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Type State is the alias of Arc<Mutex<Vec<Messages>>> and holds the state of the server
pub type State = Arc<Mutex<Vec<Message>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Sender's username
    pub user: String,

    /// Content of the message
    pub message: String,
}

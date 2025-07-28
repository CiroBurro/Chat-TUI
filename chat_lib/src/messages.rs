use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type State = Arc<Mutex<Vec<Message>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub user: String,
    pub message: String,
}

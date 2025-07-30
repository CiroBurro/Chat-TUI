/// Main function for the chat app server
// Necessary imports
use chat_lib::{handle_connection, messages::State, IP_ADDR, PORT};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init(); // Logging

    // State of the server, containes all the messages
    let state: State = Arc::new(Mutex::new(Vec::new()));

    // Listening for connection at the default socket address
    let socket_addr = format!("{IP_ADDR}:{PORT}");
    let listener = TcpListener::bind(socket_addr).await?;
    info!("Listening on: {}", listener.local_addr()?);

    // Main loop of the server, it accepts connection until it is stopped
    loop {
        let (stream, _) = listener.accept().await?;

        let state = Arc::clone(&state);
        // Spawn a tokio task for every connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                info!(?e, "failed to handle socket")
            }
        });
    }
}

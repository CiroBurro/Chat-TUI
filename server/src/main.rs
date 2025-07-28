use chat_lib::{handle_connection, messages::State, IP_ADDR, PORT};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let state: State = Arc::new(Mutex::new(Vec::new()));

    let socket_addr = format!("{IP_ADDR}:{PORT}");
    let listener = TcpListener::bind(socket_addr).await?;
    info!("Listening on: {}", listener.local_addr()?);

    loop {
        let (stream, addr) = listener.accept().await?;

        let state = Arc::clone(&state);
        tokio::spawn(async move {
            info!(?addr, "New connection:");

            if let Err(e) = handle_connection(stream, state).await {
                info!(?e, "failed to handle socket")
            }
        });
    }
}

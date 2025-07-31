/// Main function for the chat app server
// Necessary imports
use chat_lib::{args::Args, handle_connection, messages::State, IP_ADDR, PORT};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init(); // Logging

    // State of the server, containes all the messages
    let state: State = Arc::new(Mutex::new(Vec::new()));

    // Cli args
    let Args { ip, port } = Args::from_args();
    let ip_addr = if ip.is_some() {
        ip.unwrap()
    } else {
        IP_ADDR.to_string()
    };

    let port = if port.is_some() {
        port.unwrap()
    } else {
        PORT.to_string()
    };
    // Listening for connection at the default or specified socket address
    let socket_addr = format!("{ip_addr}:{port}");
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

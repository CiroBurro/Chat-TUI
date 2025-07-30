/// Library for the chat app client and server
// Necessary imports
use messages::State;
use request::{parse_request, Request};
use response::{get_response, Response};
use tokio::{io::AsyncWriteExt, net::TcpStream};

// Modules of the library
pub mod messages;
pub mod request;
pub mod response;

/// Default IP_ADDR of the server: 127.0.0.1 (localhost)
pub static IP_ADDR: &str = "127.0.0.1";

/// Default PORT of the server: 8080
pub static PORT: &str = "8080";

/// Handle connection function handles the connection for the server
///
/// Args:
///     - stream: tcp stream of the connection
///     - state: state of the server with all the messages
pub async fn handle_connection(mut stream: TcpStream, state: State) -> Result<(), anyhow::Error> {
    // Parse the request from the client
    let request: Request = parse_request(&mut stream).await?;
    // Generate a response
    let response: Response = get_response(request, state).await?;

    // Send the response
    stream.write_all(response.to_string().as_bytes()).await?;

    Ok(())
}

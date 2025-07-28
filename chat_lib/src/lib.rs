use messages::State;
use request::parse_request;
use response::get_response;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{request::Request, response::Response};

pub mod messages;
pub mod request;
pub mod response;

pub static IP_ADDR: &str = "192.168.8.170";
pub static PORT: &str = "8080";

pub async fn handle_connection(mut socket: TcpStream, state: State) -> Result<(), anyhow::Error> {
    let request: Request = parse_request(&mut socket).await?;
    let response: Response = get_response(request, state).await?;

    socket.write_all(response.to_string().as_bytes()).await?;

    Ok(())
}

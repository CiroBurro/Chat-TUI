/// Response module
// Necesary imports
use crate::{
    messages::{Message, State},
    request::{Method, Request},
};
use anyhow::anyhow;
use core::panic;
use std::collections::HashMap;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

/// Response struct representing an HTTP response
pub struct Response {
    /// Status of the response
    status: Status,

    /// Headers og the response
    headers: HashMap<String, String>,

    /// Body of the response
    pub body: String,
}

/// Status enum enumerates the possible status code of the response (only 200, 404 and 404 for this app)
pub enum Status {
    /// Status Ok: 200
    Ok,
    /// Status Not Found: 404
    NotFound,
    /// Status Bad Request: 400
    BadRequest,
}

// ToString implementation for Response struct: format the Response struct in the right way to be sent over the tcp stream
impl ToString for Response {
    fn to_string(&self) -> String {
        let status_line = self.status.to_string();
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\r\n");

        format!("{status_line}\r\n{headers}\r\n\r\n{}", self.body)
    }
}

// ToString implementation for Status Enum: format the status into the correct status line of the response (including http version)
impl ToString for Status {
    fn to_string(&self) -> String {
        let status_line = match self {
            Self::Ok => "HTTP/1.1 200 OK",
            Self::NotFound => "HTTP/1.1 404 NOT FOUND",
            Self::BadRequest => "HTTP/1.1 400 BAD REQUEST",
        };

        status_line.to_string()
    }
}

/// Get response function generate an appropriate HTTP response based on the request
///
/// Args:
///     - req: request from the client
///     - state: state of the server with all the messages
pub async fn get_response(req: Request, state: State) -> Result<Response, anyhow::Error> {
    // Check the request method and uri in order to return the correct response
    let (status, content_type, body) = match (req.method, req.uri.as_str()) {
        (Method::Get, "/messages") => {
            // Get method + /messages endpoint means that the client is asking for the messages
            let msgs = state.lock().await;

            (
                Status::Ok,
                "application/json".to_string(),
                serde_json::to_string(&*msgs)?, // the body of the response will be the messages in json format
            )
        }
        (Method::Post, "/messages") => {
            // Post method + /messages endpoint means that the client is trying to send a new message

            // Check if the response has a body
            if req.body.is_some() {
                // Read the message from the body and update the server state
                let body_content: String = req.body.unwrap();
                let msg = serde_json::from_str::<Message>(body_content.as_str())?;
                let mut msgs = state.lock().await;
                msgs.push(msg);
                (
                    Status::Ok,
                    "application/json".to_string(),
                    r#"{"status":"ok"}"#.to_string(),
                )
            } else {
                // If there's no message return a bad request status code
                (
                    Status::BadRequest,
                    "text/plain".to_string(),
                    "Invalid Message".to_string(),
                )
            }
        }
        _ => (
            Status::NotFound,
            "text/plain".to_string(),
            "Not Found".to_string(),
        ),
    };

    // Construct the headers
    let length = body.len();

    let mut headers = HashMap::new();

    headers.insert("Content-Type".to_string(), content_type);
    headers.insert("Content-Length".to_string(), length.to_string());

    let response = Response {
        status,
        headers,
        body,
    };

    Ok(response)
}

/// Parse response function construct a Response struct from the HTTP response
///
/// Args:
///     - stream: tcp stream of the connection
pub async fn parse_response(mut stream: &mut TcpStream) -> Result<Response, anyhow::Error> {
    let mut buf_reader = BufReader::new(&mut stream);

    // The buf reader reads the first line of the response
    let mut first_line = String::new();
    let _ = buf_reader.read_line(&mut first_line).await?;

    // The first line is splitted and its parts are used to fill the Response struct's fields
    let mut parts = first_line.split_whitespace();

    let _ = parts.next();
    let status_code: String = parts
        .next()
        .ok_or(anyhow!("Missing status code"))
        .map(Into::into)?;

    let status = match status_code.parse()? {
        200 => Status::Ok,
        404 => Status::NotFound,
        400 => Status::BadRequest,
        _ => panic!("Invalid status code"),
    };

    let mut headers = HashMap::new();

    // The buf reader reads a line in loop to read the headers
    loop {
        let mut line = String::new();
        let _ = buf_reader.read_line(&mut line).await?;

        let line = line.trim_end_matches(&['\r', '\n'][..]);

        if line.is_empty() {
            break;
        }

        // Each header is divided into key and value and then inserted into an HashMap
        let mut parts = line.splitn(2, ":");
        let key = parts.next().unwrap().trim();
        let value = parts.next().unwrap().trim();

        headers.insert(key.to_string(), value.to_string());
    }

    let body;

    // Check if the content length header exists
    if let Some(cl) = headers.get("Content-Length") {
        // The buf reader reads also the body
        let len = cl.parse()?;
        let mut buf = vec![0; len];
        buf_reader.read_exact(&mut buf).await?;
        body = String::from_utf8_lossy(&buf).into_owned();
    } else {
        // Returns an error because a response must have a body in this app
        return Err(anyhow!("Missing body"));
    }

    let response = Response {
        status,
        headers,
        body,
    };

    Ok(response)
}

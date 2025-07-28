use anyhow::anyhow;
use tracing::info;

use crate::{
    messages::{Message, State},
    request::{Method, Request},
};
use core::panic;
use std::collections::HashMap;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

pub struct Response {
    status: Status,
    headers: HashMap<String, String>,
    pub body: String,
}

pub enum Status {
    Ok,
    NotFound,
    BadRequest,
}

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

pub async fn get_response(req: Request, state: State) -> Result<Response, anyhow::Error> {
    //info!(?req, "request");

    let (status, content_type, body) = match (req.method, req.uri.as_str()) {
        (Method::Get, "/messages") => {
            let msgs = state.lock().await;

            (
                Status::Ok,
                "application/json".to_string(),
                serde_json::to_string(&*msgs)?,
            )
        }
        (Method::Post, "/messages") => {
            if req.body.is_some() {
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

pub async fn parse_response(mut stream: &mut TcpStream) -> Result<Response, anyhow::Error> {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut first_line = String::new();
    let _ = buf_reader.read_line(&mut first_line).await?;

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

    loop {
        let mut line = String::new();
        let _ = buf_reader.read_line(&mut line).await?;

        let line = line.trim_end_matches(&['\r', '\n'][..]);

        if line.is_empty() {
            break;
        }

        let mut parts = line.splitn(2, ":");
        let key = parts.next().unwrap().trim();
        let value = parts.next().unwrap().trim();

        headers.insert(key.to_string(), value.to_string());
    }

    let body;

    if let Some(cl) = headers.get("Content-Length") {
        let len = cl.parse()?;
        let mut buf = vec![0; len];
        buf_reader.read_exact(&mut buf).await?;
        body = String::from_utf8_lossy(&buf).into_owned();
    } else {
        return Err(anyhow!("Missing body"));
    }

    let response = Response {
        status,
        headers,
        body,
    };

    Ok(response)
}

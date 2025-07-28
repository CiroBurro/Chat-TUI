use std::{collections::HashMap, hash::Hash};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};
use tracing::info;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Hash)]
pub enum Method {
    Get,
    Post,
}

impl TryFrom<&str> for Method {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            m => Err(anyhow::anyhow!("unsupported method: {m}")),
        }
    }
}

pub async fn parse_request(mut stream: &mut TcpStream) -> Result<Request, anyhow::Error> {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut first_line = String::new();
    let _ = buf_reader.read_line(&mut first_line).await?;

    let mut parts = first_line.split_whitespace();

    let method: Method = parts
        .next()
        .ok_or(anyhow::anyhow!("Missing method"))
        .and_then(TryInto::try_into)?;

    let uri: String = parts
        .next()
        .ok_or(anyhow::anyhow!("Missing URI"))
        .map(Into::into)?;

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

    let body = if let Some(cl) = headers.get("Content-Length") {
        let len = cl.parse()?;
        info!(?len, "content-length");
        let mut buf = vec![0; len];
        buf_reader.read_exact(&mut buf).await?;
        let content = String::from_utf8_lossy(&buf).into_owned();
        Some(content)
    } else {
        None
    };

    let request = Request {
        method,
        uri,
        headers,
        body,
    };

    Ok(request)
}

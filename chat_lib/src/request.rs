/// Request module
// Necessary imports
use std::{collections::HashMap, hash::Hash};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

#[derive(Debug)]
/// Request struct representing an HTTP request
pub struct Request {
    /// Method of the request
    pub method: Method,

    /// Uri of the request
    pub uri: String,

    /// Headers of the request
    pub headers: HashMap<String, String>,

    /// Body if the request, if exists
    pub body: Option<String>,
}

#[derive(Debug, Hash)]
/// Method enum enumerates the possible methods of an HTTP request (only GET and POST for this app)
pub enum Method {
    /// Get HTTP method
    Get,
    /// Post HTTP method
    Post,
}

// TryFrom<&str> implementation for Method enum: tries to construct a Method from a &str
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

/// Parse request function parses the client request to construct a Request struct
///
/// Args:
///     - stream: tcp stream of the connection
pub async fn parse_request(mut stream: &mut TcpStream) -> Result<Request, anyhow::Error> {
    let mut buf_reader = BufReader::new(&mut stream);

    // Buf reader reads the first line of the request
    let mut first_line = String::new();
    let _ = buf_reader.read_line(&mut first_line).await?;

    // The first line is splitted and the parts are trasformend in the Request struct's fields
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

    // The buf reader reads in loop the other lines with the headers
    loop {
        let mut line = String::new();
        let _ = buf_reader.read_line(&mut line).await?;

        let line = line.trim_end_matches(&['\r', '\n'][..]);

        if line.is_empty() {
            break;
        }

        // Each header is splitted in key and value and then inserted in an HashMap
        let mut parts = line.splitn(2, ":");
        let key = parts.next().unwrap().trim();
        let value = parts.next().unwrap().trim();

        headers.insert(key.to_string(), value.to_string());
    }

    // If the content length headers exists the buf reader also reads the body of the request
    let body = if let Some(cl) = headers.get("Content-Length") {
        let len = cl.parse()?;
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

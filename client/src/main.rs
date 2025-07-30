/// Main function for the tui client
// Necesary imports
use anyhow::anyhow;
use chat_lib::{messages::Message, response::parse_response, IP_ADDR, PORT};
use std::io::stdin;
use tokio::{
    io::{stdout, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};
use tui::App;

mod tui;

// Login function: used to get the username
async fn login() -> Result<String, anyhow::Error> {
    let mut stdout = stdout();

    if cfg!(target_os = "windows") {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    } else {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }

    println!("Inserisci un nome utente:");

    stdout.write_all(b"\n\n> ").await?;
    stdout.flush().await?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let user = input.trim_end().to_string();

    Ok(user)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let user = login().await?;
    let (tx, mut rx) = mpsc::channel(100); // Tokio channel to send messages from one task to the other

    // Receiver handle task, it connects to the server gets all the messages in realtime
    let _ = tokio::spawn(async move {
        let socket_addr = format!("{}:{}", IP_ADDR, PORT); // Default socket address of the server

        loop {
            // Connection to the server
            let mut stream = TcpStream::connect(socket_addr.clone())
                .await
                .expect("Failed to connect");

            // Send GET request at /messages endpoint
            let request = "GET /messages HTTP/1.1\r\nHost: localhost\r\n\r\n";
            let _ = stream.write_all(request.as_bytes()).await;

            // Read the response and get the messages
            let response = parse_response(&mut stream)
                .await
                .expect("Failed to parse response");

            // Deserialize them from json to struct Vec<Message>
            let messages: Vec<Message> =
                serde_json::from_str::<Vec<Message>>(response.body.as_str())
                    .expect("Failed to deserialize messages");

            // Send them to the other task
            tx.send(messages).await.expect("Failed to send messages");

            // Delay of 100 ms
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Tui handle task
    let tui_handle = tokio::spawn(async move {
        // Initialize the terminal
        let mut terminal = ratatui::init();

        // Main loop of the client
        let result = App::new(user).run(&mut terminal, &mut rx).await;

        // Restore the terminal once the client is closed
        ratatui::restore();
        result
    });

    // Check the result of the tui handle task once it is complete and end the program
    if tui_handle.await.is_ok() {
        return Ok(());
    } else {
        return Err(anyhow!("TUI failed"));
    }
}

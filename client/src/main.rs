use anyhow::Ok;
use chat_lib::{messages::Message, response::parse_response, IP_ADDR, PORT};
use std::io::stdin;
use tokio::{
    io::{stdout, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};
use tui::App;

mod tui;

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
    let (tx, mut rx) = mpsc::channel(100);

    let _ = tokio::spawn(async move {
        let socket_addr = format!("{}:{}", IP_ADDR, PORT);

        loop {
            let mut stream = TcpStream::connect(socket_addr.clone())
                .await
                .expect("Failed to connect");

            let request = "GET /messages HTTP/1.1\r\nHost: localhost\r\n\r\n";
            let _ = stream.write_all(request.as_bytes()).await;

            let response = parse_response(&mut stream)
                .await
                .expect("Failed to parse response");

            let messages: Vec<Message> =
                serde_json::from_str::<Vec<Message>>(response.body.as_str())
                    .expect("Failed to deserialize messages");

            tx.send(messages).await.expect("Failed to send messages");

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    let _ = tokio::spawn(async move {
        let mut terminal = ratatui::init();
        let result = App::new(user).run(&mut terminal, &mut rx).await;
        ratatui::restore();
        result
    });

    tokio::signal::ctrl_c().await?;
    Ok(())
}

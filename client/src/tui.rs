use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    widgets::{Block, Borders, List, Paragraph},
    DefaultTerminal, Frame,
};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::mpsc::Receiver};

use tui_input::{backend::crossterm::EventHandler, Input};

use chat_lib::{messages::Message, IP_ADDR, PORT};

#[derive(Debug)]
pub struct App {
    user: String,
    input: Input,
    messages: Vec<Message>,
}

impl App {
    pub fn new(user: String) -> Self {
        Self {
            user,
            input: Input::default(),
            messages: vec![],
        }
    }

    pub async fn run(
        mut self,
        terminal: &mut DefaultTerminal,
        rx: &mut Receiver<Vec<Message>>,
    ) -> Result<(), anyhow::Error> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let msgs = rx.recv().await;

            if msgs.is_some() {
                self.messages = msgs.unwrap();
            }

            if event::poll(tokio::time::Duration::from_millis(100))? {
                let event: Event = event::read()?;

                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Enter => {
                            let socket_addr = format!("{}:{}", IP_ADDR, PORT);
                            let mut stream = TcpStream::connect(socket_addr).await?;

                            let message = self.input.value_and_reset();
                            let body =
                                format!(r#"{{"user":"{}", "message":"{message}"}}"#, self.user);
                            let request = format!("POST /messages HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len());
                            stream.write_all(request.as_bytes()).await?;
                        }
                        KeyCode::Char('c') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                return Ok(());
                            } else {
                                self.input.handle_event(&event);
                            }
                        }
                        _ => {
                            self.input.handle_event(&event);
                        }
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        let [messages_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![Constraint::Percentage(85), Constraint::Percentage(15)])
            .areas(frame.area());

        // Render messages
        let messages = self
            .messages
            .iter()
            .map(|message| format!("{}: {}", message.user, message.message));

        frame.render_widget(
            List::new(messages).block(
                Block::new()
                    .bold()
                    .fg(Color::Blue)
                    .borders(Borders::ALL)
                    .title("Messages"),
            ),
            messages_area,
        );

        // Render input
        let width = input_area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);

        let x = self.input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((input_area.x + x as u16, input_area.y + 1))
    }
}

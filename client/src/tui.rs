use chat_lib::{messages::Message, IP_ADDR, PORT};
/// TUI module to implement a tui with ratatui
// Needed imports
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    widgets::{Block, Borders, List, Paragraph},
    DefaultTerminal, Frame,
};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::mpsc::Receiver};
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Debug)]
pub struct App {
    /// App struct containing its state

    /// Username
    user: String,

    /// User input
    input: Input,

    /// Chat messages
    messages: Vec<Message>,
}

// Methods for the App struct
impl App {
    /// Constructor method
    ///
    /// Args:
    ///     - user: username
    pub fn new(user: String) -> Self {
        Self {
            user,
            input: Input::default(),
            messages: vec![],
        }
    }

    /// Run method runs the application in loop until it is stopped
    ///
    /// Args:
    ///     - terminal: the terminal instance
    ///     - rx: receiver for the messages over the channel between the two tasks
    pub async fn run(
        mut self,
        terminal: &mut DefaultTerminal,
        rx: &mut Receiver<Vec<Message>>,
    ) -> Result<(), anyhow::Error> {
        // Main loop of the client
        loop {
            // Draw a frame on the terminal
            terminal.draw(|frame| self.draw(frame))?;

            // Receives the messages and update the App's state
            let msgs = rx.recv().await;
            if msgs.is_some() {
                self.messages = msgs.unwrap();
            }

            // Check if there's an event in an interval of 100ms
            if event::poll(tokio::time::Duration::from_millis(100))? {
                let event: Event = event::read()?;

                // Handle the event
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Enter => {
                            // Connection to the server
                            let socket_addr = format!("{}:{}", IP_ADDR, PORT);
                            let mut stream = TcpStream::connect(socket_addr).await?;

                            // Get the message from the input and construct the POST request
                            let message = self.input.value_and_reset();
                            let body =
                                format!(r#"{{"user":"{}", "message":"{message}"}}"#, self.user);
                            let request = format!("POST /messages HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len());

                            // Send the POST request to the server
                            stream.write_all(request.as_bytes()).await?;
                        }
                        KeyCode::Char('c') => {
                            // End the program if event is CTRL-C
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                return Ok(());
                            } else {
                                // Write 'c' in the input
                                self.input.handle_event(&event);
                            }
                        }
                        _ => {
                            // Every other character is written in the input
                            self.input.handle_event(&event);
                        }
                    }
                }
            }
        }
    }

    /// Draw function draws the frame on the terminal
    ///
    /// Args:
    ///     - frame: frame to render
    fn draw(&self, frame: &mut Frame<'_>) {
        // Layout of the tui with two areas: one for the messages and the other for the input
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

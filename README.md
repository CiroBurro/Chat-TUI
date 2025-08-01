# Chat-TUI

Chat-TUI is a simple terminal-based chat application written in Rust, composed of three crates:
- **chat_lib**: a shared library containing core logic, data structures, and utilities for both client and server.
- **server**: a binary crate implementing the chat server.
- **client**: a binary crate implementing a TUI (Text User Interface) chat client.

## Features

- Asynchronous networking using Tokio.
- Simple and extensible message protocol.
- Multi-user support.
- TUI client for an interactive chat experience.
- Configurable IP address and port for both client and server.
- Clean separation between library, client, and server logic.

## Installation

1. Download the latest release from the [GitHub Releases page](https://github.com/CiroBurro/Chat-TUI/releases).
2. Unpack the archive and ensure the binaries (`server` and `client`) are executable.

Alternatively, you can build from source (requires Rust toolchain):

```sh
git clone https://github.com/CiroBurro/Chat-TUI.git
cd Chat-TUI
cargo build --release
```
The binaries will be in `target/release/`.

## Usage

### Server

Start the server (by default on `127.0.0.1:8080`):

```sh
./server
```

You can specify a custom IP and/or port:

```sh
./server --ip 0.0.0.0 --port 9000
```

#### Server CLI Arguments

- `-i`, `--ip <IP>`: Specify a different IP address (default: 127.0.0.1)
- `-p`, `--port <PORT>`: Specify a different port (default: 8080)

### Client

Start the TUI client (by default connects to `127.0.0.1:8080`):

```sh
./client
```

You can specify a custom server IP and/or port:

```sh
./client --ip 192.168.1.100 --port 9000
```

#### Client CLI Arguments

- `-i`, `--ip <IP>`: Server IP address to connect to (default: 127.0.0.1)
- `-p`, `--port <PORT>`: Server port to connect to (default: 8080)

## How it works

- The server listens for incoming TCP connections and manages chat state.
- Each client connects to the server, logs in with a username, and can send/receive messages in real time.
- The TUI client provides a simple, interactive interface in the terminal.

## Notes

- This project is meant for educational e demonstration purposes, not for daily use, thus it may lack some production-ready features like authentication, encryption, etc.


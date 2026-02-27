use chx::{ChxClient, ChxError, dbg_println, server};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Runs the application in server mode
    Server {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 3800)]
        port: u16,
        #[arg(long, default_value_t = 2592000)]
        expire_time: u64,
    },
    /// Runs the application in client mode
    Client {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 3800)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), ChxError> {
    let cli = Cli::parse();

    let mode = if let Some(mode) = cli.mode {
        mode
    } else {
        // Default to server mode if no subcommand is provided
        Mode::Server {
            host: "127.0.0.1".to_string(),
            port: 3800,
            expire_time: 2592000,
        }
    };

    match mode {
        Mode::Server {
            host,
            port,
            expire_time,
        } => server(&host, port, expire_time).await,
        Mode::Client { host, port } => repl(host, port).await,
    }
}

async fn repl(host: String, port: u16) -> Result<(), ChxError> {
    let addr = format!("{}:{}", host, port);
    let current_addr = addr.clone();

    // Initial connection attempt
    let mut client_option = loop {
        match ChxClient::connect(&current_addr).await {
            Ok(client) => {
                println!(
                    "Connected to Chx server at {}. Type 'help' for commands.",
                    current_addr
                );
                break Some(client);
            }
            Err(e) => {
                eprintln!(
                    "Failed to connect to {}: {}. Retrying in 3 seconds...",
                    current_addr, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
    };

    let mut rl = rustyline::DefaultEditor::new()?;

    loop {
        let readline = rl.readline("chx> ");
        let line = match readline {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("Ctrl-D detected. Exiting Chx client.");
                break Ok(());
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("Ctrl-C detected. Exiting Chx client.");
                break Ok(());
            }
            Err(e) => {
                dbg_println!("Error reading line: {:?}", e);
                break Err(ChxError::Other(format!("Readline error: {}", e)));
            }
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        rl.add_history_entry(line)?;

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let command_result = async {
            let client = client_option.as_mut().expect("Client should be connected");
            match parts[0].to_lowercase().as_str() {
                "get" => {
                    if parts.len() == 2 {
                        let key = parts[1];
                        match client.get(key).await {
                            Ok(Some(value)) => println!("{}", value),
                            Ok(None) => println!("Key not found"),
                            Err(_e) => {
                                return Err(ChxError::Other(format!("Connection error: {}", _e)));
                            }
                        }
                    } else {
                        eprintln!("Usage: GET <key>");
                    }
                }
                "set" => {
                    if parts.len() >= 3 {
                        let key = parts[1];
                        let value = parts[2..].join(" ");
                        match client.set(key, &value).await {
                            Ok(_) => println!("Key set successfully"),
                            Err(_e) => {
                                return Err(ChxError::Other(format!("Connection error: {}", _e)));
                            }
                        }
                    } else {
                        eprintln!("Usage: SET <key> <value>");
                    }
                }
                "del" => {
                    if parts.len() == 2 {
                        let key = parts[1];
                        match client.del(key).await {
                            Ok(_) => println!("Key deleted successfully"),
                            Err(_e) => {
                                return Err(ChxError::Other(format!("Connection error: {}", _e)));
                            }
                        }
                    } else {
                        eprintln!("Usage: DEL <key>");
                    }
                }
                "quit" | "exit" => {
                    println!("Exiting Chx client.");
                    return Ok(());
                }
                "help" => {
                    println!("Available commands:");
                    println!("  GET <key>        - Retrieve the value associated with a key");
                    println!("  SET <key> <value> - Set a key-value pair");
                    println!("  DEL <key>        - Delete a key");
                    println!("  quit | exit      - Exit the client");
                }
                _ => {
                    eprintln!("Unknown command: {}", parts[0]);
                    println!("Type 'help' for available commands.");
                }
            }
            Ok(())
        }
        .await;

        if let Err(_e) = command_result {
            dbg_println!("Command error: {}. Attempting to reconnect...", _e);
            client_option = loop {
                match ChxClient::connect(&current_addr).await {
                    Ok(client) => {
                        println!("Reconnected to Chx server at {}.", current_addr);
                        break Some(client);
                    }
                    Err(_e) => {
                        dbg_println!(
                            "Failed to reconnect to {}: {}. Retrying in 3 seconds...",
                            current_addr,
                            _e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    }
                }
            };
        }
    }
}

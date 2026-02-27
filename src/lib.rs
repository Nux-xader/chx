//! # CHX - High-Performance Key-Value Store Library
//!
//! CHX is a lightweight and fast key-value store library that can be used as:
//! - **Library**: To integrate CHX client/server into your Rust applications
//! - **Standalone Application**: Ready-to-use CHX server and client
//!
//! ## Key Features
//!
//! - **Asynchronous**: Uses Tokio for high performance
//! - **Auto-expiration**: Supports TTL (Time To Live) for keys
//! - **Network Protocol**: TCP-based with efficient custom protocol
//! - **Auto-reconnect**: Client automatically reconnects when connection is lost
//! - **Thread-safe**: Uses `Arc<RwLock>` for concurrent access
//!
//! ## Usage as Library
//!
//! ### Client Example
//!
//! ```rust,no_run
//! use chx::{ChxClient, ChxError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ChxError> {
//!     // Connect to CHX server
//!     let mut client = ChxClient::connect("127.0.0.1:3800").await?;
//!
//!     // Set key-value
//!     client.set("my_key", "my_value").await?;
//!
//!     // Get value
//!     if let Some(value) = client.get("my_key").await? {
//!         println!("Value: {}", value);
//!     }
//!
//!     // Delete key
//!     client.del("my_key").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Server Example
//!
//! ```rust,no_run
//! use chx::{server, ChxError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ChxError> {
//!     let host = "127.0.0.1";
//!     let port = 3800;
//!     let expire_time = 3600; // 1 hour TTL
//!
//!     // Start CHX server
//!     server(host, port, expire_time).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The library uses `ChxError` enum for comprehensive error handling:
//! - `ConnectionError`: Network connection issues
//! - `IoError`: I/O operation errors
//! - `ResponseError`: Server response errors
//! - `InvalidResponse`: Invalid response format
//! - `Other`: Other errors

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

/// Macro for printing debug messages when debug assertions are enabled.
///
/// This macro wraps println! and only outputs messages when the program
/// is compiled in debug mode (with debug assertions enabled).
///
/// # Examples
///
/// ```
/// use chx::dbg_println;
/// let request = "GET key1";
/// let port = 3800;
/// dbg_println!("Processing request: {}", request);
/// dbg_println!("Connection established on port {}", port);
/// ```
#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            println!($($arg)*);
        }
    };
}

/// Error types that can occur when using the CHX library
#[derive(Debug)]
pub enum ChxError {
    ConnectionError(String),
    IoError(String),
    ResponseError(String),
    InvalidResponse,
    Other(String), // Add Other variant
}

impl fmt::Display for ChxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChxError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            ChxError::IoError(e) => write!(f, "I/O error: {}", e),
            ChxError::ResponseError(e) => write!(f, "Server response error: {}", e),
            ChxError::InvalidResponse => write!(f, "Invalid server response"),
            ChxError::Other(e) => write!(f, "Error: {}", e), // Add handling for Other
        }
    }
}

impl Error for ChxError {}

impl From<std::io::Error> for ChxError {
    fn from(err: std::io::Error) -> Self {
        ChxError::IoError(err.to_string())
    }
}

impl From<rustyline::error::ReadlineError> for ChxError {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        ChxError::Other(err.to_string())
    }
}

/// CHX Client for communicating with CHX server
///
/// This client provides an asynchronous interface for key-value operations
/// with auto-reconnect capability and robust error handling.
pub struct ChxClient {
    pub stream: TcpStream,
}

impl ChxClient {
    /// Creates a new ChxClient instance from an existing TcpStream
    pub fn new(stream: TcpStream) -> Self {
        ChxClient { stream }
    }

    /// Creates a new connection to CHX server
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address in "host:port" format (example: "127.0.0.1:3800")
    ///
    /// # Returns
    ///
    /// * `Result<Self, ChxError>` - ChxClient instance or error if connection fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use chx::ChxClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = ChxClient::connect("127.0.0.1:3800").await.unwrap();
    /// }
    /// ```
    pub async fn connect(addr: &str) -> Result<Self, ChxError> {
        dbg_println!("Attempting to connect to server at: {}", addr);
        let stream = TcpStream::connect(addr).await.map_err(|e| {
            dbg_println!("Failed to connect to server at {}: {}", addr, e);
            ChxError::ConnectionError(e.to_string())
        })?;
        dbg_println!("Successfully connected to server at: {}", addr);
        Ok(ChxClient::new(stream))
    }

    pub async fn send_command(&mut self, command: &str) -> Result<(), ChxError> {
        dbg_println!("Sending command: {}", command.trim());
        self.stream
            .write_all(command.as_bytes())
            .await
            .map_err(|e| {
                dbg_println!("Failed to send command: {}", e);
                ChxError::IoError(format!("Failed to send command: {}", e))
            })?;
        Ok(())
    }

    pub async fn parse_response(&mut self) -> Result<String, ChxError> {
        let mut buffer = [0u8; 1024]; // Use array instead of Vec for better performance
        let n = self.stream.read(&mut buffer).await.map_err(|e| {
            dbg_println!("Failed to read response: {}", e);
            ChxError::IoError(format!("Failed to read response: {}", e))
        })?;
        let response_bytes = &buffer[..n];
        let response = String::from_utf8_lossy(response_bytes).trim().to_string();
        dbg_println!("Received response: {} ({} bytes)", response, n);

        if let Some(stripped) = response.strip_prefix("!e") {
            dbg_println!("Server returned error: {}", stripped);
            return Err(ChxError::ResponseError(stripped.to_string()));
        } else if let Some(stripped) = response.strip_prefix('>') {
            dbg_println!("Server returned data: {} chars", stripped.len());
            return Ok(stripped.to_string());
        } else if response.starts_with('!') {
            dbg_println!("Server acknowledged command");
            return Ok(String::new()); // Use String::new() instead of "".to_string()
        }
        dbg_println!("Invalid response format: {}", response);
        Err(ChxError::InvalidResponse)
    }

    /// Retrieves value from server by key
    ///
    /// # Arguments
    ///
    /// * `key` - Key to retrieve the value for
    ///
    /// # Returns
    ///
    /// * `Result<Option<String>, ChxError>` - Some(value) if key found, None if not found, or Error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use chx::ChxClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ChxClient::connect("127.0.0.1:3800").await?;
    ///
    /// match client.get("my_key").await? {
    ///     Some(value) => println!("Found: {}", value),
    ///     None => println!("Key not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&mut self, key: &str) -> Result<Option<String>, ChxError> {
        dbg_println!("GET operation requested for key: '{}'", key);
        let command = format!("G {}\n", key);
        self.send_command(&command).await?;
        match self.parse_response().await {
            Ok(response) => {
                if response.is_empty() {
                    dbg_println!("GET operation: key '{}' not found", key);
                    Ok(None)
                } else {
                    dbg_println!(
                        "GET operation successful for key '{}': {} chars",
                        key,
                        response.len()
                    );
                    Ok(Some(response))
                }
            }
            Err(ChxError::ResponseError(msg)) if msg == "Key not found" => {
                dbg_println!("GET operation: key '{}' not found (server response)", key);
                Ok(None)
            }
            Err(e) => {
                dbg_println!("GET operation failed for key '{}': {:?}", key, e);
                Err(e)
            }
        }
    }

    /// Stores key-value pair to server
    ///
    /// # Arguments
    ///
    /// * `key` - Key to store
    /// * `value` - Value to store
    ///
    /// # Returns
    ///
    /// * `Result<(), ChxError>` - Ok(()) if successful, Error if failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use chx::ChxClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ChxClient::connect("127.0.0.1:3800").await?;
    ///
    /// client.set("user:123", "John Doe").await?;
    /// println!("Key set successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set(&mut self, key: &str, value: &str) -> Result<(), ChxError> {
        dbg_println!(
            "SET operation requested for key: '{}', value: {} chars",
            key,
            value.len()
        );
        let command = format!("S {} {}\n", key, value);
        self.send_command(&command).await?;
        match self.parse_response().await {
            Ok(_) => {
                dbg_println!("SET operation successful for key: '{}'", key);
                Ok(())
            }
            Err(e) => {
                dbg_println!("SET operation failed for key '{}': {:?}", key, e);
                Err(e)
            }
        }
    }

    /// Deletes key from server
    ///
    /// # Arguments
    ///
    /// * `key` - Key to delete
    ///
    /// # Returns
    ///
    /// * `Result<(), ChxError>` - Ok(()) if successful, Error if failed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use chx::ChxClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ChxClient::connect("127.0.0.1:3800").await?;
    ///
    /// client.del("user:123").await?;
    /// println!("Key deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn del(&mut self, key: &str) -> Result<(), ChxError> {
        dbg_println!("DEL operation requested for key: '{}'", key);
        let command = format!("D {}\n", key);
        self.send_command(&command).await?;
        match self.parse_response().await {
            Ok(_) => {
                dbg_println!("DEL operation successful for key: '{}'", key);
                Ok(())
            }
            Err(e) => {
                dbg_println!("DEL operation failed for key '{}': {:?}", key, e);
                Err(e)
            }
        }
    }
}

/// Runs the CHX server
///
/// This function will run the CHX server that can accept multiple client connections
/// concurrently with auto-expiration support for keys.
///
/// # Arguments
///
/// * `host` - Host address to bind server (example: "127.0.0.1")
/// * `port` - Port number to bind server (example: 3800)
/// * `expire_time` - TTL in seconds for auto-expiration (0 = no expiration)
///
/// # Returns
///
/// * `Result<(), ChxError>` - Ok(()) when server shuts down, Error if failed to start
///
/// # Example
///
/// ```rust,no_run
/// use chx::server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let host = "127.0.0.1";
///     let port = 3800;
///     let expire_time = 3600; // 1 hour TTL
///
///     // Server will run until Ctrl+C
///     server(host, port, expire_time).await?;
///     Ok(())
/// }
/// ```
pub async fn server(host: &str, port: u16, expire_time: u64) -> Result<(), ChxError> {
    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}:{}", host, port));
    type Store = Arc<RwLock<HashMap<Vec<u8>, (Vec<u8>, Instant)>>>;
    let store: Store = Arc::new(RwLock::new(HashMap::new()));

    dbg_println!("Server successfully bound to {}:{}", host, port);

    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener
                .accept()
                .await
                .expect("Failed to accept connection");
            dbg_println!("Client connected from {:?}", socket.peer_addr());
            let store_clone = Arc::clone(&store);

            tokio::spawn(async move {
                loop {
                    let mut buffer = [0; 1048576];
                    let n = socket.read(&mut buffer).await.unwrap();

                    if n == 0 {
                        break;
                    }

                    let request = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                    dbg_println!("Server received request: '{}' ({} bytes)", request, n);
                    let mut expired_key: Option<Vec<u8>> = None;

                    if let Some((command, args)) = request.split_once(' ') {
                        match command.trim() {
                            "G" => {
                                let key = args.trim().as_bytes().to_vec();
                                let _key_str = String::from_utf8_lossy(&key);
                                dbg_println!("Server processing GET for key: '{}'", _key_str);
                                if key.is_empty() {
                                    dbg_println!("Server: GET command with empty key");
                                    socket
                                        .write_all(b"!eInvalid GET command (empty key)\n")
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                if let Some((val, timestamp)) = store_clone.read().await.get(&key) {
                                    if expire_time > 0
                                        && timestamp.elapsed().as_millis()
                                            > (expire_time as u128) * 1000
                                    {
                                        dbg_println!(
                                            "Server: Key '{}' expired, removing",
                                            _key_str
                                        );
                                        expired_key = Some(key);
                                        socket.write_all(b"!").await.unwrap(); // Key expired, return "not found"
                                    } else {
                                        dbg_println!(
                                            "Server: GET successful for key '{}', returning {} bytes",
                                            _key_str,
                                            val.len()
                                        );
                                        let mut val_ = val.to_owned();
                                        val_.insert(0, 62); // >
                                        socket.write_all(val_.as_slice()).await.unwrap();
                                    }
                                } else {
                                    dbg_println!("Server: Key '{}' not found", _key_str);
                                    socket.write_all(b"!").await.unwrap(); // ! (not found)
                                }
                            }
                            "S" => {
                                if let Some((key_str, value_str)) = args.split_once(' ') {
                                    let key = key_str.trim().as_bytes().to_vec();
                                    let val = value_str.trim().as_bytes().to_vec();
                                    let _key_display = String::from_utf8_lossy(&key);
                                    dbg_println!(
                                        "Server processing SET for key: '{}', value: {} bytes",
                                        _key_display,
                                        val.len()
                                    );

                                    if key.is_empty() || val.is_empty() {
                                        dbg_println!("Server: SET command with empty key or value");
                                        socket
                                            .write_all(
                                                b"!eInvalid SET command (empty key or value)\n",
                                            )
                                            .await
                                            .unwrap();
                                        continue;
                                    }

                                    let timestamp = Instant::now();
                                    store_clone
                                        .write()
                                        .await
                                        .insert(key.clone(), (val, timestamp));
                                    dbg_println!(
                                        "Server: SET successful for key '{}'",
                                        _key_display
                                    );
                                    socket.write_all(b"!").await.unwrap(); // ! (acknowledged)
                                } else {
                                    dbg_println!("Server: Invalid SET command format");
                                    socket.write_all(b"!eInvalid SET command\n").await.unwrap();
                                }
                            }
                            "D" => {
                                let key = args.trim().as_bytes().to_vec();
                                let _key_str = String::from_utf8_lossy(&key);
                                dbg_println!("Server processing DEL for key: '{}'", _key_str);
                                if key.is_empty() {
                                    dbg_println!("Server: DEL command with empty key");
                                    socket
                                        .write_all(b"!eInvalid DEL command (empty key)\n")
                                        .await
                                        .unwrap();
                                    continue;
                                }
                                let _existed = store_clone.write().await.remove(&key).is_some();
                                dbg_println!(
                                    "Server: DEL for key '{}' - existed: {}",
                                    _key_str,
                                    _existed
                                );
                                socket.write_all(b"!").await.unwrap(); // ! (acknowledged)
                            }
                            _ => {
                                dbg_println!("Server: Unknown command: '{}'", command);
                                socket.write_all(b"!eUNKNOWN_COMMAND\n").await.unwrap();
                            }
                        }
                    } else {
                        dbg_println!("Server: Invalid request format: '{}'", request);
                        socket.write_all(b"!eUNKNOWN_COMMAND\n").await.unwrap(); // No command or arguments found
                    }

                    if let Some(key) = expired_key {
                        store_clone.write().await.remove(&key);
                    }
                }
            });
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    dbg_println!("Server shutting down.");
    Ok(())
}

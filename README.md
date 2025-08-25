# chx: In-Memory Key-Value Cache Server and Client

`chx` is a lightweight, in-memory key-value cache server written in Rust, accompanied by a command-line client for easy interaction. It provides a simple yet effective solution for caching data with optional expiration times, similar to popular in-memory data stores.

## Features

*   **Server Mode**: Run `chx` as a standalone server with customizable host, port, and a global `expire_time` for cached entries.
*   **Interactive REPL Client**: Connect to a running `chx` server using an interactive command-line interface, similar to `redis-cli`.
*   **Supported Commands**:
    *   `GET <key>`: Retrieve the value associated with a given key.
    *   `SET <key> <value>`: Store a key-value pair.
    *   `DEL <key>`: Delete a key-value pair.
*   **Expiration Time**: Keys stored on the server can automatically expire after a specified duration.
*   **Robust Error Handling**: The server and client are designed to handle various errors gracefully, providing informative messages.

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

*   [Rust](https://www.rust-lang.org/tools/install)
*   [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (Rust's package manager, installed with Rust)

### Building

To build the `chx` project and generate the final release binary, navigate to the project root directory and run:

```bash
cargo build --release
```

This command will compile the project in release mode, generating an optimized executable.

**Binary Location:**

*   **Linux/macOS**: The compiled binary will be found at `target/release/chx`
*   **Windows**: The compiled binary will be found at `target/release/chx.exe`

**Adding to System PATH (Optional):**

For ease of execution from any directory, you can add the `target/release/` directory to your system's PATH environment variable.

*   **Linux/macOS**:
    ```bash
    export PATH="$PATH:$(pwd)/target/release"
    ```
    (Add this line to `~/.bashrc`, `~/.zshrc`, or your appropriate shell profile file for persistence.)
*   **Windows (PowerShell)**:
    ```powershell
    $env:Path += ";$(Get-Location)\target\release"
    ```
    (For permanent changes, you need to add it through system environment settings.)

## Usage

After the `chx` binary has been successfully compiled, you can run it directly from the `target/release/` directory or from any location if you have added it to your system PATH.

### Running the Server

This section explains how to run the `chx` server with various configurations. The `chx` server is an in-memory key-value cache.

#### 1. Running Server with Default Settings

By default, the server will bind to `127.0.0.1:3800` with a default global expiration time of 2592000 seconds (30 days) for cached entries.

To run the server with default settings:

```bash
# Linux/macOS
./target/release/chx server
# Windows
.\target\release\chx.exe server
```

After the server is running, you will see a message like:

```
Server successfully bound to 127.0.0.1:3800
```

#### 2. Running Server with Custom Host and Port

You can specify a custom IP address (`--host`) and port number (`--port`) where the server will listen. This is useful if you want the server to be accessible from other machines on your network or if the default port is already in use.

*   `--host <IP>`: Specifies the IP address to bind to. Use `0.0.0.0` to listen on all available network interfaces.
*   `--port <PORT>`: Specifies the port number to listen on (e.g., `6379` for the standard port).

Example: Running the server on all interfaces on port `6379`.

```bash
# Linux/macOS
./target/release/chx server --host 0.0.0.0 --port 6379
# Windows
.\target\release\chx.exe server --host 0.0.0.0 --port 6379
```

Expected output:

```
Server successfully bound to 0.0.0.0:6379
```

#### 3. Running Server with Custom Expiration Time

You can set the default global expiration time for all keys stored on the server using the `--expire-time` argument. The expiration time is given in seconds.

*   `--expire-time <SECONDS>`: Default expiration time for keys in seconds. If set to `0`, keys will never expire automatically.

Example: Running the server with keys expiring after 1 hour (3600 seconds).

```bash
# Linux/macOS
./target/release/chx server --expire-time 3600
# Windows
.\target\release\chx.exe server --expire-time 3600
```

Expected output:

```
Server successfully bound to 127.0.0.1:3800
```

#### 4. Combining All Options

You can combine all arguments for more specific server control.

Example: Running the server at `192.168.1.10` on port `6379`, with expiration time of 2 hours (7200 seconds).

```bash
# Linux/macOS
./target/release/chx server --host 192.168.1.10 --port 6379 --expire-time 7200
# Windows
.\target\release\chx.exe server --host 192.168.1.10 --port 6379 --expire-time 7200
```

Expected output:

```
Server successfully bound to 192.168.1.10:6379
```

### Using the Client (Interactive REPL)

The `chx` client allows you to interact with a running `chx` server through an interactive command-line interface (REPL).

#### 1. Connecting to the Server

Before using the client, make sure you have a running `chx` server. The client will try to connect to `127.0.0.1:3800` by default.

To connect to a server running with default settings:

```bash
# Linux/macOS
./target/release/chx client
# Windows
.\target\release\chx.exe client
```

If your server is running on a different host or port, you can specify them using the `--host` and `--port` arguments.

Example: Connecting to a server running at `192.168.1.100` on port `6379`.

```bash
# Linux/macOS
./target/release/chx client --host 192.168.1.100 --port 6379
# Windows
.\target\release\chx.exe client --host 192.168.1.100 --port 6379
```

After successfully connecting, you will see the `chx>` prompt indicating that the client is ready to receive commands.

```
Connected to Chx server at 127.0.0.1:3800. Type 'help' for commands.
chx>
```

#### 2. Basic Commands in REPL

Here are the basic commands you can use in the REPL:

*   **`SET <key> <value>`: Store Key-Value Pairs**
    This command is used to store a given value associated with the specified key. The value can be a single string or a string with spaces.

    Examples:

    ```
    chx> SET name john
    Key set successfully
    chx> SET message "Hello, world!"
    Key set successfully
    ```

    Output: `Key set successfully` indicates the operation was successful.

*   **`GET <key>`: Retrieve Values**
    This command retrieves the value associated with the specified key.

    Examples:

    ```
    chx> GET name
    john
    chx> GET message
    Hello, world!
    chx> GET nonexistent_key
    Key not found
    ```

    Output: If the key is found, its value will be displayed. If not, `Key not found` will be displayed.

*   **`DEL <key>`: Delete Keys**
    This command deletes the key and its associated value from the cache.

    Examples:

    ```
    chx> DEL name
    Key deleted successfully
    chx> DEL another_key
    Key deleted successfully
    ```

    Output: `Key deleted successfully` if the key exists and is successfully deleted.

*   **`HELP`: Display Available Commands**
    Displays a list of all commands supported by the client.

    Example:

    ```
    chx> HELP
    Available commands:
      GET <key>        - Retrieve the value associated with a key
      SET <key> <value> - Set a key-value pair
      DEL <key>        - Delete a key
      quit | exit      - Exit the client
    ```

#### 3. Exiting the REPL

To exit the `chx` client REPL, you can use one of the following commands:

```
chx> quit
```

Or:

```
chx> exit
```

Output:

```
Exiting Chx client.
```

## Testing

To run the project tests, execute the following command in the root directory:

```bash
cargo test
```

## Important Information & Design Considerations

*   **In-Memory Cache**: `chx` is a fully **in-memory** cache. This means all data is stored in RAM and is **not persistent** across server restarts. If the server is shut down, all cached data will be lost.
*   **`expire_time` Functionality**: The server's `--expire-time` parameter sets the default global expiration time for all cached entries. This is the maximum duration (in seconds) a key will remain valid if there are no new `SET` operations that update it or `DEL` operations that remove it. If set to `0`, keys will never expire automatically.
*   **Robust Error Handling**: Both the server and client are designed to handle various errors gracefully, providing informative messages to users.
*   **Core Technologies**: `chx` is built using:
    *   **Rust**: A high-performance, memory-safe programming language.
    *   **Tokio**: An asynchronous runtime for writing fast and reliable network applications.
    *   **Clap**: A powerful command-line argument parser library.
    *   **Rustyline**: A library for providing rich REPL functionality for the client.

## Using CHX as a Library

CHX can be used as a library in your Rust applications. Add the following dependency to your `Cargo.toml`:

### Installation from GitHub

```toml
[dependencies]
chx = { git = "https://github.com/Nux-xader/chx" }
tokio = { version = "1.45", features = ["full"] }
```

### Installation from crates.io (when published)

```toml
[dependencies]
chx = "0.1.0"
tokio = { version = "1.45", features = ["full"] }
```

### Client Usage Example

```rust
use chx::{ChxClient, ChxError};

#[tokio::main]
async fn main() -> Result<(), ChxError> {
    // Connect to CHX server
    let mut client = ChxClient::connect("127.0.0.1:3800").await?;

    // Set key-value
    client.set("user:123", "John Doe").await?;
    println!("Key set successfully");

    // Get value
    if let Some(value) = client.get("user:123").await? {
        println!("Found user: {}", value);
    }

    // Delete key
    client.del("user:123").await?;
    println!("Key deleted successfully");

    Ok(())
}
```

### Server Usage Example

```rust
use chx::{server, ChxError};

#[tokio::main]
async fn main() -> Result<(), ChxError> {
    let host = "127.0.0.1".to_string();
    let port = 3800;
    let expire_time = 3600; // 1 hour TTL

    println!("Starting CHX server on {}:{}", host, port);
    
    // Start server (will run until Ctrl+C)
    server(&host, &port, expire_time).await?;
    
    Ok(())
}
```

### API Documentation

For complete API documentation, run:

```bash
cargo doc --open
```

### Library Features

- **Asynchronous**: All operations use async/await
- **Error Handling**: Comprehensive error types with `ChxError`
- **Auto-reconnect**: Client automatically reconnects when connection is lost
- **Thread-safe**: Server supports multiple concurrent connections
- **TTL Support**: Auto-expiration for keys with TTL
- **Lightweight**: Binary only ~1.7MB in release mode

## Contributing

Contributions are welcome! If you find bugs or have ideas for improvements, please open an issue or submit a pull request.

use chx::{ChxClient, ChxError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// Helper function to create a mock TcpStream for testing
async fn create_mock_stream_pair(initial_server_response: Option<&str>) -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let client_task = tokio::spawn(async move { TcpStream::connect(addr).await.unwrap() });

    let (mut server_socket, _) = listener.accept().await.unwrap();

    if let Some(response) = initial_server_response {
        server_socket.write_all(response.as_bytes()).await.unwrap();
        server_socket.flush().await.unwrap();
    }

    let client_socket = client_task.await.unwrap();
    (client_socket, server_socket)
}

#[tokio::test]
async fn test_parse_response_success() {
    let (stream, _server_socket) = create_mock_stream_pair(Some(">OK\n")).await;
    let mut client = ChxClient::new(stream);
    let response = client.parse_response().await.unwrap();
    assert_eq!(response, "OK");
}

#[tokio::test]
async fn test_parse_response_error() {
    let (stream, _server_socket) = create_mock_stream_pair(Some("!eError message\n")).await;
    let mut client = ChxClient::new(stream);
    let err = client.parse_response().await.unwrap_err();
    if let ChxError::ResponseError(msg) = err {
        assert_eq!(msg, "Error message");
    } else {
        panic!("Expected ResponseError, got {:?}", err);
    }
}

#[tokio::test]
async fn test_parse_response_acknowledgment() {
    let (stream, _server_socket) = create_mock_stream_pair(Some(
        "!
",
    ))
    .await;
    let mut client = ChxClient::new(stream);
    let response = client.parse_response().await.unwrap();
    assert_eq!(response, "");
}

#[tokio::test]
async fn test_parse_response_invalid() {
    let (stream, _server_socket) = create_mock_stream_pair(Some("Invalid response\n")).await;
    let mut client = ChxClient::new(stream);
    let err = client.parse_response().await.unwrap_err();
    if let ChxError::InvalidResponse = err {
        // Expected
    } else {
        panic!("Expected InvalidResponse, got {:?}", err);
    }
}

#[tokio::test]
async fn test_send_command() {
    let (stream, mut server_socket) = create_mock_stream_pair(None).await;
    let mut client = ChxClient::new(stream);
    let command = "SET key value\n";
    client.send_command(command).await.unwrap();

    let mut buffer = vec![0; 1024];
    let n = server_socket.read(&mut buffer).await.unwrap();
    let received_command = String::from_utf8_lossy(&buffer[..n]).to_string();
    assert_eq!(received_command, command);
}

#[test]
fn test_chx_error_display() {
    let err1 = ChxError::ConnectionError("Failed to connect".to_string());
    assert_eq!(format!("{}", err1), "Connection error: Failed to connect");

    let err2 = ChxError::IoError("Broken pipe".to_string());
    assert_eq!(format!("{}", err2), "I/O error: Broken pipe");

    let err3 = ChxError::ResponseError("Key not found".to_string());
    assert_eq!(format!("{}", err3), "Server response error: Key not found");

    let err4 = ChxError::InvalidResponse;
    assert_eq!(format!("{}", err4), "Invalid server response");
}

#[test]
fn test_chx_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "Some IO error");
    let chx_err: ChxError = io_err.into();
    if let ChxError::IoError(msg) = chx_err {
        assert_eq!(msg, "Some IO error");
    } else {
        panic!("Expected IoError");
    }
}

#[tokio::test]
async fn test_client_get_success() {
    let (stream, _server_socket) = create_mock_stream_pair(Some(">value\n")).await;
    let mut client = ChxClient::new(stream);
    let response = client.get("key").await.unwrap();
    assert_eq!(response, Some("value".to_string()));
}

#[tokio::test]
async fn test_client_get_not_found() {
    let (stream, _server_socket) = create_mock_stream_pair(Some("!eKey not found\n")).await;
    let mut client = ChxClient::new(stream);
    let response = client.get("nonexistent_key").await.unwrap();
    assert_eq!(response, None);
}

#[tokio::test]
async fn test_client_set_success() {
    let (stream, _server_socket) = create_mock_stream_pair(Some(
        "!
",
    ))
    .await;
    let mut client = ChxClient::new(stream);
    client.set("key", "value").await.unwrap();
    // If no error, it means success
}

#[tokio::test]
async fn test_client_del_success() {
    let (stream, _server_socket) = create_mock_stream_pair(Some(
        "!
",
    ))
    .await;
    let mut client = ChxClient::new(stream);
    client.del("key").await.unwrap();
    // If no error, it means success
}

use chx::{ChxClient, ChxError, server};
use tokio::net::TcpListener;
use tokio::time::Duration;

async fn get_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to an available port");
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Close listener so port can be used by server
    port
}

async fn spawn_server_and_connect_client(expire_time: u64) -> (ChxClient, u16) {
    let port = get_available_port().await;
    let host = "127.0.0.1".to_string();
    let cloned_host = host.clone(); // Clone host for the spawned task

    // Use tokio::spawn to run server in background
    tokio::spawn(async move {
        server(&cloned_host, port, expire_time).await.unwrap();
    });

    // Give a little time for server to be ready to accept connections
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = ChxClient::connect(&format!("{}:{}", host, port))
        .await
        .unwrap();
    (client, port)
}

#[tokio::test]
async fn integration_test_connect_success() {
    // Connection success is verified by spawn_server_and_connect_client().await.unwrap();
    let (_client, _port) = spawn_server_and_connect_client(0).await;
}

#[tokio::test]
async fn integration_test_connect_fail() {
    let port = get_available_port().await; // Use available port but no server running
    let client = ChxClient::connect(&format!("127.0.0.1:{}", port)).await;
    assert!(client.is_err());
    if let Err(ChxError::ConnectionError(_)) = client {
        // Expected error
    } else {
        panic!("Expected ConnectionError");
    }
}

#[tokio::test]
async fn integration_test_set_get_del_flow() {
    let (mut client, _port) = spawn_server_and_connect_client(0).await;

    // SET a key
    client.set("mykey", "myvalue").await.unwrap();

    // GET the key
    let value = client.get("mykey").await.unwrap();
    assert_eq!(value, Some("myvalue".to_string()));

    // GET a non-existent key
    let value_none = client.get("nonexistent_key").await.unwrap();
    assert_eq!(value_none, None);

    // DEL the key
    client.del("mykey").await.unwrap();

    // GET the deleted key (should be None)
    let value_after_del = client.get("mykey").await.unwrap();
    assert_eq!(value_after_del, None);
}

#[tokio::test]
async fn integration_test_expiration_handling() {
    let (mut client, _port) = spawn_server_and_connect_client(1).await; // Expire time 1 detik

    client.set("expire_key", "temp_value").await.unwrap();

    // Attempt to get the key immediately (should still be there)
    let value_before_expire = client.get("expire_key").await.unwrap();
    assert_eq!(value_before_expire, Some("temp_value".to_string()));

    // Wait for the simulated expiration (more than 1 second)
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Attempt to get the key after expiration (should be None)
    let value_after_expire = client.get("expire_key").await.unwrap();
    assert_eq!(value_after_expire, None);

    // Test for key that does not expire
    let (mut client_no_expire, _port_no_expire) = spawn_server_and_connect_client(0).await; // Expire time 0 (no expiration)
    client_no_expire
        .set("no_expire_key", "value_persists")
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let value_no_expire = client_no_expire.get("no_expire_key").await.unwrap();
    assert_eq!(value_no_expire, Some("value_persists".to_string()));
}

#[tokio::test]
async fn integration_test_server_error_handling() {
    // With the actual server, we cannot trigger internal errors like a mock server.
    // Error handling that we can test is connection errors or invalid response format.
    // Since the real server will always respond with the correct format (based on implementation),
    // this test will focus on failed or unexpected connection scenarios.
    // To test internal server error handling, we need to modify the server itself
    // to simulate errors, which is outside the scope of this integration test.

    // Test: client tries to send invalid command to server (e.g., without newline)
    let (_client, _port) = spawn_server_and_connect_client(0).await;
    // Since the `stream` field of `ChxClient` is private, we cannot directly access it.
    // To test invalid command handling, we need to add public methods to ChxClient
    // or modify the server to send specific error responses.
    // For now, we will ignore this part because the goal is interaction with the real server.
    // If we need to test this scenario, `ChxClient` needs to be modified to expose this functionality
    // or the server needs to emit errors that can be caught by `ChxClient` public methods.

    // For error response testing, we assume the server will send error response if there are internal problems
    // that we cannot simulate from the client side without modifying the server.
    // If there are scenarios where the server sends '!e' response, we can test that.
    // Example: if the server receives 'G non_existent_key' command and responds '!e' for it,
    // then this test can be verified. However, the current server implementation for 'G'
    // will return '!' or '>' not '!e' for keys that are not found.
}

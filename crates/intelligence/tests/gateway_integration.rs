use common::Error;
use intelligence::gateway::{AnthropicGateway, LlmGateway};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn start_mock_server(response_status: u16) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        let mut buffer = [0; 1024];
        let _ = socket.read(&mut buffer).await.unwrap();

        // We could inspect the request here if needed
        // let request = String::from_utf8_lossy(&buffer);

        let response = if response_status == 200 {
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json\r\n\
             \r\n\
             {\"id\":\"msg_123\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"Hello\"}]}"
        } else if response_status == 401 {
             "HTTP/1.1 401 Unauthorized\r\n\
             Content-Type: application/json\r\n\
             \r\n\
             {\"error\":{\"type\":\"authentication_error\",\"message\":\"invalid api key\"}}"
        } else {
             "HTTP/1.1 500 Internal Server Error\r\n\r\n"
        };

        socket.write_all(response.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
    });

    (url, handle)
}

#[tokio::test]
async fn test_anthropic_gateway_initialize_success() {
    let (base_url, _handle) = start_mock_server(200).await;

    let client = reqwest::Client::new();
    let mut gateway = AnthropicGateway::new(
        "test-key".to_string(),
        "claude-3".to_string(),
        client
    ).with_base_url(base_url);

    let result = gateway.initialize().await;
    assert!(result.is_ok(), "Initialization should succeed with 200 response");
}

#[tokio::test]
async fn test_anthropic_gateway_initialize_failure() {
    let (base_url, _handle) = start_mock_server(401).await;

    let client = reqwest::Client::new();
    let mut gateway = AnthropicGateway::new(
        "test-key".to_string(),
        "claude-3".to_string(),
        client
    ).with_base_url(base_url);

    let result = gateway.initialize().await;

    match result {
        Err(Error::Config(msg)) => {
            assert!(msg.contains("Invalid API Key"), "Should return config error for 401");
        }
        _ => panic!("Expected Error::Config, got {:?}", result),
    }
}

use common::Result;
use intelligence::gateway::{OllamaGateway, LlmGateway};
use reqwest::Client;

#[tokio::test]
#[ignore]
async fn test_ollama_initialization() -> Result<()> {
    let client = Client::new();
    let mut gateway = OllamaGateway::new("llama2".to_string(), client);

    // This expects Ollama to be running on localhost:11434/api/tags
    gateway.initialize().await?;

    assert!(gateway.health_check().await?);

    Ok(())
}

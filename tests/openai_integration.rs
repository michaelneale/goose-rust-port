use std::env;
use anyhow::Result;
use rust_goose::exchange::{OpenAIOptions, OpenAIProvider, Message, Provider};

// Helper function to check if we have a valid OpenAI API key
fn has_valid_api_key() -> bool {
    match env::var("OPENAI_API_KEY") {
        Ok(key) => !key.is_empty() && key != "invalid_key",
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_openai_conversation() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Skip test if no valid API key
    if !has_valid_api_key() {
        println!("Skipping test_openai_conversation - no valid API key found");
        return Ok(());
    }

    // Create provider with test options
    let options = OpenAIOptions {
        model: "gpt-4".to_string(),
        temperature: 0.7,
        max_tokens: 2048,
        system_prompt: Some("You are a helpful assistant.".to_string()),
    };
    let provider = OpenAIProvider::new(Some(options)).unwrap();
    
    // Test a simple conversation
    let messages = vec![Message::user("Hello!")];
    
    let response = provider.generate(&messages, None).await?;
    assert!(!response.text().is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_openai_error_handling() -> Result<()> {
    // Save original API key if it exists
    let original_key = env::var("OPENAI_API_KEY").ok();
    
    // Set invalid API key
    env::set_var("OPENAI_API_KEY", "invalid_key");
    
    let provider = OpenAIProvider::new(None)?;
    let messages = vec![Message::user("Hello")];
    
    let result = provider.generate(&messages, None).await;
    assert!(result.is_err());

    // Restore original API key if it existed
    if let Some(key) = original_key {
        env::set_var("OPENAI_API_KEY", key);
    } else {
        env::remove_var("OPENAI_API_KEY");
    }
    
    Ok(())
}
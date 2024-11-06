use rust_goose::cli::session::Session;
use anyhow::Result;

#[tokio::test]
async fn test_session_stats_integration() -> Result<()> {
    // Create a new session
    let mut session = Session::new(
        Some("test_session".to_string()),
        None,  // default profile
        None,  // no plan
        Some("INFO".to_string()),
        false, // no tracing
    ).await?;
    
    // Get initial stats
    let initial_stats = session.get_stats();
    assert_eq!(initial_stats.total_messages, 0);
    assert_eq!(initial_stats.total_tokens, 0);
    
    // Process a test message
    let message = rust_goose::models::Message::user("test message");
    session.process_message(message).await?;
    
    // Check updated stats
    let updated_stats = session.get_stats();
    assert_eq!(updated_stats.total_messages, 1);
    assert!(updated_stats.total_tokens > 0);
    
    Ok(())
}
use rust_goose::stats::{SessionStats, StatsTracker};
use rust_goose::session::SessionLoop;
use anyhow::Result;

#[tokio::test]
async fn test_session_stats_integration() -> Result<()> {
    // Create a new session
    let mut session = SessionLoop::new("test_session".to_string(), None);
    
    // Get initial stats
    let initial_stats = session.get_stats();
    assert_eq!(initial_stats.total_messages, 0);
    assert_eq!(initial_stats.total_tokens, 0);
    
    // Process a test message
    let message = rust_goose::exchange::Message::user("test message");
    session.process_message(message)?;
    
    // Check updated stats
    let updated_stats = session.get_stats();
    assert_eq!(updated_stats.total_messages, 1);
    
    // Test total stats
    let total_stats = session.get_total_stats()?;
    assert_eq!(total_stats.total_messages, 1);
    
    Ok(())
}
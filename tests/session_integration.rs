use std::sync::Once;
use anyhow::Result;
use rust_goose::cli::session::Session;
use rust_goose::exchange::Message;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        // Load environment variables from .env file if present
        dotenv::dotenv().ok();
    });
}

#[tokio::test]
async fn test_session_start_basic() -> Result<()> {
    setup();

    // Create a new session with default settings
    let mut session = Session::new(
        Some("test_session".to_string()),
        None,  // default profile
        None,  // no plan
        Some("INFO".to_string()),
        false, // no tracing
    ).await?;

    // Mock user input for testing
    let test_message = Message::user("Hello!");
    session.process_message(test_message).await?;

    // Verify session stats
    let stats = session.get_stats();
    assert_eq!(stats.total_messages, 1);
    assert!(stats.total_tokens > 0);

    Ok(())
}

#[tokio::test]
async fn test_session_start_with_profile() -> Result<()> {
    setup();

    // Create a new session with a specific profile
    let mut session = Session::new(
        Some("test_session_profile".to_string()),
        Some("default".to_string()),
        None,
        Some("INFO".to_string()),
        false,
    ).await?;

    // Verify profile was loaded
    assert_eq!(session.profile_name.as_deref().unwrap_or(""), "default");

    Ok(())
}

#[tokio::test]
async fn test_session_interruption() -> Result<()> {
    setup();

    let mut session = Session::new(
        Some("test_session_interrupt".to_string()),
        None,
        None,
        Some("INFO".to_string()),
        false,
    ).await?;

    // Simulate an interruption
    session.interrupt();

    // Verify session is interrupted
    assert!(session.is_interrupted());

    // Try to process a message after interruption
    let message = Message::user("This should be interrupted");
    session.process_message(message).await?;

    // Verify interruption was handled and cleared
    assert!(!session.is_interrupted());

    Ok(())
}
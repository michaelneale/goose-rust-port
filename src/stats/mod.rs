use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_messages: u32,
    pub total_tokens: u32,
    pub total_cost: f64,
}

impl SessionStats {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            start_time: Utc::now(),
            end_time: None,
            total_messages: 0,
            total_tokens: 0,
            total_cost: 0.0,
        }
    }

    pub fn duration(&self) -> Duration {
        let end = self.end_time.unwrap_or_else(Utc::now);
        end.signed_duration_since(self.start_time)
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(0))
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Utc::now());
    }

    pub fn add_message(&mut self) {
        self.total_messages += 1;
    }

    pub fn add_tokens(&mut self, tokens: u32) {
        self.total_tokens += tokens;
        // Update cost based on token usage
        // TODO: Implement proper cost calculation based on model
        self.total_cost += (tokens as f64) * 0.0001;
    }

    pub fn summary(&self) -> String {
        format!(
            "Session {} stats:\n\
             Duration: {:?}\n\
             Messages: {}\n\
             Tokens: {}\n\
             Estimated cost: ${:.4}",
            self.session_id,
            self.duration(),
            self.total_messages,
            self.total_tokens,
            self.total_cost
        )
    }
}

#[derive(Default)]
pub struct StatsTracker {
    stats: Vec<SessionStats>,
}

impl StatsTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_session(&mut self, stats: SessionStats) {
        self.stats.push(stats);
    }

    pub fn get_session_stats(&self, session_id: &str) -> Option<&SessionStats> {
        self.stats.iter().find(|s| s.session_id == session_id)
    }

    pub fn get_total_stats(&self) -> SessionStats {
        let mut total = SessionStats::new("total".to_string());
        for stats in &self.stats {
            total.total_messages += stats.total_messages;
            total.total_tokens += stats.total_tokens;
            total.total_cost += stats.total_cost;
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_session_stats() {
        let mut stats = SessionStats::new("test".to_string());
        
        // Add some activity
        stats.add_message();
        stats.add_tokens(100);
        
        // Simulate some time passing
        thread::sleep(Duration::from_millis(100));
        
        // Complete the session
        stats.complete();
        
        assert_eq!(stats.total_messages, 1);
        assert_eq!(stats.total_tokens, 100);
        assert!(stats.duration().as_millis() >= 100);
    }

    #[test]
    fn test_stats_tracker() {
        let mut tracker = StatsTracker::new();
        
        let mut stats1 = SessionStats::new("session1".to_string());
        stats1.add_tokens(100);
        stats1.complete();
        
        let mut stats2 = SessionStats::new("session2".to_string());
        stats2.add_tokens(200);
        stats2.complete();
        
        tracker.track_session(stats1);
        tracker.track_session(stats2);
        
        let total = tracker.get_total_stats();
        assert_eq!(total.total_tokens, 300);
    }
}
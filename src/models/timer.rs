use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum TimerState {
    Working(DateTime<Utc>),
    ShortBreak(Duration),
    LongBreak(Duration),
    Paused(Duration, Box<TimerState>),
    Idle,
}

pub struct PomodoroConfig {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub sessions_before_long_break: u32,
}

impl PomodoroConfig {
    // Constructor with custom values
    pub fn new(work_mins: u64, short_break_mins: u64, long_break_mins: u64, sessions: u32) -> Self {
        Self {
            work_duration: Duration::from_secs(work_mins * 60),
            short_break_duration: Duration::from_secs(short_break_mins * 60),
            long_break_duration: Duration::from_secs(long_break_mins * 60),
            sessions_before_long_break: sessions,
        }
    }

    // Validator
    pub fn is_valid(&self) -> bool {
        self.work_duration.as_secs() > 0
            && self.short_break_duration.as_secs() > 0
            && self.long_break_duration.as_secs() > 0
            && self.sessions_before_long_break > 0
    }
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_before_long_break: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimerUpdate {
    pub state: TimerState,
    pub remaining: Duration,
    pub completed_sessions: u32,
    pub total_sessions: u32,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer}; // <-- Important
use std::time::Duration;

pub fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?; // uses u64::deserialize
    Ok(Duration::from_secs(secs))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TimerState {
    // This variant has a DateTime<Utc>, which chrono can handle if "serde" is enabled
    Working(DateTime<Utc>),

    // For Duration fields, we attach our custom (de)serialize functions
    ShortBreak(
        #[serde(
            serialize_with = "serialize_duration",
            deserialize_with = "deserialize_duration"
        )]
        Duration,
    ),
    LongBreak(
        #[serde(
            serialize_with = "serialize_duration",
            deserialize_with = "deserialize_duration"
        )]
        Duration,
    ),

    // Paused has a Duration and another TimerState
    Paused(
        #[serde(
            serialize_with = "serialize_duration",
            deserialize_with = "deserialize_duration"
        )]
        Duration,
        Box<TimerState>,
    ),

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerUpdate {
    pub state: TimerState,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub remaining: Duration,
    pub completed_sessions: u32,
    pub total_sessions: u32,
}

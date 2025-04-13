use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use {
    crate::models::timer::{PomodoroConfig, TimerState, TimerUpdate},
    crate::services::pomodoro::PomodoroTimer,
};

#[tokio::main]
async fn main() {
    // Create a PomodoroConfig
    let config = PomodoroConfig {
        work_duration: Duration::new(1500, 0),       // 25 minutes
        short_break_duration: Duration::new(300, 0), // 5 minutes
        long_break_duration: Duration::new(900, 0),  // 15 minutes
        sessions_before_long_break: 1,
    };

    // Create a PomodoroTimer
    let timer = PomodoroTimer::new(config);

    // Subscribe to the timer updates
    let mut rx: Receiver<TimerUpdate> = timer.subscribe().await;

 

    // Print the timer updates
    while let Ok(update) = rx.recv().await {
        println!("{:?}", update);
    }
}

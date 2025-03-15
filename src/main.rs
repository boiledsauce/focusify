use focusify::{
    models::timer::{PomodoroConfig, TimerState, TimerUpdate},
    services::pomodoro::PomodoroTimer,
};
use std::time::Duration;
use tokio::sync::broadcast::Receiver;

#[tokio::main]
async fn main() {
    // Create a PomodoroConfig
    let config = PomodoroConfig {
        work_duration: Duration::new(1500, 0),       // 25 minutes
        short_break_duration: Duration::new(300, 0), // 5 minutes
        long_break_duration: Duration::new(900, 0),  // 15 minutes
        sessions_before_long_break: 4,
    };

    // Create a PomodoroTimer
    let timer = PomodoroTimer::new(config);

    // Subscribe to the timer updates
    let mut rx: Receiver<TimerUpdate> = timer.subscribe().await;

    // Start the timer
    tokio::spawn(async move {
        timer.start().await;
    });

    // Print the timer updates
    while let Ok(update) = rx.recv().await {
        println!("{:?}", update);
    }
}

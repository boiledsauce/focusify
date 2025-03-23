use crate::models::timer::{PomodoroConfig, TimerState, TimerUpdate};
use chrono::Utc;
use std::time::Duration;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::Mutex;

pub struct PomodoroTimer {
    state: Mutex<TimerState>,
    config: PomodoroConfig,
    remaining: Mutex<Duration>,
    completed_sessions: Mutex<u32>,
    tx: Sender<TimerUpdate>,
}

impl PomodoroTimer {
    pub fn new(cfg: PomodoroConfig) -> PomodoroTimer {
        let work_duration = cfg.work_duration; // Copy the value before cfg is moved

        PomodoroTimer {
            config: cfg,
            state: Mutex::new(TimerState::Working(Utc::now())),
            remaining: Mutex::new(work_duration),
            completed_sessions: Mutex::new(0),
            tx: channel(16).0,
        }
    }

    pub async fn subscribe(&self) -> Receiver<TimerUpdate> {
        self.tx.subscribe()
    }

    // Set remaining time to beging
    pub async fn run_timer_loop(&self) {
        loop {
            let state_snapshot: TimerState = self.state.lock().await.clone();

            if !state_snapshot.is_running() {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            self.send_timer_message().await;
            {
                let mut remaining_time = self.remaining.lock().await;

                // Handle state transition if time is up
                if *remaining_time == Duration::from_secs(0) {
                    let mut state = self.state.lock().await;
                    self.handle_state_transition(&mut state).await;
                    drop(state);
                }

                *remaining_time = remaining_time.saturating_sub(Duration::from_secs(1));
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn start(&self) {
        let mut state: tokio::sync::MutexGuard<'_, TimerState> = self.state.lock().await;
        *state = TimerState::Working(Utc::now());
    }

    pub async fn stop(&self) {
        let mut state: tokio::sync::MutexGuard<'_, TimerState> = self.state.lock().await;
        *state = TimerState::Paused;
    }

    async fn send_timer_message(&self) {
        // Don't unwrap, just log or ignore errors
        let result = self.tx.send(TimerUpdate {
            state: self.state.lock().await.clone(),
            remaining: *self.remaining.lock().await,
            completed_sessions: *self.completed_sessions.lock().await,
            total_sessions: self.config.sessions_before_long_break,
        });

        // Handle the error gracefully
        if let Err(e) = result {
            println!("Failed to send timer update: no active receivers");
        }
    }

    async fn handle_state_transition(&self, state: &mut TimerState) {
        // Increase completed sessions
        let mut completed_sessions = self.completed_sessions.lock().await;
        *completed_sessions += 1;

        match state {
            TimerState::Working(_) => {
                *state = TimerState::ShortBreak(self.config.short_break_duration);
                *self.remaining.lock().await = self.config.short_break_duration;
            }
            TimerState::ShortBreak(_) => {
                *state = TimerState::Working(Utc::now());
                *self.remaining.lock().await = self.config.work_duration;
            }
            TimerState::LongBreak(_) => {
                *state = TimerState::Working(Utc::now());
                *self.remaining.lock().await = self.config.work_duration;
            }

            // These are the only states that the timer can be in
            _ => {}
        }
    }
}

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
    state_change_notify: tokio::sync::Notify,
    tx: Sender<TimerUpdate>,
    previous_state: Mutex<Option<TimerState>>, // Add this field to remember previous state
}

impl PomodoroTimer {
    pub fn new(cfg: PomodoroConfig) -> PomodoroTimer {
        let work_duration = cfg.work_duration; // Copy the value before cfg is moved

        PomodoroTimer {
            config: cfg,
            state: Mutex::new(TimerState::Paused),
            remaining: Mutex::new(work_duration),
            completed_sessions: Mutex::new(0),
            state_change_notify: tokio::sync::Notify::new(),
            tx: channel(16).0,
            previous_state: Mutex::new(None), // Initialize as None
        }
    }

    pub async fn subscribe(&self) -> Receiver<TimerUpdate> {
        self.tx.subscribe()
    }

    // Set remaining time to beging
    pub async fn run_timer_loop(&self) {
        loop {
            // Check if we need to pause
            let is_running = {
                let state = self.state.lock().await;
                state.is_running()
            };
    
            if !is_running {
                // Wait for notification instead of sleeping
                tokio::select! {
                    _ = self.state_change_notify.notified() => {
                        // State changed, continue loop immediately
                        continue;
                    }
                }
            }
    
            // Send current state, regardless of whether we're about to change it
            self.send_timer_message().await;
    
            // Check if timer is zero (without holding the lock for long)
            let is_time_up = {
                let remaining = self.remaining.lock().await;
                *remaining == Duration::from_secs(0)
            };
    
            // Handle state transition if needed
            if is_time_up {
                // Handle the transition independently
                let mut state = self.state.lock().await;
                self.handle_state_transition(&mut state).await;
            } else {
                // Decrement time only if not transitioning
                let mut remaining = self.remaining.lock().await;
                *remaining = remaining.saturating_sub(Duration::from_secs(1));
            }
    
            // Wait for next tick
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    // Notify in start method
    pub async fn start(&self) {
        {
            let mut state = self.state.lock().await;
            let mut prev_state = self.previous_state.lock().await;
            
            // If we have a stored previous state that wasn't Paused, restore it
            if let Some(previous) = prev_state.take() {
                match previous {
                    TimerState::ShortBreak(duration) => {
                        *state = TimerState::ShortBreak(duration);
                    },
                    TimerState::LongBreak(duration) => {
                        *state = TimerState::LongBreak(duration);
                    },
                    _ => {
                        // Default to Working if previous state wasn't a break
                        *state = TimerState::Working(Utc::now());
                    }
                }
            } else {
                // No previous state, start a working session
                *state = TimerState::Working(Utc::now());
            }
        }
        self.state_change_notify.notify_one(); // Wake up timer loop
        self.send_timer_message().await;
    }

    pub async fn stop(&self) {
        {
            let state = self.state.lock().await;
            let mut prev_state = self.previous_state.lock().await;
            
            // Remember current state before pausing (if it's not already Paused)
            if !matches!(*state, TimerState::Paused) {
                *prev_state = Some(state.clone());
            }
        }
        {
            let mut state = self.state.lock().await;
            *state = TimerState::Paused;
        }
        self.state_change_notify.notify_one(); // Wake up timer loop
        self.send_timer_message().await;
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
        // Determine next state without holding other locks
        let next_state_info = match state {
            TimerState::Working(_) => {
                // Update completed_sessions and get new value
                let updated_sessions = {
                    let mut sessions = self.completed_sessions.lock().await;
                    *sessions += 1;
                    *sessions
                };
        
                // Determine break type based on session count
                if updated_sessions % self.config.sessions_before_long_break == 0 {
                    (
                        TimerState::LongBreak(self.config.long_break_duration),
                        self.config.long_break_duration,
                    )
                } else {
                    (
                        TimerState::ShortBreak(self.config.short_break_duration),
                        self.config.short_break_duration,
                    )
                }
            }
            _ => (TimerState::Working(Utc::now()), self.config.work_duration),
        };
    
        // Update state (we already have this lock)
        *state = next_state_info.0;
        
        // Update the remaining time (with its own lock)
        {
            let mut remaining = self.remaining.lock().await;
            *remaining = next_state_info.1;
        }
        
        // Store the updated state for the message
        let updated_state = state.clone();
        
        // Send update after transition (acquires locks internally)
        // First prepare the values we need outside the function
        let remaining_time = *self.remaining.lock().await;
        let completed_sessions = *self.completed_sessions.lock().await;
        
        // Now send the update with the pre-fetched values instead of acquiring locks again
        let result = self.tx.send(TimerUpdate {
            state: updated_state,
            remaining: remaining_time,
            completed_sessions: completed_sessions,
            total_sessions: self.config.sessions_before_long_break,
        });
    
        if let Err(_) = result {
            println!("Failed to send timer update: no active receivers");
        }
    }
}

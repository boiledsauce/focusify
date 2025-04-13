#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use backend::tokio;
use tauri::Manager;

mod commands {
    use backend::models::timer::{PomodoroConfig, TimerState, TimerUpdate};
    use backend::services::pomodoro::PomodoroTimer;
    use backend::tokio;
    use backend::tokio::sync::broadcast::Receiver;
    use once_cell::sync::Lazy;
    use std::sync::Arc;
    use tauri::{Emitter, Manager, Window};

    // Use a static variable with lazy initialization
    static TIMER: Lazy<Arc<PomodoroTimer>> = Lazy::new(|| {
        let config: PomodoroConfig = PomodoroConfig::default();
        Arc::new(PomodoroTimer::new(config))
    });

    // Function to start the timer loop
    pub async fn init_timer() {
        tokio::spawn(async move {
            TIMER.run_timer_loop().await;
        });
    }

    pub async fn subscribe(app_handle: tauri::AppHandle) -> Result<(), String> {
        let mut rx: Receiver<TimerUpdate> = TIMER.subscribe().await;

        tokio::spawn(async move {
            while let Ok(update) = rx.recv().await {
                println!("Got update from timer: {:?}", update);
                
                // Get main window
                if let Some(window) = app_handle.get_webview_window("main") {
                    // Handle window state based on timer state
                    match &update.state {
                        TimerState::Working(_) => {
                            // Minimize window during work sessions
                            let _ = window.minimize();
                        },
                        TimerState::ShortBreak(_) | TimerState::LongBreak(_) => {
                            // Maximize window during breaks
                            let _ = window.maximize();
                            let _ = window.set_focus(); // Ensure window gets focus
                        },
                        _ => {}
                    }
                }
                
                let _ = app_handle.emit("timer-update", &update);
            }
        });

        Ok(())
    }

    // Start command just updates state
    #[tauri::command]
    pub async fn start_pomodoro() -> Result<(), String> {
        TIMER.start().await;
        Ok(())
    }

    #[tauri::command]
    pub async fn stop_pomodoro() -> Result<(), String> {
        TIMER.stop().await;
        Ok(())
    }
}

fn main() {
    // Initialize tokio runtime and start the timer
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.spawn(commands::init_timer());

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
    
        // Get the monitor's size
        let monitor = window.current_monitor().unwrap().unwrap();
        let monitor_size = monitor.size();
        
        // Center horizontally but keep the top position
        let x = ((monitor_size.width as f64 - 400.0) / 2.0) as i32;
        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y: 10 })).unwrap();
        
        // Make window visible after positioning
        window.show().unwrap();

        window.set_focus().unwrap_or(());

        




            // Spawn subscription task during setup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(commands::subscribe(app_handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_pomodoro,
            commands::stop_pomodoro
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

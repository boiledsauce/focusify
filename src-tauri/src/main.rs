#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands {
    use backend::models::timer::{PomodoroConfig, TimerUpdate};
    use backend::services::pomodoro::PomodoroTimer;
    use backend::tokio;
    use backend::tokio::sync::broadcast::Receiver;
    use tauri::Emitter;

    #[tauri::command]
    pub async fn start_pomodoro(app_handle: tauri::AppHandle) -> Result<(), String> {
        let config = PomodoroConfig::default();
        let timer = PomodoroTimer::new(config);
        let mut rx: Receiver<TimerUpdate> = timer.subscribe().await;

        tokio::spawn(async move {
            timer.start().await;
        });

        tokio::spawn(async move {
            while let Ok(update) = rx.recv().await {
                println!("Got update from timer: {:?}", update);
                let _ = app_handle.emit("timer-update", &update);
            }
        });

        Ok(())
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::start_pomodoro])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

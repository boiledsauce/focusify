import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

interface TimerUpdate {
  state?: string; // "running", "paused", "stopped"
  remaining?: number; // seconds remaining
  completed_sessions?: number;
  total_sessions?: number;
}


// This function calls the Tauri command to start the pomodoro timer.
async function startPomodoro() {
  try {
    await invoke('start_pomodoro')
    console.log('Pomodoro started')
  } catch (error) {
    console.error('Failed to start pomodoro:', error)
  }
}

function PomodoroComponent() {
  const [latestUpdate, setLatestUpdate] = useState<TimerUpdate | null>(null)

  useEffect(() => {
    // Listen for "timer-update" events from Rust
    const unlistenPromise = listen('timer-update', (event) => {
      console.log('Received timer update:', event.payload)
      setLatestUpdate(event.payload as TimerUpdate)
    })

    // Cleanup: unlisten to the event when the component unmounts
    return () => {
      unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  return (
    <div>
      <button onClick={startPomodoro}>Start Pomodoro</button>
      {latestUpdate && (
        <div>
          <p>Timer Update: {JSON.stringify(latestUpdate, null, 2)}</p>
        </div>
      )}
    </div>
  )
}

export default PomodoroComponent

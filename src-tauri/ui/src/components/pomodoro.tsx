import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import './pomodoro.css'  // Make sure this is imported

interface TimerUpdate {
  state?: { type: string; value?: unknown }; 
  remaining?: number;
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

async function stopPomodoro() {
  try {
    await invoke('stop_pomodoro')
    console.log('Pomodoro stopped')
  } catch (error) {
    console.error('Failed to stop pomodoro:', error)
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

  // Check if timer is running based on state type
  const isRunning = latestUpdate?.state?.type === 'Working' || 
                    latestUpdate?.state?.type === 'ShortBreak' || 
                    latestUpdate?.state?.type === 'LongBreak';
  
  // Check if this is a break state
  const isBreak = latestUpdate?.state?.type === 'ShortBreak' || 
                  latestUpdate?.state?.type === 'LongBreak';

  // Format remaining time as MM:SS
  const formatTime = (seconds?: number) => {
    if (seconds === undefined) return '00:00';
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Get current session type
  const getSessionType = (state?: { type: string }) => {
    if (!state) return 'Not Started';
    switch (state.type) {
      case 'Working': return 'Work Session';
      case 'ShortBreak': return 'Short Break';
      case 'LongBreak': return 'Long Break';
      case 'Paused': return 'Paused';
      default: return 'Not Running';
    }
  };

  return (
    <div className="pomodoro-container">
      <div className="timer-display">
        <h2>{getSessionType(latestUpdate?.state)}</h2>
        <div className="time">{formatTime(latestUpdate?.remaining)}</div>
        <div className="sessions">
          Session {latestUpdate?.completed_sessions ?? 0}
        </div>
        
        {isBreak && (
          <div className="break-message">
            <h3>Time to rest!</h3>
            <p>Take a moment to relax and recharge.</p>
          </div>
        )}
      </div>
      
      <div className="controls">
        {isRunning ? (
          <button onClick={stopPomodoro} className="stop-btn">
            Stop
          </button>
        ) : (
          <button onClick={startPomodoro} className="start-btn">
            Start
          </button>
        )}
      </div>
    </div>
  )
}

export default PomodoroComponent
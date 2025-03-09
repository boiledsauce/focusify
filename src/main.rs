use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

// This function sets up a watcher and returns a channel receiver that will provide events
fn watch_directory(dir_path: &Path) -> Result<Receiver<Event>> {
    println!("Starting to watch for changes in: {}", dir_path.display());

    // Create a channel to receive events
    let (sender, receiver) = channel();

    // Create a watcher that will send events to our channel
    let mut watcher = notify::recommended_watcher(move |res: Result<Event>| match res {
        Ok(event) => sender.send(event).unwrap(),
        Err(e) => println!("Error: {:?}", e),
    })?;

    // Start watching the directory
    if dir_path.exists() {
        watcher.watch(dir_path, RecursiveMode::Recursive)?;
        println!("Watching directory for changes...");
    } else {
        println!("Directory not found. Creating it...");
        std::fs::create_dir(dir_path)?;
        watcher.watch(dir_path, RecursiveMode::Recursive)?;
        println!("Created and watching directory for changes...");
    }

    // We need to keep the watcher alive, so we'll use a Box and leak it
    // This is fine for a long-running program but not ideal for all scenarios
    Box::leak(Box::new(watcher));

    // Return the receiver so the caller can get events
    Ok(receiver)
}

fn main() -> Result<()> {
    // Get the current directory where the program is running
    let current_dir = std::env::current_dir()?;

    // Start watching the directory and get the receiver for events
    let event_receiver: Receiver<Event> = watch_directory(&current_dir)?;
    println!("Main process waiting for file changes... (Press Ctrl+C to quit)");

    // Process events in the main loop
    loop {
        match event_receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                match event.kind {
                    EventKind::Create(_) => println!("📝 New file was created!"),
                    EventKind::Modify(_) => handle_event(&event),
                    EventKind::Remove(_) => println!("🗑️ File was deleted!"),
                    _ => (), // Ignore other events
                }
            }
            Err(_) => {} // Timeout, just continue
        }
    }
}

fn handle_event(event: &Event) {
    // List of patterns to ignore
    let ignore_patterns = [".git", ".gitignore", "target", ".vscode"];

    // Disallow files matching any ignore pattern
    if event.paths.iter().any(|path| {
        let path_str = path.to_string_lossy();
        ignore_patterns
            .iter()
            .any(|&pattern| path_str.contains(pattern))
    }) {
        // Git-related event, don't process it
        return;
    }

    // Print the paths that were affected
    println!("Paths affected:");
    for path in &event.paths {
        println!("  {}", path.display());
    }
}

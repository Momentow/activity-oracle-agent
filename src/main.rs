mod db;
mod monitor;

use chrono::{DateTime, Utc};
use std::sync::mpsc;
use std::thread;
use monitor::ActivityEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize DB and print status
    println!("Initializing Database...");
    let conn = match db::init_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("CRITICAL ERROR: Could not open database: {}", e);
            wait_for_user();
            return Err(Box::new(e));
        }
    };
    println!("Database initialized successfully.");

    let (tx, rx) = mpsc::channel::<ActivityEvent>();

    // 2. Start Monitor Thread
    thread::spawn(move || {
        println!("Starting Windows Monitor thread...");
        monitor::start_event_loop(tx);
    });

    let mut last_state: Option<(String, String, DateTime<Utc>)> = None;

    println!("Agent running. Switch windows to track time...");
    println!("(Press Ctrl+C to stop)");

    // 3. Main Loop
    while let Ok(event) = rx.recv() {
        let now = Utc::now();
        println!("Received event: {} ({})", event.title, event.process_name);

        // If we have a previous window, try to log it
        if let Some((old_title, old_process, start_time)) = last_state {
            let duration = (now - start_time).num_seconds();
            
            // IGNORE very short switches (< 1 second) to reduce spam
            if duration > 0 {
                println!(" -> Logging: '{}' ({}s)", old_title, duration);
                
                // CRITICAL FIX: Use match instead of '?' so we don't crash on error
                let result = conn.execute(
                    "INSERT INTO activities (app_name, window_title, start_time, end_time) VALUES (?1, ?2, ?3, ?4)",
                    (
                        &old_process, 
                        &old_title, 
                        start_time.to_rfc3339(), 
                        Some(now.to_rfc3339())
                    ),
                );

                if let Err(e) = result {
                    eprintln!("!!! DATABASE ERROR: {}", e);
                } else {
                    println!(" -> Saved to DB.");
                }
            } else {
                println!(" -> Skipped (duration too short)");
            }
        }

        // Update state
        last_state = Some((event.title, event.process_name, now));
    }

    Ok(())
}

// Helper to keep console open if it crashes early
fn wait_for_user() {
    println!("\nPress Enter to exit...");
    let _ = std::io::stdin().read_line(&mut String::new());
}
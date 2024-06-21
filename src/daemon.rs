use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;

pub fn start(processes: Arc<Mutex<HashMap<String, std::process::Child>>>) {
    thread::spawn(move || {
        loop {
            {
                let mut processes = processes.lock().unwrap();
                processes.retain(|program_name, child| {
                    if let Ok(Some(status)) = child.try_wait() {
                        println!("\nProcess {} exited with status: {}", program_name, status);
                        print!("> ");
                        io::stdout().flush().expect("Flush error");
                        false
                    } else {
                        true
                    }
                });
            }
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::process::Child;
use crate::Program;
use crate::commands::start_program;

pub fn start(processes: Arc<Mutex<HashMap<String, Child>>>, programs: Arc<HashMap<String, Program>>) {
    thread::spawn(move || {
        loop {
            let mut processes_to_restart = Vec::new();
            {
                let mut processes = processes.lock().unwrap();
                processes.retain(|program_name, child| {
                    if let Ok(Some(status)) = child.try_wait() {
                        println!("\nProcess {} exited with status: {}", program_name, status);
                        processes_to_restart.push(program_name.clone());
                        false
                    } else {
                        true
                    }
                });
            }
            for program_name in processes_to_restart {
                if let Some(program) = programs.get(&program_name) {
                    match start_program(program) {
                        Ok(new_child) => {
                            let mut processes = processes.lock().unwrap();
                            processes.insert(program_name.clone(), new_child);
                            println!("Restarted {}", program_name);
							print!("> ");
							io::stdout().flush().expect("Flush error");
                        }
                        Err(e) => {
                            eprintln!("Failed to restart {}: {}", program_name, e);
                        }
                    }
                }
            }
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

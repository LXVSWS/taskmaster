use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::process::Child;
use crate::Program;
use crate::Logger;
use crate::commands::{start_program, reload_config};
use signal_hook::{consts::SIGHUP, iterator::Signals};

pub fn start(programs: Arc<Mutex<HashMap<String, Program>>>, processes: Arc<Mutex<HashMap<String, Child>>>, logger: Arc<Logger>) {
	let processes_clone = Arc::clone(&processes);
    let programs_clone = Arc::clone(&programs);
	let logger_clone = Arc::clone(&logger);

	thread::spawn(move || {
        let mut signals = Signals::new(&[SIGHUP]).expect("Unable to create signal handler");
        for signal in signals.forever() {
            if signal == SIGHUP {
				logger_clone.log("SIGHUP received, reloading config").expect("Failed to log message");
                println!("Received SIGHUP, reloading config...");
				print!("> ");
				io::stdout().flush().expect("Flush error");
                reload_config(&programs_clone, &processes_clone);
            }
        }
    });

    thread::spawn(move || {
        loop {
            let mut processes_to_restart = Vec::new();
            {
                let mut processes = processes.lock().unwrap();
                processes.retain(|program_name, child| {
                    if let Ok(Some(status)) = child.try_wait() {
						logger.log_formatted("Process", format_args!("{} exited with status: {}", program_name, status)).expect("Failed to log message");
                        println!("\nProcess {} exited with status: {}", program_name, status);
                        processes_to_restart.push(program_name.clone());
                        false
                    } else {
                        true
                    }
                });
            }
            for program_name in processes_to_restart {
				let programs = programs.lock().unwrap();
                if let Some(program) = programs.get(&program_name) {
                    match start_program(program) {
                        Ok(new_child) => {
                            let mut processes = processes.lock().unwrap();
                            processes.insert(program_name.clone(), new_child);
							logger.log_formatted("Restarted", format_args!("{}", program_name)).expect("Failed to log message");
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

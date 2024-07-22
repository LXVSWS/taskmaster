use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::process::Child;
use crate::Program;
use crate::Logger;
use crate::commands::{start_program, reload_config};
use signal_hook::{consts::SIGHUP, iterator::Signals};

pub fn start(programs: Arc<Mutex<HashMap<String, Program>>>, processes: Arc<Mutex<HashMap<String, Vec<Child>>>>, logger: Arc<Logger>) {
    let processes_clone = Arc::clone(&processes);
    let programs_clone = Arc::clone(&programs);
    let logger_clone = Arc::clone(&logger);

    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGHUP]).expect("Unable to create signal handler");
        for signal in signals.forever() {
            if signal == SIGHUP {
                logger_clone.log("SIGHUP received, reloading config").expect("Failed to log message");
                println!("Received SIGHUP, reloading config...");
                reload_config(&programs_clone, &processes_clone, &logger_clone);
				print!("> ");
                io::stdout().flush().expect("Flush error");
            }
        }
    });

    thread::spawn(move || {
        loop {
            let mut processes_to_restart = Vec::new();
            {
                let mut processes = processes.lock().unwrap();

                for (program_name, children) in processes.iter_mut() {
                    let mut i = 0;
                    while i < children.len() {
                        if let Ok(Some(status)) = children[i].try_wait() {
							logger.log_formatted("Process", format_args!("{} instance {} exited with status: {}", program_name, i, status))
                            .expect("Failed to log message");
                        	println!("\nProcess {} instance {} exited with status: {}", program_name, i, status);
                            processes_to_restart.push(program_name.clone());
                            children.remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }
            }

            for program_name in processes_to_restart {
                let programs = programs.lock().unwrap();
                if let Some(program) = programs.get(&program_name) {
                    match start_program(program) {
                        Ok(new_child) => {
                            let mut processes = processes.lock().unwrap();
                            processes.entry(program_name.clone()).or_insert_with(Vec::new).push(new_child);
                            logger.log_formatted("Restarted ", format_args!("{} instance", program_name)).expect("Failed to log message");
                            println!("Restarted {} instance", program_name);
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

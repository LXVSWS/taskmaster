use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use crate::Program;
use crate::Logger;
use crate::commands::{start_program, reload_config};
use crate::ProcessInfo;
use crate::commands::check_running_time;
use signal_hook::iterator::Signals;

pub fn start(programs: Arc<Mutex<HashMap<String, Program>>>, processes: Arc<Mutex<HashMap<String, Vec<ProcessInfo>>>>, logger: Arc<Logger>) {
    let processes_clone = Arc::clone(&processes);
    let programs_clone = Arc::clone(&programs);
    let logger_clone = Arc::clone(&logger);

    thread::spawn(move || {
		let all_signals: Vec<i32> = (1..=31).filter(|&signal| {
			signal != libc::SIGKILL &&
			signal != libc::SIGSTOP &&
			signal != libc::SIGILL &&
			signal != libc::SIGFPE &&
			signal != libc::SIGSEGV
		}).collect();
		let mut signals = Signals::new(&all_signals).expect("Unable to create signal handler");
        for signal in signals.forever() {
			if signal == libc::SIGHUP {
                logger_clone.log("SIGHUP received, reloading config").expect("Failed to log message");
                println!("Received SIGHUP, reloading config...");
                reload_config(&programs_clone, &processes_clone, &logger_clone);
                print!("> ");
                io::stdout().flush().expect("Flush error");
            }
			else {
				let programs = programs_clone.lock().unwrap();
				for (name, program) in programs.iter() {
					if signal == program.stopsignal {
						logger_clone
							.log(&format!("Received {} for program '{}', stopping...", signal, name))
							.expect("Failed to log message");
						println!("Gracefully stopping program '{}' in {} seconds...", name, program.stoptime);
						print!("> ");
						io::stdout().flush().expect("Flush error");
						thread::sleep(Duration::from_secs(program.stoptime.into()));
						// integrate stopping childs
					}
				}
        	}
		}
    });

    thread::spawn(move || {
        loop {
            let mut processes_to_restart = Vec::new();

            {
                let mut processes = processes.lock().unwrap();
                let programs = programs.lock().unwrap();

                for (program_name, children) in processes.iter_mut() {
                    if let Some(program) = programs.get(program_name) {
                        let mut i = 0;
                        while i < children.len() {
                            if let Ok(Some(status)) = children[i].child.try_wait() {
                                let exit_code = status.code().unwrap_or(-1);
                                let expected_exit = program.exitcodes.contains(&exit_code);
                                logger.log_formatted("Program", format_args!("{} exited with status: {}", program_name, exit_code))
                                    .expect("Failed to log message");
                                println!("\nProgram {} exited with status: {}", program_name, exit_code);
                                print!("> ");
                                io::stdout().flush().expect("Flush error");
                                if program.autorestart == "always" || (program.autorestart == "unexpected" && !expected_exit) {
                                    processes_to_restart.push((program_name.clone(), i, program.clone()));
                                }
                                children.remove(i);
                            } else {
								check_running_time(program_name, &mut children[i], program.starttime.into(), &logger);
                                i += 1;
                            }
                        }
                    }
                }
                processes.retain(|_, children| !children.is_empty());
            }

            for (program_name, _, program) in processes_to_restart {
                match start_program(&program) {
                    Ok(new_child) => {
                        let mut processes = processes.lock().unwrap();
                        processes.entry(program_name.clone()).or_insert_with(Vec::new).push(new_child);
                        logger.log_formatted("Restarted", format_args!("{} instance", program_name))
                            .expect("Failed to log message");
                        println!("Restarted {} instance", program_name);
                        print!("> ");
                        io::stdout().flush().expect("Flush error");
                    }
                    Err(e) => {
                        eprintln!("Failed to restart {}: {}", program_name, e);
                    }
                }
            }

            thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}

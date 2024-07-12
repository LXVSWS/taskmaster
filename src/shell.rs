use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::Child;
use rustyline::{Editor, error::ReadlineError};
use crate::Program;
use crate::commands::start_program;
use crate::logger::Logger;

pub fn start(programs: Arc<Mutex<HashMap<String, Program>>>, processes: Arc<Mutex<HashMap<String, Child>>>, logger: Arc<Logger>) {
    let mut rl = Editor::<()>::new().expect("Failed to create line editor");
    loop {
        match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let cmd: Vec<&str> = line.trim().split_whitespace().collect();
                if cmd.is_empty() {
                    continue;
                }
                let mut processes = processes.lock().unwrap();
                let programs = programs.lock().unwrap();
                match cmd[0] {
                    "exit" | "quit" => break,
                    "status" => {
                        for (program_name, _program) in programs.iter() {
                            if processes.contains_key(program_name) {
                                println!("{} status: running", program_name);
                            } else {
                                println!("{} status: not running", program_name);
                            }
                        }
                    }
                    "start" => {
                        if cmd.len() < 2 {
                            println!("Please specify a program to start");
                            continue;
                        }
                        let program_name = cmd[1].to_string();
                        if let Some(program) = programs.get(&program_name) {
                            if processes.contains_key(&program_name) {
                                println!("Program {} is already running", program_name);
                                continue;
                            }
                            match start_program(program) {
                                Ok(child) => {
                                    processes.insert(program_name.clone(), child);
									logger.log_formatted("Started", format_args!("{}", program_name)).expect("Failed to log message");
									println!("Started {}", program_name);
                                }
                                Err(e) => {
                                    eprintln!("Failed to start {}: {}", program_name, e);
                                }
                            }
                        } else {
                            println!("Program not found");
                        }
                    }
                    "stop" => {
                        if cmd.len() < 2 {
                            println!("Please specify a program to stop");
                            continue;
                        }
                        let program_name = cmd[1].to_string();
                        if let Some(mut child) = processes.remove(&program_name) {
                            match child.kill() {
                                Ok(_) => {
									logger.log_formatted("Stopped", format_args!("{}", program_name)).expect("Failed to log message");
                                    println!("Stopped {}", program_name);
                                }
                                Err(e) => {
                                    eprintln!("Failed to stop {}: {}", program_name, e);
                                    processes.insert(program_name, child);
                                }
                            }
                        } else {
                            println!("Program not found or not running");
                        }
                    }
                    "restart" => {
                        if cmd.len() < 2 {
                            println!("Please specify a program to restart");
                            continue;
                        }
                        let program_name = cmd[1].to_string();
                        if let Some(mut child) = processes.remove(&program_name) {
                            match child.kill() {
                                Ok(_) => {
									logger.log_formatted("Stopped", format_args!("{}", program_name)).expect("Failed to log message");
                                    println!("Stopped {}", program_name);
                                }
                                Err(e) => {
                                    eprintln!("Failed to stop {}: {}", program_name, e);
                                    processes.insert(program_name.clone(), child);
                                }
                            }
                        }
                        if let Some(program) = programs.get(&program_name) {
                            match start_program(program) {
                                Ok(child) => {
                                    processes.insert(program_name.clone(), child);
									logger.log_formatted("Restarted", format_args!("{}", program_name)).expect("Failed to log message");
                                    println!("Restarted {}", program_name);
                                }
                                Err(e) => {
                                    eprintln!("Failed to start {}: {}", program_name, e);
                                }
                            }
                        } else {
                            println!("Program not found");
                        }
                    }
                    _ => {
                        println!("Unknown command");
                    }
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    let mut processes = processes.lock().unwrap();
    for (program_name, mut child) in processes.drain() {
        match child.kill() {
            Ok(_) => {
				logger.log_formatted("Killed", format_args!("{}", program_name)).expect("Failed to log message");
				println!("Killed {}", program_name);
			}
            Err(e) => eprintln!("Failed to kill {}: {}", program_name, e),
        }
    }
}

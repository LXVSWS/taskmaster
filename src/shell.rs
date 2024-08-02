use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;
use rustyline::{Editor, error::ReadlineError};
use crate::Program;
use crate::commands::start_program;
use crate::logger::Logger;
use crate::ProcessInfo;
use crate::commands::check_running_time;

pub fn start(programs: Arc<Mutex<HashMap<String, Program>>>, processes: Arc<Mutex<HashMap<String, Vec<ProcessInfo>>>>, logger: Arc<Logger>) {
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
                        for program_name in programs.keys() {
                            if let Some(instances) = processes.get(program_name) {
                                if instances.is_empty() {
                                    println!("{} status: exited", program_name);
                                } else {
                                    println!("{} status: running ({} instances)", program_name, instances.len());
                                }
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
						if processes.contains_key(&program_name) {
							println!("Program {} is already running", program_name);
							continue;
						}
						let program = match programs.get(&program_name) {
							Some(program) => program,
							None => {
								println!("Program not found");
								continue;
							}
						};

						let mut instances = Vec::new();
						for i in 0..program.numprocs {
							let mut attempts = 0;
							while attempts < program.startretries {
								match start_program(program) {
									Ok(process_info) => {
										instances.push(process_info);
										if let Some(last_process_info) = instances.last_mut() {
											check_running_time(&program_name, last_process_info, program.starttime.into(), &logger);
										}
										logger.log_formatted("Started", format_args!("{} instance {}", program_name, i))
											.expect("Failed to log message");
										println!("Started {} instance {}", program_name, i);
										break;
									}
									Err(e) => {
										eprintln!("Failed to start {} instance {}: {}", program_name, i, e);
										attempts += 1;
										if attempts < program.startretries {
											logger.log_formatted("Retry", format_args!("Retrying to start {} instance {} (attempt {}/{})", program_name, i, attempts + 1, program.startretries))
												.expect("Failed to log message");
											eprintln!("Retrying to start {} instance {} (attempt {}/{})", program_name, i, attempts + 1, program.startretries);
										}
									}
								}
							}
						}
						if !instances.is_empty() {
							processes.insert(program_name, instances);
						}
					}
                    "stop" => {
                        if cmd.len() < 2 {
                            println!("Please specify a program to stop");
                            continue;
                        }
                        let program_name = cmd[1].to_string();
                        if let Some(mut instances) = processes.remove(&program_name) {
                            for (i, mut process_info) in instances.drain(..).enumerate() {
                                match process_info.child.kill() {
                                    Ok(_) => {
                                        logger.log_formatted("Stopped", format_args!("{} instance {}", program_name, i))
                                            .expect("Failed to log message");
                                        println!("Stopped {} instance {}", program_name, i);
										process_info.time_elapsed_since_stop = Some(Instant::now());
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to stop {} instance {}: {}", program_name, i, e);
                                        processes.entry(program_name.clone()).or_insert_with(Vec::new).push(process_info);
                                    }
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
						match processes.remove(&program_name) {
							Some(instances) => {
								for (i, mut instance) in instances.into_iter().enumerate() {
									let _ = instance.child.kill();
									let _ = instance.child.wait();
									logger.log_formatted("Stopped", format_args!("{} instance {}", program_name, i))
										.expect("Failed to log message");
									println!("Stopped {} instance {}", program_name, i);
								}

								let program = match programs.get(&program_name) {
									Some(program) => program,
									None => {
										println!("Program not found");
										continue;
									}
								};

								let mut new_instances = Vec::new();
								for i in 0..program.numprocs {
									let mut attempts = 0;
									while attempts < program.startretries {
										match start_program(program) {
											Ok(process_info) => {
												new_instances.push(process_info);
												if let Some(last_process_info) = new_instances.last_mut() {
													check_running_time(&program_name, last_process_info, program.starttime.into(), &logger);
												}
												logger.log_formatted("Started", format_args!("{} instance {}", program_name, i))
													.expect("Failed to log message");
												println!("Started {} instance {}", program_name, i);
												break;
											}
											Err(e) => {
												eprintln!("Failed to start {} instance {}: {}", program_name, i, e);
												attempts += 1;
												if attempts < program.startretries {
													logger.log_formatted("Retry", format_args!("Retrying to restart {} instance {} (attempt {}/{})", program_name, i, attempts + 1, program.startretries))
														.expect("Failed to log message");
													eprintln!("Retrying to restart {} instance {} (attempt {}/{})", program_name, i, attempts + 1, program.startretries);
												}
											}
										}
									}
								}
								if !new_instances.is_empty() {
									processes.insert(program_name, new_instances);
								}
							}
							None => {
								println!("Program not running");
							}
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
    for (program_name, mut instances) in processes.drain() {
        for (i, mut process_info) in instances.drain(..).enumerate() {
            match process_info.child.kill() {
                Ok(_) => {
                    logger.log_formatted("Killed", format_args!("{} instance {}", program_name, i))
                        .expect("Failed to log message");
                    println!("Killed {} instance {}", program_name, i);
                }
                Err(e) => eprintln!("Failed to kill {} instance {}: {}", program_name, i, e),
            }
        }
    }
}

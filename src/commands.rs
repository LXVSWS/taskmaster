use std::fs::File;
use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;
use crate::{parsing, Program, Logger};
use crate::ProcessInfo;
use std::os::unix::process::CommandExt;
use libc;

pub fn start_program(program: &Program) -> Result<ProcessInfo, std::io::Error> {
    let mut command_parts = program.cmd.split_whitespace();
    let executable = command_parts.next().expect("Executable not found");
    let args: Vec<&str> = command_parts.collect();
    let mut command = Command::new(executable);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(File::create(&program.stdout)?)
        .stderr(File::create(&program.stderr)?)
		.current_dir(&program.workingdir);
    if let Some(ref env) = program.env {
            command.envs(env);
    };
    let new_umask = u16::from_str_radix(&program.umask, 8).expect("Failed to parse umask");

    unsafe {
        command.pre_exec(move || {
            println!("Setting umask to: {:o}", new_umask);  // Log before setting umask
            libc::umask(new_umask); // Appel direct de libc::umask sans fonction intermédiaire
            println!("Umask set to: {:o}", new_umask);  // Log after setting umask
            Ok(())
        });
    }
    let child = command.spawn()?;
    Ok(ProcessInfo {
        child,
        start_time: Instant::now(),
        time_elapsed_since_stop: None,
		successfully_started: false
    })
}

pub fn autostart_programs(programs: &Arc<Mutex<HashMap<String, Program>>>, processes: &Arc<Mutex<HashMap<String, Vec<ProcessInfo>>>>, logger: &Arc<Logger>) {
    let programs = programs.lock().unwrap();
    let mut processes = processes.lock().unwrap();

    for (name, program) in programs.iter() {
        if program.autostart {
            let mut instances = Vec::new();

            for i in 0..program.numprocs {
                let mut attempts = 0;
                while attempts < program.startretries {
                    match start_program(program) {
                        Ok(process_info) => {
                            instances.push(process_info);
                            if let Some(last_process_info) = instances.last_mut() {
                                check_running_time(name, last_process_info, program.starttime.into(), &logger);
                            }
                            logger.log_formatted("Started", format_args!("{} instance {}", name, i))
                                .expect("Failed to log message");
                            println!("Started {} instance {}", name, i);
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to start {} instance {}: {}", name, i, e);
                            attempts += 1;
                            if attempts < program.startretries {
                                logger.log_formatted("Retry", format_args!("Retrying to start {} instance {} (attempt {}/{})", name, i, attempts + 1, program.startretries))
                                    .expect("Failed to log message");
                                eprintln!("Retrying to start {} instance {} (attempt {}/{})", name, i, attempts + 1, program.startretries);
                            }
                        }
                    }
                }
            }

            if !instances.is_empty() {
                processes.insert(name.clone(), instances);
            }
        }
    }
}

pub fn reload_config(programs: &Arc<Mutex<HashMap<String, Program>>>, processes: &Arc<Mutex<HashMap<String, Vec<ProcessInfo>>>>, logger: &Arc<Logger>) {
    let new_programs = parsing();
    let mut programs = programs.lock().unwrap();
    let mut processes = processes.lock().unwrap();

    for (name, program) in &new_programs {
        if let Some(existing_program) = programs.get(name) {
            if existing_program != program {
                if let Some(children) = processes.remove(name) {
					for (i, mut process_info) in children.into_iter().enumerate() {
                        if process_info.child.kill().is_ok() {
                            logger.log_formatted("Killed", format_args!("{} instance {}", name, i))
                                .expect("Failed to log message");
                            println!("Killed {} instance {}", name, i);
                        }
					}
                }
            }
        }
    }

    let old_programs: Vec<String> = programs.keys().cloned().collect();
    for old_program in old_programs {
        if !new_programs.contains_key(&old_program) {
            if let Some(children) = processes.remove(&old_program) {
				for (i, mut process_info) in children.into_iter().enumerate() {
                    if process_info.child.kill().is_ok() {
                        logger.log_formatted("Killed", format_args!("{} instance {}", old_program, i))
                            .expect("Failed to log message");
                        println!("Killed {} instance {}", old_program, i);
                    }
                }
            }
        }
    }

    *programs = new_programs;

	for (name, program) in programs.iter() {
        if program.autostart && (!processes.contains_key(name) || processes.get(name).unwrap().is_empty()) {
            let mut instances = Vec::new();
            for i in 0..program.numprocs {
                let mut attempts = 0;
                while attempts < program.startretries {
                    match start_program(program) {
                        Ok(process_info) => {
                            instances.push(process_info);
                            if let Some(last_process_info) = instances.last_mut() {
                                check_running_time(name, last_process_info, program.starttime.into(), &logger);
                            }
                            logger.log_formatted("Started", format_args!("{} instance {}", name, i))
                                .expect("Failed to log message");
                            println!("Started {} instance {}", name, i);
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to start {} instance {}: {}", name, i, e);
                            attempts += 1;
                            if attempts < program.startretries {
                                logger.log_formatted("Retry", format_args!("Retrying to start {} instance {} (attempt {}/{})", name, i, attempts + 1, program.startretries))
                                    .expect("Failed to log message");
                                eprintln!("Retrying to start {} instance {} (attempt {}/{})", name, i, attempts + 1, program.startretries);
                            }
                        }
                    }
                }

                if attempts >= program.startretries {
                    logger.log_formatted("Failed", format_args!("Failed to start {} instance {} after {} attempts", name, i, program.startretries))
                        .expect("Failed to log message");
                    eprintln!("Failed to start {} instance {} after {} attempts", name, i, program.startretries);
                }
            }

            if !instances.is_empty() {
                processes.insert(name.clone(), instances);
            }
        }
    }
}

pub fn check_running_time(program_name: &str, process_info: &mut ProcessInfo, starttime: u64, logger: &Arc<Logger>) {
    let elapsed_time = process_info.start_time.elapsed().as_secs();
    if !process_info.successfully_started && elapsed_time >= starttime {
        let message = format!("{} successfully started ({} seconds)", program_name, elapsed_time);
        println!("{}", message);
        logger.log(&message).expect("Failed to log message");
		print!("> ");
		io::stdout().flush().expect("Flush error");
        process_info.successfully_started = true;
    }
}

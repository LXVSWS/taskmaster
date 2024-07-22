use std::fs::File;
use std::process::{Command, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::{parsing, Program, Logger};

pub fn start_program(program: &Program) -> Result<Child, std::io::Error> {
    let mut command_parts = program.cmd.split_whitespace();
    let executable = command_parts.next().expect("Executable not found");
    let args: Vec<&str> = command_parts.collect();
    Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .stdout(File::create(&program.stdout)?)
        .stderr(File::create(&program.stderr)?)
        .spawn()
}

pub fn autostart_programs(programs: &Arc<Mutex<HashMap<String, Program>>>, processes: &Arc<Mutex<HashMap<String, Vec<Child>>>>, logger: &Arc<Logger>) {
    let programs = programs.lock().unwrap();
    let mut processes = processes.lock().unwrap();
    for (name, program) in programs.iter() {
        if program.autostart {
            for i in 0..program.numprocs {
                match start_program(program) {
                    Ok(child) => {
                        processes.entry(name.clone()).or_insert_with(Vec::new).push(child);
                        logger.log_formatted("Started", format_args!("{} instance {}", name, i))
                            .expect("Failed to log message");
                        println!("Started {} instance {}", name, i);
                    }
                    Err(e) => {
                        eprintln!("Failed to start {} instance {}: {}", name, i, e);
                    }
                }
            }
        }
    }
}

pub fn reload_config(programs: &Arc<Mutex<HashMap<String, Program>>>, processes: &Arc<Mutex<HashMap<String, Vec<Child>>>>, logger: &Arc<Logger>) {
    let new_programs = parsing();
    let mut programs = programs.lock().unwrap();
    let mut processes = processes.lock().unwrap();

    for (name, program) in &new_programs {
        if let Some(existing_program) = programs.get(name) {
            if existing_program != program {
                if let Some(children) = processes.remove(name) {
					for (i, mut child) in children.into_iter().enumerate() {
                        if child.kill().is_ok() {
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
				for (i, mut child) in children.into_iter().enumerate() {
                    if child.kill().is_ok() {
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
            for i in 0..program.numprocs {
                match start_program(program) {
                    Ok(child) => {
                        processes.entry(name.clone()).or_insert_with(Vec::new).push(child);
                        logger.log_formatted("Started", format_args!("{} instance {}", name, i))
                            .expect("Failed to log message");
                        println!("Started {} instance {}", name, i);
                    }
                    Err(e) => {
                        eprintln!("Failed to start {} instance {}: {}", name, i, e);
                    }
                }
            }
        }
    }
}

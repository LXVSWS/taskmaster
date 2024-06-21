use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use std::collections::HashMap;
use std::process::Child;
use crate::Program;
use crate::process;

pub fn start(programs: HashMap<String, Program>, processes: Arc<Mutex<HashMap<String, Child>>>) {
    loop {
        print!("> ");
        io::stdout().flush().expect("Flush error");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Readline error");
        let cmd: Vec<&str> = input.trim().split_whitespace().collect();
        if cmd.is_empty() {
            continue;
        }
        match cmd[0] {
            "exit" | "quit" => break,
            "status" => {
                let processes = processes.lock().unwrap();
                for (program_name, _program) in &programs {
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
                    let mut processes = processes.lock().unwrap();
                    if processes.contains_key(&program_name) {
                        println!("Program {} is already running", program_name);
                        continue;
                    }
                    match process::start(&program) {
                        Ok(child) => {
                            processes.insert(program_name.clone(), child);
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
                let mut processes = processes.lock().unwrap();
                if let Some(mut child) = processes.remove(&program_name) {
                    match child.kill() {
                        Ok(_) => {
                            println!("Killed {}", program_name);
                        }
                        Err(e) => {
                            eprintln!("Failed to kill {}: {}", program_name, e);
                            processes.insert(program_name, child);
                        }
                    }
                } else {
                    println!("Program not found or not running");
                }
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
    let mut processes = processes.lock().unwrap();
    for (program_name, mut child) in processes.drain() {
        match child.kill() {
            Ok(_) => println!("Killed {}", program_name),
            Err(e) => eprintln!("Failed to kill {}: {}", program_name, e),
        }
    }
}

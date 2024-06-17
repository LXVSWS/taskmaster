use std::io::{self, Write};
use std::fs::{self, File};
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio, Child};
use std::collections::HashMap;
use std::thread;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Program {
    cmd: String,
    numprocs: u32,
    umask: String,
    workingdir: String,
    autostart: bool,
    autorestart: String,
    exitcodes: Vec<i32>,
    startretries: u32,
    starttime: u32,
    stopsignal: String,
    stoptime: u32,
    stdout: String,
    stderr: String,
    env: Option<HashMap<String, String>>,
}

fn parsing() -> HashMap<String, Program> {
    let config = fs::read_to_string("config.yml").expect("Failed to read config file");
	serde_yaml::from_str(&config).expect("Failed to parse config")
}

fn start_process(program: &Program) -> Result<Child, std::io::Error> {
    let mut command = program.cmd.split_whitespace();
    let executable = command.next().expect("Executable not found");
    let args: Vec<&str> = command.collect();
    Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .stdout(File::create(&program.stdout)?)
        .stderr(File::create(&program.stderr)?)
        .spawn()
}

fn daemon(processes: Arc<Mutex<HashMap<String, Child>>>) {
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

fn shell(programs: HashMap<String, Program>, processes: Arc<Mutex<HashMap<String, Child>>>) {
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
                    match start_process(&program) {
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

fn main() {
    println!("Taskmaster");
    let processes = Arc::new(Mutex::new(HashMap::new()));
    daemon(Arc::clone(&processes));
	shell(parsing(), processes);
    println!("Bye");
}

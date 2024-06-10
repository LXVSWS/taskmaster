use std::io::{self, Write};
use std::fs::{self, File};
use std::process::{Command, Stdio, Child};
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
	programs: HashMap<String, Program>,
}

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

fn parsing() -> Config {
    let config_content = fs::read_to_string("config.yml").expect("Failed to read config file");
    let config: Config = serde_yaml::from_str(&config_content).expect("Failed to parse config");
	config
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

fn control_shell(config: Config) {
	let mut running_processes: HashMap<String, Child> = HashMap::new();
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
				for (program_name, _config_values) in &config.programs {
					if running_processes.contains_key(program_name) {
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
				let program_name = cmd[1];
				if let Some(program) = config.programs.get(program_name) {
					if running_processes.contains_key(program_name) {
                        println!("Program {} is already running", program_name);
                        continue;
                    }
					match start_process(&program) {
						Ok(child) => {
							running_processes.insert(program_name.to_string(), child);
							println!("Started {}", program_name);
						}
						Err(e) => {
							eprintln!("Failed to start {}: {}", program_name, e);
						}
					}
				}
				else {
					println!("Program not found");
				}
			}
			"stop" => {
				if cmd.len() < 2 {
					println!("Please specify a program to stop");
					continue;
				}
				let program_name = cmd[1].to_string();
				if let Some(mut child) = running_processes.remove(&program_name) {
					match child.kill() {
						Ok(_) => {
							println!("Killed {}", program_name);
						}
						Err(e) => {
							eprintln!("Failed to kill {}: {}", program_name, e);
							running_processes.insert(program_name, child);
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
	for (program_name, mut child) in running_processes {
        match child.kill() {
            Ok(_) => println!("Killed {}", program_name),
            Err(e) => eprintln!("Failed to kill {}: {}", program_name, e),
        }
    }
}

fn main() {
    println!("Taskmaster");
	control_shell(parsing());
	println!("Bye");
}

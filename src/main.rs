mod shell;
mod daemons;
mod commands;
mod logger;

use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::Child;
use serde::Deserialize;
use std::time::Instant;
use crate::logger::Logger;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Program {
    cmd: String,
    numprocs: u32,
    umask: String,
    workingdir: String,
    autostart: bool,
    autorestart: String,
    exitcodes: Vec<i32>,
    startretries: u32,
    starttime: u32,
    stopsignal: i32,
    stoptime: u32,
    stdout: String,
    stderr: String,
    env: Option<HashMap<String, String>>,
}

pub struct ProcessInfo {
    pub child: Child,
    pub start_time: Instant,
    pub time_elapsed_since_stop: Option<Instant>,
	pub successfully_started: bool
}

fn parsing() -> HashMap<String, Program> {
    let config = fs::read_to_string("config.yml").expect("Failed to read config file");
	serde_yaml::from_str(&config).expect("Failed to parse config")
}

fn main() {
    println!("Taskmaster");
	let logger = Arc::new(Logger::new("taskmaster.log").expect("Failed to create logger"));
    let processes = Arc::new(Mutex::new(HashMap::<String, Vec<ProcessInfo>>::new()));
    let programs = Arc::new(Mutex::new(parsing()));
    commands::autostart_programs(&programs, &processes, &logger);
    daemons::start(programs.clone(), processes.clone(), logger.clone());
    shell::start(programs, processes, logger);
    println!("Bye");
}

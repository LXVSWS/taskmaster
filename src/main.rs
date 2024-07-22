mod shell;
mod daemons;
mod commands;
mod logger;

use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::Child;
use serde::Deserialize;
use crate::logger::Logger;

#[derive(Debug, Deserialize, PartialEq)]
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

fn main() {
    println!("Taskmaster");
	let logger = Arc::new(Logger::new("taskmaster.log").expect("Failed to create logger"));
    let processes = Arc::new(Mutex::new(HashMap::<String, Vec<Child>>::new()));
    let programs = Arc::new(Mutex::new(parsing()));
    let programs_clone = Arc::clone(&programs);
    let processes_clone = Arc::clone(&processes);
    let logger_clone = Arc::clone(&logger);
    commands::autostart_programs(&programs_clone, &processes_clone, &logger_clone);
    daemons::start(programs_clone, processes_clone, logger_clone);
    shell::start(programs, processes, logger);
    println!("Bye");
}

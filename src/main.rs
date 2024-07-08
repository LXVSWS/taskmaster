mod shell;
mod daemons;
mod commands;

use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::Deserialize;

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
    let processes = Arc::new(Mutex::new(HashMap::new()));
    let programs = Arc::new(Mutex::new(parsing()));
    daemons::start(Arc::clone(&programs), Arc::clone(&processes));
    shell::start(Arc::clone(&programs), Arc::clone(&processes));
    println!("Bye");
}

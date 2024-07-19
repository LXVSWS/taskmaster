use std::fs::File;
use std::process::{Command, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::{parsing, Program};

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

pub fn reload_config(programs: &Arc<Mutex<HashMap<String, Program>>>, processes: &Arc<Mutex<HashMap<String, Vec<Child>>>>) {
    let new_programs = parsing();
    let mut programs = programs.lock().unwrap();
    let mut processes = processes.lock().unwrap();

    for (name, program) in &new_programs {
        if let Some(existing_program) = programs.get(name) {
            if existing_program != program {
                if let Some(children) = processes.remove(name) {
                    for mut child in children {
                        child.kill().ok();
                    }
                }
            }
        }
    }

    let old_programs: Vec<String> = programs.keys().cloned().collect();
    for old_program in old_programs {
        if !new_programs.contains_key(&old_program) {
            if let Some(children) = processes.remove(&old_program) {
                for mut child in children {
                    child.kill().ok();
                }
            }
        }
    }

    *programs = new_programs;
}

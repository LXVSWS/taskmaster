use std::process::{Command, Child, Stdio};
use std::fs::File;
use crate::Program;

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

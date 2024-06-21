use std::process::{Command, Stdio, Child};
use std::fs::File;
use crate::Program;

pub fn start(program: &Program) -> Result<Child, std::io::Error> {
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

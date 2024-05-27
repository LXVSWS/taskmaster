use std::io::{self, Write};
use std::process::Command;

fn main() {
    println!("Taskmaster");
	loop {
		print!("> ");
		io::stdout().flush().expect("Flush error");
		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Readline error");
		let input = input.trim_end();
		if input == "exit" || input == "quit" {
			break;
		}
		let command = Command::new(input).output().expect("Launch command failed");
		println!("{}", String::from_utf8_lossy(&command.stdout));
	}
	println!("Bye");
}

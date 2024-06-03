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
		let mut process = Command::new(input);
		match process.spawn() {
			Ok(mut child) => {
				let ecode = child.wait().expect("Wait error");
				println!("{}", ecode);
			}
			Err(e) => {
				eprintln!("Spawn error : {}", e);
			}
		}
	}
	println!("Bye");
}

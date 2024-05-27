use std::io::{self, Write};

fn main() {
    println!("Taskmaster");
	loop {
		print!("> ");
		io::stdout().flush().expect("Flush error");
		let mut input = String::new();
		io::stdin().read_line(&mut input).expect("Readline error");
		if input == "test\n" {
			print!("Command : {}", input);
		}
		else if input == "exit\n" || input == "quit\n" {
			break;
		}
		else {
			println!("Unknow command");
		}
	}
	println!("Bye");
}

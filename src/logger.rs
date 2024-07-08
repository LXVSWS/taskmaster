use std::fs::{OpenOptions, File};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub struct Logger {
    log_file: Arc<Mutex<File>>,
}

impl Logger {
    pub fn new(log_file: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
        Ok(Logger {
            log_file: Arc::new(Mutex::new(file)),
        })
    }

    pub fn log(&self, message: &str) -> io::Result<()> {
        let mut file = self.log_file.lock().unwrap();
        writeln!(file, "{}", message)?;
        Ok(())
    }

	pub fn log_formatted(&self, format_str: &str, args: std::fmt::Arguments) -> io::Result<()> {
        let mut file = self.log_file.lock().unwrap();
        writeln!(file, "{}", format_args!("{} {}", format_str, args))?;
        Ok(())
    }
}

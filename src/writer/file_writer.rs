use super::Writer;
use std::{io::Write, sync::RwLock};

pub struct FileWriter {
    log_file: RwLock<std::fs::File>,
}

impl FileWriter {
    pub fn new(log_file: std::path::PathBuf) -> Self {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)
            .expect("Unable to open log file");
        Self {
            log_file: RwLock::new(file),
        }
    }
}

impl Writer for FileWriter {
    fn write(&self, log: String) {
        let mut file = self
            .log_file
            .write()
            .expect("log file lock must be handled...");
        if let Err(err) = writeln!(file, "{}", log) {
            eprintln!("Unable to write to log file: {}", err.to_string());
        }
    }
}

use super::Writer;

pub struct StderrWriter;

impl Writer for StderrWriter {
    fn write(&self, log: String) {
        eprintln!("{}", log);
    }
}

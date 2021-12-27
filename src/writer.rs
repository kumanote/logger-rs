pub trait Writer: Send + Sync {
    fn write(&self, log: String);
}

mod stderr_writer;
pub use stderr_writer::StderrWriter;

mod file_writer;
pub use file_writer::FileWriter;

#[cfg(any(feature = "tcp"))]
mod tcp_writer;
#[cfg(any(feature = "tcp"))]
pub use tcp_writer::TcpWriter;

#[cfg(any(feature = "airbrake"))]
mod http_writer;
#[cfg(any(feature = "airbrake"))]
pub use http_writer::HttpWriter;

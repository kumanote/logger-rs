use super::Writer;
use std::sync::RwLock;
use std::{
    io,
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};

const NUM_SEND_RETRIES: u8 = 1;
const WRITE_TIMEOUT_MS: u64 = 2000;
const CONNECTION_TIMEOUT_MS: u64 = 5000;

pub struct TcpWriter {
    inner: RwLock<TcpStreamHandler>,
}

impl TcpWriter {
    pub fn new(endpoint: String) -> Self {
        Self {
            inner: RwLock::new(TcpStreamHandler::new(endpoint)),
        }
    }

    pub fn flush(&self) {
        let mut inner = self
            .inner
            .write()
            .expect("tcp stream lock must be handled...");
        if let Err(e) = inner.flush() {
            eprintln!(
                "[Logging] Error while flushing data to tcp endpoint({}): {}",
                inner.endpoint(),
                e
            );
        }
    }
}

impl Writer for TcpWriter {
    fn write(&self, log: String) {
        let bytes = log.as_bytes();
        let mut inner = self
            .inner
            .write()
            .expect("tcp stream lock must be handled...");
        let mut result = inner.write_all(bytes);
        for _ in 0..NUM_SEND_RETRIES {
            if result.is_ok() {
                break;
            } else {
                result = inner.write_all(bytes);
            }
        }
        if let Err(e) = result {
            eprintln!(
                "[Logging] Error while sending data to tcp endpoint({}): {}",
                inner.endpoint(),
                e
            );
        }
    }
}

struct TcpStreamHandler {
    endpoint: String,
    stream: Option<TcpStream>,
    last_connection_attempt: Option<Instant>,
}

impl TcpStreamHandler {
    fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            stream: None,
            last_connection_attempt: None,
        }
    }

    fn endpoint(&self) -> &str {
        self.endpoint.as_str()
    }

    fn connect(&mut self) -> io::Result<TcpStream> {
        let mut last_error = io::Error::new(
            io::ErrorKind::Other,
            format!("Unable to resolve and connect to {}", self.endpoint),
        );

        for socket_address in self.endpoint.to_socket_addrs()? {
            match TcpStream::connect_timeout(
                &socket_address,
                Duration::from_millis(CONNECTION_TIMEOUT_MS),
            ) {
                Ok(stream) => {
                    if let Err(err) =
                        stream.set_write_timeout(Some(Duration::from_millis(WRITE_TIMEOUT_MS)))
                    {
                        eprintln!("[Logging] Failed to set write timeout: {}", err);
                        continue;
                    }
                    return Ok(stream);
                }
                Err(err) => last_error = err,
            }
        }

        Err(last_error)
    }

    fn refresh_connection(&mut self) -> io::Result<()> {
        // Only refresh the connection once a second
        if self
            .last_connection_attempt
            .map(|t| t.elapsed() > Duration::from_millis(1000))
            .unwrap_or(true)
        {
            self.last_connection_attempt = Some(Instant::now());
            match self.connect() {
                Ok(stream) => {
                    self.stream = Some(stream);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[Logging] Failed to connect: {}", e);
                    Err(e)
                }
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "unable to refresh connection",
            ))
        }
    }
}

impl Write for TcpStreamHandler {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.stream.is_none() {
            self.refresh_connection()?;
        }
        self.stream
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "No stream"))
            .and_then(|stream| stream.write(buf))
            .map_err(|e| {
                self.stream = None;
                e
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(mut stream) = self.stream.as_ref() {
            stream.flush()
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Can't flush, not connected",
            ))
        }
    }
}

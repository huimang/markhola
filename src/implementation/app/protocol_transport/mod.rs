mod discovery;
mod peer;

#[cfg(test)]
mod tests;

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use discovery::{EndpointPaths, PublishedEndpoint};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const PROTOCOL_VERSION: u32 = 1;
const NOT_READY_RESPONSE: &[u8] = br#"{"ok":false,"error_code":"protocol_not_ready","message":"The command layer is not ready."}"#;

pub(super) struct ProtocolTransport {
    endpoint: PublishedEndpoint,
    stopping: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl ProtocolTransport {
    pub(super) fn start() -> Result<Self, String> {
        Self::start_with_paths(EndpointPaths::for_current_user()?)
    }

    fn start_with_paths(paths: EndpointPaths) -> Result<Self, String> {
        let endpoint = PublishedEndpoint::create(paths, PROTOCOL_VERSION)?;
        let listener = UnixListener::bind(endpoint.socket_path())
            .map_err(|error| format!("Failed to bind automation socket: {error}"))?;
        discovery::set_private_mode(endpoint.socket_path(), 0o600)?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("Failed to configure automation socket: {error}"))?;

        let stopping = Arc::new(AtomicBool::new(false));
        let worker_stopping = Arc::clone(&stopping);
        let expected_uid = peer::current_uid();
        let worker = thread::Builder::new()
            .name("markhola-protocol-transport".to_string())
            .spawn(move || run_listener(listener, expected_uid, worker_stopping))
            .map_err(|error| format!("Failed to start automation transport: {error}"))?;

        Ok(Self {
            endpoint,
            stopping,
            worker: Some(worker),
        })
    }

    #[cfg(test)]
    fn record_path(&self) -> &std::path::Path {
        self.endpoint.record_path()
    }

    #[cfg(test)]
    fn socket_path(&self) -> &std::path::Path {
        self.endpoint.socket_path()
    }
}

impl Drop for ProtocolTransport {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
        let _ = UnixStream::connect(self.endpoint.socket_path());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run_listener(listener: UnixListener, expected_uid: u32, stopping: Arc<AtomicBool>) {
    while !stopping.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => handle_connection(stream, expected_uid),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

fn handle_connection(mut stream: UnixStream, expected_uid: u32) {
    if peer::peer_uid(&stream) != Ok(expected_uid) {
        return;
    }
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    if read_frame(&mut stream).is_err() {
        return;
    }

    let _ = stream.write_all(NOT_READY_RESPONSE);
    let _ = stream.write_all(b"\n");
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, ()> {
    let mut payload = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let count = stream.read(&mut chunk).map_err(|_| ())?;
        if count == 0 {
            break;
        }
        payload.extend_from_slice(&chunk[..count]);
        if payload.len() > MAX_REQUEST_BYTES {
            return Err(());
        }
        if let Some(newline) = payload.iter().position(|byte| *byte == b'\n') {
            if payload[newline + 1..]
                .iter()
                .any(|byte| !byte.is_ascii_whitespace())
            {
                return Err(());
            }
            payload.truncate(newline);
            break;
        }
    }
    (!payload.is_empty()).then_some(payload).ok_or(())
}

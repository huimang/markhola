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
use serde::Deserialize;

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
        let expected_token = endpoint.instance_token().to_string();
        let worker = thread::Builder::new()
            .name("markhola-protocol-transport".to_string())
            .spawn(move || run_listener(listener, expected_uid, expected_token, worker_stopping))
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

fn run_listener(
    listener: UnixListener,
    expected_uid: u32,
    expected_token: String,
    stopping: Arc<AtomicBool>,
) {
    while !stopping.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => handle_connection(stream, expected_uid, &expected_token),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

fn handle_connection(mut stream: UnixStream, expected_uid: u32, expected_token: &str) {
    if peer::peer_uid(&stream) != Ok(expected_uid) {
        return;
    }
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    let Ok(payload) = read_frame(&mut stream) else {
        return;
    };
    if !frame_has_exact_token(&payload, expected_token) {
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
            return Err(());
        }
        payload.extend_from_slice(&chunk[..count]);
        if payload.len() > MAX_REQUEST_BYTES {
            return Err(());
        }
        if let Some(newline) = payload.iter().position(|byte| *byte == b'\n') {
            if payload.len() != newline + 1 {
                return Err(());
            }
            if newline == 0 {
                return Err(());
            }
            payload.truncate(newline);
            return Ok(payload);
        }
    }
}

#[derive(Deserialize)]
struct TransportEnvelope<'a> {
    instance_token: &'a str,
}

fn frame_has_exact_token(payload: &[u8], expected_token: &str) -> bool {
    let Ok(envelope) = serde_json::from_slice::<TransportEnvelope<'_>>(payload) else {
        return false;
    };
    constant_time_eq(
        envelope.instance_token.as_bytes(),
        expected_token.as_bytes(),
    )
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

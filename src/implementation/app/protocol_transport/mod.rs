mod control;
mod discovery;
mod peer;

#[cfg(test)]
mod tests;

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use control::ExportControl;
use discovery::{EndpointPaths, PublishedEndpoint};
use serde::Deserialize;
use tao::event_loop::EventLoopProxy;

use super::{ProtocolRequestEnvelope, UserEvent};

pub(crate) use discovery::ProtocolIdentity;

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const PROTOCOL_VERSION: u32 = 1;
const COMMAND_RESPONSE_TIMEOUT: Duration = Duration::from_secs(65);
const MAX_ACTIVE_CONNECTIONS: usize = 16;
const NOT_READY_RESPONSE: &[u8] = br#"{"ok":false,"error_code":"protocol_not_ready","message":"The command layer is not ready."}"#;

pub(super) struct ProtocolTransport {
    endpoint: PublishedEndpoint,
    stopping: Arc<AtomicBool>,
    proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,
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
        let proxy = Arc::new(Mutex::new(None));
        let worker_stopping = Arc::clone(&stopping);
        let worker_proxy = Arc::clone(&proxy);
        let expected_uid = peer::current_uid();
        let expected_token = endpoint.instance_token().to_string();
        let control = Arc::new(ExportControl::new(
            endpoint
                .identity(PROTOCOL_VERSION)
                .instance_id()
                .to_string(),
        ));
        let worker = thread::Builder::new()
            .name("markhola-protocol-transport".to_string())
            .spawn(move || {
                run_listener(
                    listener,
                    expected_uid,
                    expected_token,
                    control,
                    worker_proxy,
                    worker_stopping,
                )
            })
            .map_err(|error| format!("Failed to start automation transport: {error}"))?;

        Ok(Self {
            endpoint,
            stopping,
            proxy,
            worker: Some(worker),
        })
    }

    pub(super) fn attach_proxy(&self, proxy: EventLoopProxy<UserEvent>) {
        if let Ok(mut attached) = self.proxy.lock() {
            *attached = Some(proxy);
        }
    }

    pub(super) fn identity(&self) -> ProtocolIdentity {
        self.endpoint.identity(PROTOCOL_VERSION)
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
    control: Arc<ExportControl>,
    proxy: Arc<Mutex<Option<EventLoopProxy<UserEvent>>>>,
    stopping: Arc<AtomicBool>,
) {
    let active_connections = Arc::new(AtomicUsize::new(0));
    while !stopping.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                if active_connections
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                        (count < MAX_ACTIVE_CONNECTIONS).then_some(count + 1)
                    })
                    .is_err()
                {
                    continue;
                }
                let token = expected_token.clone();
                let proxy = Arc::clone(&proxy);
                let control = Arc::clone(&control);
                let connections = Arc::clone(&active_connections);
                if thread::Builder::new()
                    .name("markhola-protocol-connection".to_string())
                    .spawn(move || {
                        handle_connection(stream, expected_uid, &token, &control, &proxy);
                        connections.fetch_sub(1, Ordering::AcqRel);
                    })
                    .is_err()
                {
                    active_connections.fetch_sub(1, Ordering::AcqRel);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
}

fn handle_connection(
    mut stream: UnixStream,
    expected_uid: u32,
    expected_token: &str,
    control: &ExportControl,
    proxy: &Mutex<Option<EventLoopProxy<UserEvent>>>,
) {
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
    let queued_export = match register_queued_export(&payload) {
        Ok(request_id) => request_id,
        Err(response) => {
            let _ = stream.write_all(&response);
            let _ = stream.write_all(b"\n");
            return;
        }
    };
    if let Some(response) = control.handle(&payload) {
        let _ = stream.write_all(&response);
        let _ = stream.write_all(b"\n");
        return;
    }

    let response = dispatch_request(payload, proxy).unwrap_or_else(|| NOT_READY_RESPONSE.to_vec());
    if let Some(request_id) = queued_export {
        crate::export_service::finish_unresolved_export(&request_id);
    }
    let _ = stream.write_all(&response);
    let _ = stream.write_all(b"\n");
}

fn dispatch_request(
    payload: Vec<u8>,
    proxy: &Mutex<Option<EventLoopProxy<UserEvent>>>,
) -> Option<Vec<u8>> {
    let proxy = proxy.lock().ok()?.clone()?;
    let (response, receiver) = mpsc::sync_channel(1);
    proxy
        .send_event(UserEvent::ProtocolRequest(ProtocolRequestEnvelope {
            payload,
            response,
        }))
        .ok()?;
    receiver.recv_timeout(COMMAND_RESPONSE_TIMEOUT).ok()
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

#[derive(Deserialize)]
struct ExportEnvelope<'a> {
    request_id: &'a str,
    command: &'a str,
}

fn register_queued_export(payload: &[u8]) -> Result<Option<String>, Vec<u8>> {
    let Ok(envelope) = serde_json::from_slice::<ExportEnvelope<'_>>(payload) else {
        return Ok(None);
    };
    if matches!(
        envelope.command,
        "export_png" | "export_pdf" | "export_html"
    ) {
        if crate::export_service::register_queued_export(envelope.request_id) {
            Ok(Some(envelope.request_id.to_string()))
        } else {
            Err(serde_json::to_vec(&serde_json::json!({
                "request_id": envelope.request_id,
                "ok": false,
                "command": envelope.command,
                "error_code": "request_capacity_exceeded",
                "message": "The process-lifetime export request cache is full.",
            }))
            .unwrap_or_else(|_| br#"{"ok":false,"error_code":"internal_error"}"#.to_vec()))
        }
    } else {
        Ok(None)
    }
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

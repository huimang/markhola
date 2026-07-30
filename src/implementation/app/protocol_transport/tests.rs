use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::discovery::{EndpointPaths, EndpointRecord};
use super::{NOT_READY_RESPONSE, PROTOCOL_VERSION, ProtocolTransport};

fn temporary_paths(name: &str) -> (std::path::PathBuf, EndpointPaths) {
    let root = std::path::PathBuf::from("/tmp").join(format!("mhp-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let paths = EndpointPaths {
        runtime_directory: root.join("runtime"),
        discovery_directory: root.join("discovery"),
    };
    (root, paths)
}

#[test]
fn publishes_private_exact_instance_endpoint_and_cleans_up() {
    let (root, paths) = temporary_paths("lifecycle");
    let transport = ProtocolTransport::start_with_paths(paths.clone()).unwrap();
    let record_path = transport.record_path().to_path_buf();
    let socket_path = transport.socket_path().to_path_buf();
    let record: EndpointRecord = serde_json::from_slice(&fs::read(&record_path).unwrap()).unwrap();

    assert_eq!(record.protocol_version, PROTOCOL_VERSION);
    assert_eq!(record.pid, std::process::id());
    assert_eq!(record.instance_token.len(), 64);
    assert!(record.instance_id.starts_with(&format!("{}-", record.pid)));
    assert_eq!(record.socket_path, socket_path.to_string_lossy());
    assert_private(&paths.runtime_directory, 0o700);
    assert_private(&paths.discovery_directory, 0o700);
    assert_private(&record_path, 0o600);
    assert_private(&socket_path, 0o600);

    drop(transport);
    assert!(!record_path.exists());
    assert!(!socket_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn accepts_same_uid_unix_peer_without_exposing_commands() {
    let (root, paths) = temporary_paths("peer");
    let transport = ProtocolTransport::start_with_paths(paths).unwrap();
    let response = round_trip(&transport, &request_frame(&record_token(&transport)));
    assert_eq!(
        response,
        [NOT_READY_RESPONSE, b"\n"].concat(),
        "transport must not expose an unimplemented command surface"
    );

    drop(transport);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_missing_wrong_and_stale_tokens_but_accepts_current_token() {
    let (root, paths) = temporary_paths("token");
    let first = ProtocolTransport::start_with_paths(paths.clone()).unwrap();
    let stale_token = record_token(&first);
    drop(first);

    let transport = ProtocolTransport::start_with_paths(paths).unwrap();
    let current_token = record_token(&transport);
    assert_ne!(stale_token, current_token);
    assert!(round_trip(&transport, b"{\"command\":\"noop\"}\n").is_empty());
    assert!(
        round_trip(
            &transport,
            b"{\"instance_token\":\"wrong\",\"command\":\"noop\"}\n"
        )
        .is_empty()
    );
    assert!(
        round_trip(&transport, &request_frame(&stale_token)).is_empty(),
        "a token from a stopped instance must fail closed"
    );
    assert_eq!(
        round_trip(&transport, &request_frame(&current_token)),
        [NOT_READY_RESPONSE, b"\n"].concat()
    );

    drop(transport);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_eof_before_newline_and_any_trailing_bytes() {
    let (root, paths) = temporary_paths("newline-trailer");
    let transport = ProtocolTransport::start_with_paths(paths).unwrap();
    let token = record_token(&transport);
    let terminated = request_frame(&token);
    let unterminated = &terminated[..terminated.len() - 1];
    assert!(round_trip(&transport, unterminated).is_empty());

    let mut trailing_whitespace = terminated.clone();
    trailing_whitespace.extend_from_slice(b" ");
    assert!(round_trip(&transport, &trailing_whitespace).is_empty());

    let mut second_frame = terminated;
    second_frame.extend_from_slice(b"{}\n");
    assert!(round_trip(&transport, &second_frame).is_empty());

    drop(transport);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_empty_frame_without_response() {
    let (root, paths) = temporary_paths("empty-frame");
    let transport = ProtocolTransport::start_with_paths(paths).unwrap();
    let mut stream = UnixStream::connect(transport.socket_path()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream.write_all(b"\n").unwrap();
    let _ = stream.shutdown(std::net::Shutdown::Write);

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    assert!(response.is_empty());

    drop(transport);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_oversized_frames_without_response() {
    let (root, paths) = temporary_paths("oversized");
    let transport = ProtocolTransport::start_with_paths(paths).unwrap();
    let mut stream = UnixStream::connect(transport.socket_path()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let _ = stream.write_all(&vec![b'x'; super::MAX_REQUEST_BYTES + 1]);
    let _ = stream.shutdown(std::net::Shutdown::Write);

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    assert!(response.is_empty());

    drop(transport);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn current_user_paths_use_private_runtime_and_application_support_discovery() {
    let home = PathBuf::from(std::env::var("HOME").unwrap());
    let paths = EndpointPaths::for_current_user().unwrap();

    assert_eq!(
        paths.runtime_directory,
        std::env::temp_dir().join("markhola-automation")
    );
    assert_eq!(
        paths.discovery_directory,
        home.join("Library")
            .join("Application Support")
            .join("MarkHola")
            .join("automation")
            .join("endpoints")
    );
}

#[test]
fn publishing_endpoint_record_is_atomic_and_leaves_no_temporary_record() {
    let (root, paths) = temporary_paths("record-atomic");
    let transport = ProtocolTransport::start_with_paths(paths.clone()).unwrap();
    let record_path = transport.record_path().to_path_buf();
    let temporary = record_path.with_extension("json.tmp");

    assert!(record_path.exists());
    assert!(!temporary.exists());
    assert_private(&record_path, 0o600);

    drop(transport);
    let _ = fs::remove_dir_all(root);
}

fn record_token(transport: &ProtocolTransport) -> String {
    let record: EndpointRecord =
        serde_json::from_slice(&fs::read(transport.record_path()).unwrap()).unwrap();
    record.instance_token
}

fn request_frame(token: &str) -> Vec<u8> {
    format!(r#"{{"instance_token":"{token}","command":"noop"}}"#)
        .bytes()
        .chain(std::iter::once(b'\n'))
        .collect()
}

fn round_trip(transport: &ProtocolTransport, payload: &[u8]) -> Vec<u8> {
    let mut stream = UnixStream::connect(transport.socket_path()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let _ = stream.write_all(payload);
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    response
}

fn assert_private(path: &std::path::Path, mode: u32) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let metadata = fs::metadata(path).unwrap();
    assert_eq!(metadata.uid(), super::peer::current_uid());
    assert_eq!(metadata.permissions().mode() & 0o777, mode);
}

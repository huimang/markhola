use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
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
    let mut stream = UnixStream::connect(transport.socket_path()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream.write_all(b"{}\n").unwrap();

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
    assert_eq!(
        response,
        [NOT_READY_RESPONSE, b"\n"].concat(),
        "transport must not expose an unimplemented command surface"
    );

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

fn assert_private(path: &std::path::Path, mode: u32) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let metadata = fs::metadata(path).unwrap();
    assert_eq!(metadata.uid(), super::peer::current_uid());
    assert_eq!(metadata.permissions().mode() & 0o777, mode);
}

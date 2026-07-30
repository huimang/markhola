use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub(super) struct EndpointPaths {
    pub(super) runtime_directory: PathBuf,
    pub(super) discovery_directory: PathBuf,
}

impl EndpointPaths {
    pub(super) fn for_current_user() -> Result<Self, String> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or("HOME is unavailable for protocol discovery.")?;
        Ok(Self {
            runtime_directory: std::env::temp_dir().join("markhola-automation"),
            discovery_directory: home
                .join("Library")
                .join("Application Support")
                .join("MarkHola")
                .join("automation")
                .join("endpoints"),
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct EndpointRecord {
    pub(super) protocol_version: u32,
    pub(super) pid: u32,
    pub(super) instance_id: String,
    pub(super) instance_token: String,
    pub(super) socket_path: String,
}

pub(super) struct PublishedEndpoint {
    socket_path: PathBuf,
    record_path: PathBuf,
    instance_token: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ProtocolIdentity {
    protocol_version: u32,
    pid: u32,
    instance_id: String,
    instance_token: String,
    socket_path: String,
}

impl ProtocolIdentity {
    pub(crate) fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub(crate) fn pid(&self) -> u32 {
        self.pid
    }

    pub(crate) fn instance_id(&self) -> &str {
        &self.instance_id
    }

    #[allow(dead_code)]
    pub(super) fn instance_token(&self) -> &str {
        &self.instance_token
    }

    pub(crate) fn exact_instance_token(&self) -> &str {
        &self.instance_token
    }

    pub(crate) fn socket_path(&self) -> &str {
        &self.socket_path
    }

    #[cfg(test)]
    pub(crate) fn for_test(instance_id: &str, instance_token: &str) -> Self {
        Self {
            protocol_version: 1,
            pid: std::process::id(),
            instance_id: instance_id.to_string(),
            instance_token: instance_token.to_string(),
            socket_path: "/tmp/markhola-protocol-test.sock".to_string(),
        }
    }
}

impl PublishedEndpoint {
    pub(super) fn create(paths: EndpointPaths, protocol_version: u32) -> Result<Self, String> {
        create_private_directory(&paths.runtime_directory)?;
        create_private_directory(&paths.discovery_directory)?;

        let pid = std::process::id();
        let token = random_token()?;
        let instance_id = format!("{pid}-{}", &token[..16]);
        let socket_path = paths
            .runtime_directory
            .join(format!("mh-{pid}-{}.sock", &token[..8]));
        let record_path = paths
            .discovery_directory
            .join(format!("{instance_id}.json"));
        let record = EndpointRecord {
            protocol_version,
            pid,
            instance_id,
            instance_token: token.clone(),
            socket_path: socket_path.to_string_lossy().into_owned(),
        };
        write_record(&record_path, &record)?;

        Ok(Self {
            socket_path,
            record_path,
            instance_token: token,
        })
    }

    pub(crate) fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub(crate) fn instance_token(&self) -> &str {
        &self.instance_token
    }

    pub(super) fn identity(&self, protocol_version: u32) -> ProtocolIdentity {
        let instance_id = self
            .record_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        ProtocolIdentity {
            protocol_version,
            pid: std::process::id(),
            instance_id,
            instance_token: self.instance_token.clone(),
            socket_path: self.socket_path.to_string_lossy().into_owned(),
        }
    }

    #[cfg(test)]
    pub(super) fn record_path(&self) -> &Path {
        &self.record_path
    }
}

impl Drop for PublishedEndpoint {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.record_path);
        let _ = fs::remove_file(&self.socket_path);
    }
}

pub(super) fn set_private_mode(path: &Path, mode: u32) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|error| format!("Failed to secure {}: {error}", path.display()))?;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Failed to inspect {}: {error}", path.display()))?;
    if metadata.uid() != super::peer::current_uid() || metadata.mode() & 0o777 != mode {
        return Err(format!("Unsafe ownership or mode for {}.", path.display()));
    }
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|error| format!("Failed to create {}: {error}", path.display()))?;
    set_private_mode(path, 0o700)
}

fn write_record(path: &Path, record: &EndpointRecord) -> Result<(), String> {
    let bytes = serde_json::to_vec(record)
        .map_err(|error| format!("Failed to encode endpoint record: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .map_err(|error| format!("Failed to write endpoint record: {error}"))?;
    set_private_mode(&temporary, 0o600)?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("Failed to publish endpoint record: {error}"))?;
    set_private_mode(path, 0o600)
}

fn random_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    use std::io::Read;
    fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bytes))
        .map_err(|error| format!("Failed to generate instance token: {error}"))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

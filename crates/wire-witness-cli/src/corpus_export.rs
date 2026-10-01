//! Filesystem custody for completed policy-bounded corpus exports.
//!
//! Governed by spec 010, policy-bounded corpus export.

// region: policy-bounded-corpus-export-host

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use wire_witness_core::corpus_export::{CorpusExport, DIGEST_CONSTRUCTION};
use wire_witness_core::exchange::sha256_hex;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallReceipt {
    pub path: PathBuf,
    pub byte_length: u64,
    pub digest: String,
    pub digest_construction: &'static str,
}

#[derive(Debug)]
pub enum InstallError {
    InvalidPath,
    InvalidExport,
    AlreadyExists,
    TemporaryNameExhausted,
    Io(io::Error),
}

impl PartialEq for InstallError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidPath, Self::InvalidPath)
            | (Self::InvalidExport, Self::InvalidExport)
            | (Self::AlreadyExists, Self::AlreadyExists)
            | (Self::TemporaryNameExhausted, Self::TemporaryNameExhausted) => true,
            (Self::Io(left), Self::Io(right)) => left.kind() == right.kind(),
            _ => false,
        }
    }
}

impl Eq for InstallError {}

impl From<io::Error> for InstallError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Installs an already validated export through a complete private temporary
/// file and an exclusive atomic hard-link at the final path.
pub fn install_export(path: &Path, export: &CorpusExport) -> Result<InstallReceipt, InstallError> {
    if export.manifest.canonical_bytes() != export.manifest_bytes
        || sha256_hex(&export.manifest_bytes) != export.manifest_digest
    {
        return Err(InstallError::InvalidExport);
    }
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or(InstallError::InvalidPath)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && *name != "." && *name != "..")
        .ok_or(InstallError::InvalidPath)?;
    if path.symlink_metadata().is_ok() {
        return Err(InstallError::AlreadyExists);
    }

    let (temporary_path, mut temporary) = create_private_temporary(parent, file_name)?;
    let write_result = (|| {
        #[cfg(unix)]
        {
            temporary.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        temporary.write_all(&export.manifest_bytes)?;
        temporary.flush()?;
        temporary.sync_all()?;
        drop(temporary);
        fs::hard_link(&temporary_path, path).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                InstallError::AlreadyExists
            } else {
                InstallError::Io(error)
            }
        })?;
        fs::remove_file(&temporary_path).map_err(InstallError::Io)?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(InstallError::Io)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary_path);
        return write_result.map(|()| unreachable!());
    }
    Ok(InstallReceipt {
        path: path.to_owned(),
        byte_length: export.manifest_bytes.len() as u64,
        digest: export.manifest_digest.clone(),
        digest_construction: DIGEST_CONSTRUCTION,
    })
}

fn create_private_temporary(
    parent: &Path,
    file_name: &str,
) -> Result<(PathBuf, File), InstallError> {
    for _ in 0..128 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{file_name}.wire-witness.{}.{sequence}.tmp",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&candidate) {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(InstallError::Io(error)),
        }
    }
    Err(InstallError::TemporaryNameExhausted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use wire_witness_core::corpus_export::{
        CorpusExport, CorpusManifest, ExportLimits, PolicyReceipt, RequestedBinding,
    };
    use wire_witness_core::exchange::{Binding, sha256_hex};

    fn temporary_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "wire-witness-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    fn export() -> CorpusExport {
        let binding = RequestedBinding {
            binding: Binding::Unsupervised {
                session_id: "session-1".into(),
            },
            capture_digest: "a".repeat(64),
        };
        let receipt = PolicyReceipt {
            decision_schema: "policy.test/1".into(),
            decision_id: "decision-1".into(),
            decision_digest: format!("sha256:{}", "b".repeat(64)),
            decided_at: "2026-10-01T00:00:00Z".into(),
            expires_at: "2026-10-02T00:00:00Z".into(),
            outcome: "allow".into(),
            asserted_principal: "owner:test".into(),
            verifier: "verifier:test".into(),
            verification_outcome: "verified".into(),
            scope_bindings: vec![binding.clone()],
            source_kinds: vec![],
            materialization_modes: vec![],
            digest: "c".repeat(64),
        };
        let manifest = CorpusManifest {
            export_id: "export-1".into(),
            created_at: "2026-10-01T00:00:00Z".into(),
            expires_at: "2026-10-02T00:00:00Z".into(),
            producer_identity: "wire-witness:test".into(),
            policy_receipt: receipt,
            bindings: vec![binding],
            sources: vec![],
            artifacts: vec![],
            gaps: vec![],
            complete: true,
            limits: ExportLimits {
                max_bindings: 1,
                max_entries: 1,
                max_artifacts: 1,
                max_artifact_bytes: 1000,
                max_total_bytes: 10_000,
            },
        };
        let manifest_bytes = manifest.canonical_bytes();
        CorpusExport {
            manifest,
            manifest_digest: sha256_hex(&manifest_bytes),
            manifest_bytes,
        }
    }

    #[test]
    fn installs_complete_private_file_and_refuses_overwrite() {
        let directory = temporary_directory("install");
        let path = directory.join("corpus.json");
        let export = export();
        let receipt = install_export(&path, &export).unwrap();
        assert_eq!(fs::read(&path).unwrap(), export.manifest_bytes);
        assert_eq!(receipt.digest, export.manifest_digest);
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            install_export(&path, &export).unwrap_err(),
            InstallError::AlreadyExists
        );
        assert_eq!(fs::read(&path).unwrap(), export.manifest_bytes);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn invalid_final_path_creates_nothing() {
        let directory = temporary_directory("invalid");
        assert_eq!(
            install_export(&directory, &export()).unwrap_err(),
            InstallError::AlreadyExists
        );
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn divergent_manifest_bytes_refuse_before_file_creation() {
        let directory = temporary_directory("divergent");
        let path = directory.join("corpus.json");
        let mut export = export();
        export.manifest_bytes.push(b' ');
        assert_eq!(
            install_export(&path, &export).unwrap_err(),
            InstallError::InvalidExport
        );
        assert!(!path.exists());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        fs::remove_dir_all(directory).unwrap();
    }
}

// endregion

//! Host-side acquisition for predeclared instruction observations.
//!
//! Governed by spec 008, instruction delivery observation.

// region: instruction-delivery-observation

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use wire_witness_core::instruction_observation::{ComparisonPlan, ComponentSelector, PlanError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionProbeSource {
    pub probe_name: String,
    pub target_path: PathBuf,
    pub target_digest: String,
    pub selector: ComponentSelector,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProbeLoadError {
    DuplicateProbeName,
    TargetPathNotAbsolute,
    TargetUnavailable,
    TargetNotFile,
    InvalidPlan(PlanError),
}

/// Loads every explicitly named target before a child is started.
///
/// Callers keep the returned plans alive for the attempt. Each plan clears its
/// transient target buffer when dropped. Errors never include target paths or
/// target bytes.
pub fn load_plans(
    sources: &[InstructionProbeSource],
) -> Result<Vec<ComparisonPlan>, ProbeLoadError> {
    let mut names = BTreeSet::new();
    for source in sources {
        if !names.insert(source.probe_name.as_str()) {
            return Err(ProbeLoadError::DuplicateProbeName);
        }
        if !source.target_path.is_absolute()
            || source
                .target_path
                .components()
                .any(|component| component.as_os_str() == "..")
        {
            return Err(ProbeLoadError::TargetPathNotAbsolute);
        }
    }

    sources
        .iter()
        .map(|source| {
            let mut file =
                File::open(&source.target_path).map_err(|_| ProbeLoadError::TargetUnavailable)?;
            let metadata = file
                .metadata()
                .map_err(|_| ProbeLoadError::TargetUnavailable)?;
            if !metadata.is_file() {
                return Err(ProbeLoadError::TargetNotFile);
            }
            let mut target = Vec::new();
            file.read_to_end(&mut target)
                .map_err(|_| ProbeLoadError::TargetUnavailable)?;
            ComparisonPlan::new(
                source.probe_name.clone(),
                target,
                &source.target_digest,
                source.selector.clone(),
            )
            .map_err(ProbeLoadError::InvalidPlan)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use wire_witness_core::exchange::{ProviderFamily, sha256_hex};
    use wire_witness_core::instruction_observation::ComponentKind;

    use super::*;

    fn selector() -> ComponentSelector {
        ComponentSelector {
            provider: ProviderFamily::Anthropic,
            operation: "POST /v1/messages".into(),
            kind: ComponentKind::System,
            component_index: 0,
            text_part_index: 0,
        }
    }

    #[test]
    fn target_is_loaded_and_bound_before_use() {
        let directory = std::env::temp_dir().join(format!(
            "wire-witness-instruction-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("instruction.txt");
        fs::write(&path, b"exact instruction").unwrap();
        let plans = load_plans(&[InstructionProbeSource {
            probe_name: "primary".into(),
            target_path: path.clone(),
            target_digest: sha256_hex(b"exact instruction"),
            selector: selector(),
        }])
        .unwrap();
        assert_eq!(plans[0].probe_name(), "primary");
        assert_eq!(plans[0].target_digest(), sha256_hex(b"exact instruction"));
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn duplicate_names_refuse_before_any_file_read() {
        let missing = PathBuf::from("/definitely/not/present");
        let source = InstructionProbeSource {
            probe_name: "duplicate".into(),
            target_path: missing,
            target_digest: sha256_hex(b"target"),
            selector: selector(),
        };
        assert_eq!(
            load_plans(&[source.clone(), source]),
            Err(ProbeLoadError::DuplicateProbeName)
        );
    }

    #[test]
    fn relative_paths_and_digest_mismatches_refuse() {
        let relative = InstructionProbeSource {
            probe_name: "relative".into(),
            target_path: PathBuf::from("instruction.txt"),
            target_digest: sha256_hex(b"target"),
            selector: selector(),
        };
        assert_eq!(
            load_plans(&[relative]),
            Err(ProbeLoadError::TargetPathNotAbsolute)
        );

        let directory = std::env::temp_dir().join(format!(
            "wire-witness-instruction-digest-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("instruction.txt");
        fs::write(&path, b"target").unwrap();
        let result = load_plans(&[InstructionProbeSource {
            probe_name: "digest".into(),
            target_path: path.clone(),
            target_digest: "wrong".into(),
            selector: selector(),
        }]);
        assert_eq!(
            result,
            Err(ProbeLoadError::InvalidPlan(PlanError::DigestMismatch))
        );
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}

// endregion

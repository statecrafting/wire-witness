//! The producer identity a supervisor pins: version, executable digest,
//! interface, and schema versions.
//!
//! Governed by spec 014, the wire-witness.sidecar/1 stdio host.

// region: producer-identity

use std::path::Path;

use wire_witness_core::exchange::{JsonValue, canonical_json_bytes, sha256_hex};

use crate::sidecar_protocol::{EVIDENCE_TYPE, PROTOCOL};

pub const PRODUCER_NAME: &str = "wire-witness";
pub const PRODUCER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ATTEMPT_BINDING_SCHEMA: u64 = 1;
pub const TERMINAL_SCHEMA: u64 = 1;
pub const EVIDENCE_SCHEMA: u64 = 1;
pub const APPLICATION_PROTOCOL: &str = "http/1.1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseChannel {
    Release,
    Development,
}

impl ReleaseChannel {
    /// `release` only when the build set `WIRE_WITNESS_RELEASE_CHANNEL=release`.
    #[must_use]
    pub fn of_this_build() -> Self {
        match option_env!("WIRE_WITNESS_RELEASE_CHANNEL") {
            Some("release") => Self::Release,
            _ => Self::Development,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Development => "development",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProducerIdentity {
    pub version: &'static str,
    pub release_channel: ReleaseChannel,
    /// `sha256:<hex>` of the executable bytes, or `None` when they could not
    /// be read.
    pub executable_digest: Option<String>,
}

impl ProducerIdentity {
    /// The identity of the running executable.
    #[must_use]
    pub fn current() -> Self {
        Self::for_executable(std::env::current_exe().ok().as_deref())
    }

    /// The identity of this build with the digest of the executable at
    /// `path`. An unreadable or missing path leaves the digest unknown.
    #[must_use]
    pub fn for_executable(path: Option<&Path>) -> Self {
        Self {
            version: PRODUCER_VERSION,
            release_channel: ReleaseChannel::of_this_build(),
            executable_digest: path
                .and_then(|path| std::fs::read(path).ok())
                .map(|bytes| format!("sha256:{}", sha256_hex(&bytes))),
        }
    }

    fn digest_json(&self) -> JsonValue {
        self.executable_digest
            .as_deref()
            .map_or_else(|| object([("state", string("unknown"))]), string)
    }

    /// The full identity document.
    #[must_use]
    pub fn document(&self) -> JsonValue {
        object([
            (
                "applicationProtocols",
                JsonValue::Array(vec![string(APPLICATION_PROTOCOL)]),
            ),
            ("attemptBindingSchema", number(ATTEMPT_BINDING_SCHEMA)),
            (
                "evidence",
                JsonValue::Array(vec![object([
                    ("schemaVersion", number(EVIDENCE_SCHEMA)),
                    ("type", string(EVIDENCE_TYPE)),
                ])]),
            ),
            ("executableDigest", self.digest_json()),
            ("interface", string(PROTOCOL)),
            ("name", string(PRODUCER_NAME)),
            ("releaseChannel", string(self.release_channel.as_str())),
            ("terminalSchema", number(TERMINAL_SCHEMA)),
            ("version", string(self.version)),
        ])
    }

    /// The subset repeated in `ready` and `finished`.
    #[must_use]
    pub fn message_subset(&self) -> JsonValue {
        object([
            ("attemptBindingSchema", number(ATTEMPT_BINDING_SCHEMA)),
            ("executableDigest", self.digest_json()),
            ("interface", string(PROTOCOL)),
            ("name", string(PRODUCER_NAME)),
            ("releaseChannel", string(self.release_channel.as_str())),
            ("terminalSchema", number(TERMINAL_SCHEMA)),
            ("version", string(self.version)),
        ])
    }

    /// The `{ name, revision, contentDigest }` member of the evidence
    /// manifest.
    #[must_use]
    pub fn manifest_producer(&self) -> JsonValue {
        object([
            ("contentDigest", self.digest_json()),
            ("name", string(PRODUCER_NAME)),
            ("revision", string(self.version)),
        ])
    }

    #[must_use]
    pub fn json_bytes(&self) -> Vec<u8> {
        canonical_json_bytes(&self.document())
    }

    #[must_use]
    pub fn human(&self) -> String {
        format!(
            "name: {PRODUCER_NAME}\nversion: {}\nrelease channel: {}\nexecutable digest: {}\ninterface: {PROTOCOL}\nattempt binding schema: {ATTEMPT_BINDING_SCHEMA}\nterminal schema: {TERMINAL_SCHEMA}\nevidence: {EVIDENCE_TYPE} schema {EVIDENCE_SCHEMA}\napplication protocols: {APPLICATION_PROTOCOL}\n",
            self.version,
            self.release_channel.as_str(),
            self.executable_digest.as_deref().unwrap_or("unknown"),
        )
    }
}

fn object<const N: usize>(entries: [(&str, JsonValue); N]) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect(),
    )
}

fn string(value: &str) -> JsonValue {
    JsonValue::String(value.to_owned())
}

fn number(value: u64) -> JsonValue {
    JsonValue::Number(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn members(value: &JsonValue) -> Vec<&str> {
        let JsonValue::Object(fields) = value else {
            panic!("identity is an object");
        };
        fields.keys().map(String::as_str).collect()
    }

    #[test]
    fn identity_member_set_is_exact() {
        let identity = ProducerIdentity::for_executable(None);
        assert_eq!(
            members(&identity.document()),
            [
                "applicationProtocols",
                "attemptBindingSchema",
                "evidence",
                "executableDigest",
                "interface",
                "name",
                "releaseChannel",
                "terminalSchema",
                "version",
            ]
        );
        assert_eq!(
            members(&identity.message_subset()),
            [
                "attemptBindingSchema",
                "executableDigest",
                "interface",
                "name",
                "releaseChannel",
                "terminalSchema",
                "version",
            ]
        );
    }

    #[test]
    fn unreadable_executable_is_unknown_never_guessed() {
        let identity = ProducerIdentity::for_executable(Some(Path::new(
            "/nonexistent/wire-witness-identity-probe",
        )));
        assert_eq!(identity.executable_digest, None);
        let text = String::from_utf8(identity.json_bytes()).unwrap();
        assert!(text.contains(r#""executableDigest":{"state":"unknown"}"#));
    }

    #[test]
    fn digest_names_the_exact_bytes() {
        let path = std::env::current_exe().unwrap();
        let identity = ProducerIdentity::for_executable(Some(&path));
        assert_eq!(
            identity.executable_digest,
            Some(format!(
                "sha256:{}",
                sha256_hex(&std::fs::read(&path).unwrap())
            ))
        );
    }

    #[test]
    fn this_build_is_not_presented_as_released_without_the_channel() {
        if option_env!("WIRE_WITNESS_RELEASE_CHANNEL") != Some("release") {
            assert_eq!(ReleaseChannel::of_this_build(), ReleaseChannel::Development);
        }
    }
}

// endregion

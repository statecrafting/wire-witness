//! Standalone and supervised host boundary.
//!
//! Specs 005 and 006 own the sidecar protocol and standalone process host.

#![forbid(unsafe_code)]

// region: instruction-observation-module-export
pub mod instruction_observation;
// endregion
pub mod sidecar_protocol;
pub mod standalone_host;

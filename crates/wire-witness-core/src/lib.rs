//! Pure testimony types and transformations.
//!
//! This crate deliberately performs no I/O and reads no clock. Hosts supply
//! observed bytes and all identities explicitly.

#![forbid(unsafe_code)]

pub mod custody;
// region: corpus-export-module-export
pub mod corpus_export;
// endregion
pub mod exchange;
// region: instruction-observation-module-export
pub mod instruction_observation;
// endregion
// region: measurement-summary-module-export
pub mod measurement_summary;
// endregion

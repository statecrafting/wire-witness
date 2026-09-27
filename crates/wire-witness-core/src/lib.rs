//! Pure testimony types and transformations.
//!
//! This crate deliberately performs no I/O and reads no clock. Hosts supply
//! observed bytes and all identities explicitly.

#![forbid(unsafe_code)]

pub mod custody;
pub mod exchange;

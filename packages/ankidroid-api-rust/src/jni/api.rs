//! Note: The main AnkiDroid API has been moved to src/api.rs
//!
//! This file has been relocated to provide a cleaner module structure.
//! Please use `crate::api::AnkiDroidApi` instead.

#[cfg(target_os = "android")]
pub use crate::api::*;
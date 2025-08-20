//! JNI interface for Android integration

#[cfg(target_os = "android")]
pub mod helpers;

#[cfg(target_os = "android")]
pub mod utils;

#[cfg(target_os = "android")]
pub mod content_resolver;

#[cfg(target_os = "android")]
pub mod cursor;

#[cfg(target_os = "android")]
pub use helpers::*;

#[cfg(target_os = "android")]
pub use utils::*;

#[cfg(target_os = "android")]
pub use content_resolver::*;

#[cfg(target_os = "android")]
pub use cursor::*;
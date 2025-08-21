//! Error handling for the AnkiDroid API Rust package
//!
//! This module provides comprehensive error handling for all AnkiDroid API operations,
//! including JNI errors, Android-specific errors, and application-level errors.

use thiserror::Error;

/// Main error type for AnkiDroid API operations
#[derive(Error, Debug)]
pub enum AnkiDroidError {
    /// AnkiDroid app is not available or installed
    #[error("AnkiDroid not available: {0}")]
    AnkiDroidNotAvailable(String),

    /// Permission denied for AnkiDroid API access
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid model ID provided
    #[error("Invalid model ID: {0}")]
    InvalidModelId(i64),

    /// Invalid deck ID provided
    #[error("Invalid deck ID: {0}")]
    InvalidDeckId(i64),

    /// Duplicate note detected
    #[error("Duplicate note: {0}")]
    DuplicateNote(String),

    /// Field count mismatch between provided fields and model
    #[error("Field count mismatch: expected {expected}, got {actual}")]
    FieldCountMismatch { expected: usize, actual: usize },

    /// JNI operation failed
    #[error("JNI error: {0}")]
    JniError(String),

    /// Null pointer encountered
    #[error("Null pointer: {0}")]
    NullPointer(String),

    /// String conversion failed
    #[error("String conversion error: {0}")]
    StringConversionError(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Validation failed
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// I/O operation failed
    #[error("I/O error: {0}")]
    IoError(String),

    /// JSON serialization/deserialization failed
    #[error("JSON error: {0}")]
    JsonError(String),
}

impl AnkiDroidError {
    /// Create a new AnkiDroid not available error
    pub fn not_available(msg: impl Into<String>) -> Self {
        Self::AnkiDroidNotAvailable(msg.into())
    }

    /// Create a new permission denied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create a new invalid model ID error
    pub fn invalid_model_id(id: i64) -> Self {
        Self::InvalidModelId(id)
    }

    /// Create a new invalid deck ID error
    pub fn invalid_deck_id(id: i64) -> Self {
        Self::InvalidDeckId(id)
    }

    /// Create a new duplicate note error
    pub fn duplicate_note(msg: impl Into<String>) -> Self {
        Self::DuplicateNote(msg.into())
    }

    /// Create a new field count mismatch error
    pub fn field_count_mismatch(expected: usize, actual: usize) -> Self {
        Self::FieldCountMismatch { expected, actual }
    }

    /// Create a new JNI error
    pub fn jni_error(msg: impl Into<String>) -> Self {
        Self::JniError(msg.into())
    }

    /// Create a new null pointer error
    pub fn null_pointer(msg: impl Into<String>) -> Self {
        Self::NullPointer(msg.into())
    }

    /// Create a new string conversion error
    pub fn string_conversion_error(msg: impl Into<String>) -> Self {
        Self::StringConversionError(msg.into())
    }

    /// Create a new database error
    pub fn database_error(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }

    /// Create a new validation error
    pub fn validation_error(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create a new I/O error
    pub fn io_error(msg: impl Into<String>) -> Self {
        Self::IoError(msg.into())
    }

    /// Create a new JSON error
    pub fn json_error(msg: impl Into<String>) -> Self {
        Self::JsonError(msg.into())
    }

    /// Check if this error is due to AnkiDroid not being available
    pub fn is_ankidroid_unavailable(&self) -> bool {
        matches!(self, Self::AnkiDroidNotAvailable(_))
    }

    /// Check if this error is due to permission denial
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied(_))
    }

    /// Check if this error is due to an invalid ID
    pub fn is_invalid_id(&self) -> bool {
        matches!(self, Self::InvalidModelId(_) | Self::InvalidDeckId(_))
    }

    /// Check if this error is due to JNI operations
    pub fn is_jni_error(&self) -> bool {
        matches!(self, Self::JniError(_) | Self::NullPointer(_))
    }

    /// Check if this error is recoverable (can be retried)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::DatabaseError(_) | Self::IoError(_) | Self::JniError(_)
        )
    }

    /// Get the error category for logging/monitoring purposes
    pub fn category(&self) -> &'static str {
        match self {
            Self::AnkiDroidNotAvailable(_) => "availability",
            Self::PermissionDenied(_) => "permission",
            Self::InvalidModelId(_) | Self::InvalidDeckId(_) => "validation",
            Self::DuplicateNote(_) => "duplicate",
            Self::FieldCountMismatch { .. } => "validation",
            Self::JniError(_) | Self::NullPointer(_) => "jni",
            Self::StringConversionError(_) => "conversion",
            Self::DatabaseError(_) => "database",
            Self::ValidationError(_) => "validation",
            Self::IoError(_) => "io",
            Self::JsonError(_) => "serialization",
        }
    }
}

// From trait implementations for automatic error conversion

#[cfg(target_os = "android")]
impl From<jni::errors::Error> for AnkiDroidError {
    fn from(err: jni::errors::Error) -> Self {
        Self::JniError(err.to_string())
    }
}

impl From<std::io::Error> for AnkiDroidError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for AnkiDroidError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for AnkiDroidError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::StringConversionError(format!("UTF-8 conversion failed: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for AnkiDroidError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::StringConversionError(format!("UTF-8 conversion failed: {}", err))
    }
}

/// Type alias for Results using AnkiDroidError
pub type Result<T> = std::result::Result<T, AnkiDroidError>;

/// Alias for the main error type to match lib.rs expectations
pub type Error = AnkiDroidError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation_helpers() {
        let err = AnkiDroidError::not_available("test");
        assert!(matches!(err, AnkiDroidError::AnkiDroidNotAvailable(_)));
        assert!(err.is_ankidroid_unavailable());

        let err = AnkiDroidError::permission_denied("test");
        assert!(matches!(err, AnkiDroidError::PermissionDenied(_)));
        assert!(err.is_permission_denied());

        let err = AnkiDroidError::invalid_model_id(123);
        assert!(matches!(err, AnkiDroidError::InvalidModelId(123)));
        assert!(err.is_invalid_id());

        let err = AnkiDroidError::field_count_mismatch(3, 2);
        assert!(matches!(
            err,
            AnkiDroidError::FieldCountMismatch {
                expected: 3,
                actual: 2
            }
        ));
    }

    #[test]
    fn test_error_categorization() {
        let err = AnkiDroidError::database_error("test");
        assert_eq!(err.category(), "database");
        assert!(err.is_recoverable());

        let err = AnkiDroidError::permission_denied("test");
        assert_eq!(err.category(), "permission");
        assert!(!err.is_recoverable());

        let err = AnkiDroidError::jni_error("test");
        assert_eq!(err.category(), "jni");
        assert!(err.is_jni_error());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = AnkiDroidError::field_count_mismatch(3, 2);
        assert_eq!(
            err.to_string(),
            "Field count mismatch: expected 3, got 2"
        );

        let err = AnkiDroidError::invalid_model_id(123);
        assert_eq!(err.to_string(), "Invalid model ID: 123");

        let err = AnkiDroidError::duplicate_note("checksum match");
        assert_eq!(err.to_string(), "Duplicate note: checksum match");
    }

    #[test]
    fn test_from_conversions() {
        // Test JNI error conversion (Android only)
        #[cfg(target_os = "android")]
        {
            let jni_err = jni::errors::Error::WrongJValueType("test", "test");
            let converted: AnkiDroidError = jni_err.into();
            assert!(matches!(converted, AnkiDroidError::JniError(_)));
        }

        // Test JSON error conversion
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_err.is_err());
        let converted: AnkiDroidError = json_err.unwrap_err().into();
        assert!(matches!(converted, AnkiDroidError::JsonError(_)));

        // Test UTF-8 error conversion
        let utf8_err = std::str::from_utf8(&[0xFF, 0xFE]);
        assert!(utf8_err.is_err());
        let converted: AnkiDroidError = utf8_err.unwrap_err().into();
        assert!(matches!(converted, AnkiDroidError::StringConversionError(_)));
    }

    #[test]
    fn test_result_type() {
        fn example_function() -> Result<i32> {
            Err(AnkiDroidError::validation_error("test"))
        }

        let result = example_function();
        assert!(result.is_err());
        match result {
            Err(AnkiDroidError::ValidationError(msg)) => assert_eq!(msg, "test"),
            _ => panic!("Wrong error type"),
        }
    }
}
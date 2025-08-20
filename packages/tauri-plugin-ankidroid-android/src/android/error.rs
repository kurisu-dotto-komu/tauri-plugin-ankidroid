use thiserror::Error;

/// Custom error types for AnkiDroid plugin operations
#[derive(Error, Debug)]
pub enum AndroidError {
    #[error("JNI error: {0}")]
    JniError(#[from] jni::errors::Error),

    #[error("Java exception occurred: {0}")]
    JavaException(String),

    #[error("ContentProvider error: {0}")]
    ContentProviderError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("AnkiDroid not installed")]
    AnkiDroidNotInstalled,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Deck not found: {0}")]
    DeckNotFound(String),

    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("Invalid field format: {0}")]
    InvalidFieldFormat(String),

    #[error("Resource cleanup error: {0}")]
    ResourceCleanupError(String),

    #[error("Cursor error: {0}")]
    CursorError(String),

    #[error("String conversion error: {0}")]
    StringConversionError(String),

    #[error("Local reference limit exceeded")]
    LocalReferenceLimitExceeded,
}

impl From<AndroidError> for String {
    fn from(err: AndroidError) -> String {
        err.to_string()
    }
}

impl AndroidError {
    /// Check if the error is due to permission denial
    pub fn is_permission_error(&self) -> bool {
        matches!(self, AndroidError::PermissionDenied(_))
    }

    /// Check if the error is due to AnkiDroid not being installed
    pub fn is_installation_error(&self) -> bool {
        matches!(self, AndroidError::AnkiDroidNotInstalled)
    }

    /// Check if the error is a Java exception
    pub fn is_java_exception(&self) -> bool {
        matches!(self, AndroidError::JavaException(_))
    }

    /// Create a permission denied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        AndroidError::PermissionDenied(msg.into())
    }

    /// Create a Java exception error
    pub fn java_exception(msg: impl Into<String>) -> Self {
        AndroidError::JavaException(msg.into())
    }

    /// Create a database error
    pub fn database_error(msg: impl Into<String>) -> Self {
        AndroidError::DatabaseError(msg.into())
    }

    /// Create a validation error
    pub fn validation_error(msg: impl Into<String>) -> Self {
        AndroidError::ValidationError(msg.into())
    }

    /// Create a model not found error
    pub fn model_not_found(model_name: impl Into<String>) -> Self {
        AndroidError::ModelNotFound(model_name.into())
    }

    /// Create a deck not found error
    pub fn deck_not_found(deck_name: impl Into<String>) -> Self {
        AndroidError::DeckNotFound(deck_name.into())
    }

    /// Create a cursor error
    pub fn cursor_error(msg: impl Into<String>) -> Self {
        AndroidError::CursorError(msg.into())
    }
}

/// Result type alias for Android operations
pub type AndroidResult<T> = Result<T, AndroidError>;

/// Helper trait for converting JNI results with exception checking
pub trait JniResultExt<T> {
    /// Convert JNI result and check for Java exceptions
    fn check_exception(self, env: &mut jni::JNIEnv) -> AndroidResult<T>;
}

impl<T> JniResultExt<T> for jni::errors::Result<T> {
    fn check_exception(self, env: &mut jni::JNIEnv) -> AndroidResult<T> {
        match self {
            Ok(value) => {
                // Check if a Java exception occurred
                if env.exception_check().unwrap_or(false) {
                    let exception_msg = get_exception_message(env)
                        .unwrap_or_else(|| "Unknown Java exception".to_string());
                    env.exception_clear().ok();
                    Err(AndroidError::java_exception(exception_msg))
                } else {
                    Ok(value)
                }
            }
            Err(jni_error) => Err(AndroidError::from(jni_error)),
        }
    }
}

/// Helper function to extract exception message from JNI environment
fn get_exception_message(env: &mut jni::JNIEnv) -> Option<String> {
    env.exception_occurred().ok().and_then(|exception| {
        env.call_method(&exception, "getMessage", "()Ljava/lang/String;", &[])
            .ok()
            .and_then(|msg| msg.l().ok())
            .and_then(|msg_obj| {
                if !msg_obj.is_null() {
                    env.get_string(&msg_obj.into())
                        .ok()
                        .map(|s| s.to_str().unwrap_or("").to_string())
                } else {
                    None
                }
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AndroidError::permission_denied("Test permission error");
        assert!(err.is_permission_error());
        assert!(err.to_string().contains("Permission denied"));
    }

    #[test]
    fn test_error_conversion_to_string() {
        let err = AndroidError::database_error("Test database error");
        let error_string: String = err.into();
        assert!(error_string.contains("Database error"));
    }

    #[test]
    fn test_model_not_found_error() {
        let err = AndroidError::model_not_found("Basic");
        assert!(err.to_string().contains("Model not found: Basic"));
    }

    #[test]
    fn test_deck_not_found_error() {
        let err = AndroidError::deck_not_found("Test Deck");
        assert!(err.to_string().contains("Deck not found: Test Deck"));
    }

    #[test]
    fn test_validation_error() {
        let err = AndroidError::validation_error("Invalid input");
        assert!(err.to_string().contains("Validation error: Invalid input"));
    }

    #[test]
    fn test_error_types() {
        let permission_err = AndroidError::permission_denied("test");
        let installation_err = AndroidError::AnkiDroidNotInstalled;
        let java_err = AndroidError::java_exception("test");

        assert!(permission_err.is_permission_error());
        assert!(!permission_err.is_installation_error());
        assert!(!permission_err.is_java_exception());

        assert!(!installation_err.is_permission_error());
        assert!(installation_err.is_installation_error());
        assert!(!installation_err.is_java_exception());

        assert!(!java_err.is_permission_error());
        assert!(!java_err.is_installation_error());
        assert!(java_err.is_java_exception());
    }
}

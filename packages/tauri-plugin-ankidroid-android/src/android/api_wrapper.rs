use ankidroid_api_rust::{AnkiDroidApi, AnkiDroidError};
use jni::{objects::JObject, JavaVM};
use std::ops::{Deref, DerefMut};

/// Get an AnkiDroid API instance using a callback to avoid lifetime issues
/// The callback receives the API instance and should perform all operations within it
pub fn with_api_instance<F, R>(callback: F) -> Result<R, String>
where
    F: for<'local> FnOnce(&mut AnkiDroidApi<'local>) -> Result<R, String>,
{
    // Get Android context from ndk_context
    let ctx = ndk_context::android_context();
    
    // Attach current thread to JVM
    let vm = unsafe { JavaVM::from_raw(ctx.vm() as _) }
        .map_err(|e| format!("Failed to create JavaVM: {}", e))?;
    let mut env_guard = vm.attach_current_thread()
        .map_err(|e| format!("Failed to attach thread: {}", e))?;

    // Create JObject from raw context pointer within the proper lifetime
    let context = unsafe { 
        JObject::from_raw(ctx.context() as jni::sys::jobject) 
    };
    
    // Get a mutable reference to the JNIEnv from the guard
    let env = env_guard.deref_mut();
    
    // Create the API instance with proper lifetimes
    // Clone the JNIEnv to get an owned value
    let env_owned = unsafe { std::ptr::read(env as *const _) };
    let mut api = AnkiDroidApi::try_new(env_owned, &context)
        .map_err(|e| format!("Failed to initialize AnkiDroid API: {}", e))?;

    // Execute the callback with the API instance
    callback(&mut api)
}

/// Legacy function that returns individual components (deprecated)
/// This is kept for backward compatibility but has lifetime issues
pub fn get_api_instance() -> Result<Box<dyn Fn() -> Result<(), String>>, String> {
    Err("This function is deprecated due to lifetime issues. Use with_api_instance instead.".to_string())
}

/// Convert AnkiDroidError to a user-friendly error message
pub fn format_error(error: AnkiDroidError) -> String {
    match error {
        AnkiDroidError::AnkiDroidNotAvailable(msg) => format!("AnkiDroid is not available: {}", msg),
        AnkiDroidError::PermissionDenied(msg) => format!("Permission denied: {}", msg),
        AnkiDroidError::InvalidModelId(id) => format!("Invalid model ID: {}", id),
        AnkiDroidError::InvalidDeckId(id) => format!("Invalid deck ID: {}", id),
        AnkiDroidError::DuplicateNote(msg) => format!("Duplicate note: {}", msg),
        AnkiDroidError::FieldCountMismatch { expected, actual } => format!("Field count mismatch: expected {}, got {}", expected, actual),
        AnkiDroidError::JniError(msg) => format!("JNI error: {}", msg),
        AnkiDroidError::NullPointer(msg) => format!("Null pointer: {}", msg),
        AnkiDroidError::StringConversionError(msg) => format!("String conversion error: {}", msg),
        AnkiDroidError::DatabaseError(msg) => format!("Database error: {}", msg),
        AnkiDroidError::ValidationError(msg) => format!("Validation error: {}", msg),
        AnkiDroidError::IoError(msg) => format!("I/O error: {}", msg),
        AnkiDroidError::JsonError(msg) => format!("JSON error: {}", msg),
    }
}
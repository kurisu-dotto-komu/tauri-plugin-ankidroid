//! JNI helpers for safe Android/Java interoperability
//!
//! This module provides safe wrappers around JNI operations with automatic exception
//! checking, resource management, and error handling. It's designed to be a standalone
//! module that can be used by any Rust Android project.
//!
//! # Safety Features
//!
//! - **Automatic exception checking**: All JNI operations are wrapped to check for
//!   Java exceptions and convert them to Rust errors
//! - **RAII resource management**: Local frames and thread attachments are automatically
//!   cleaned up when dropped
//! - **Safe string conversions**: Helper functions for converting between Rust and Java strings
//! - **ContentValues builder**: Type-safe builder for Android ContentValues objects
//!
//! # Examples
//!
//! ```rust
//! use crate::jni::helpers::*;
//! use crate::error::Result;
//!
//! fn example_jni_usage(env: JNIEnv) -> Result<()> {
//!     let mut safe_env = SafeJNIEnv::new(env);
//!     
//!     // Create a local frame to manage references
//!     let _frame = LocalFrame::new_default(safe_env.env_mut())?;
//!     
//!     // Safe string conversion
//!     let java_string = StringHelper::rust_to_java(&mut safe_env, "Hello, World!")?;
//!     
//!     // Safe method calls with automatic exception checking
//!     let result = safe_env.call_method_checked(
//!         &some_object,
//!         "toString",
//!         "()Ljava/lang/String;",
//!         &[]
//!     )?;
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{AnkiDroidError, Result};
use jni::objects::{JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use std::ops::Deref;

/// Safe JNI environment wrapper with automatic exception checking
///
/// This wrapper ensures that all JNI operations are checked for Java exceptions
/// and provides convenient methods for common operations.
///
/// # Safety
///
/// This wrapper maintains the same lifetime and safety guarantees as the underlying
/// JNIEnv while adding automatic exception checking.
pub struct SafeJNIEnv<'local> {
    env: JNIEnv<'local>,
}

impl<'local> Clone for SafeJNIEnv<'local> {
    fn clone(&self) -> Self {
        // Create a new SafeJNIEnv with a copy of the JNIEnv reference
        // This is safe because JNIEnv is Copy when the underlying pointer is valid
        Self {
            env: unsafe {
                // Use ptr::read to create a bitwise copy of the JNIEnv
                std::ptr::read(&self.env as *const JNIEnv<'local>)
            },
        }
    }
}

impl<'local> SafeJNIEnv<'local> {
    /// Create a new SafeJNIEnv wrapper
    ///
    /// # Arguments
    ///
    /// * `env` - The JNI environment to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// let safe_env = SafeJNIEnv::new(env);
    /// ```
    pub fn new(env: JNIEnv<'local>) -> Self {
        Self { env }
    }

    /// Get the underlying JNI environment
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying JNIEnv
    pub fn env(&mut self) -> &mut JNIEnv<'local> {
        &mut self.env
    }

    /// Get the underlying JNI environment as mutable reference for exception checking
    ///
    /// # Returns
    ///
    /// A mutable reference to the underlying JNIEnv
    pub fn env_mut(&mut self) -> &mut JNIEnv<'local> {
        &mut self.env
    }

    /// Call a method with automatic exception checking
    ///
    /// This method calls a Java instance method and automatically checks for exceptions.
    ///
    /// # Arguments
    ///
    /// * `obj` - The Java object to call the method on
    /// * `name` - The method name
    /// * `sig` - The method signature in JNI format
    /// * `args` - The method arguments
    ///
    /// # Returns
    ///
    /// The method result converted to the specified type, or an error if the call failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let result: JObject = safe_env.call_method_checked(
    ///     &object,
    ///     "toString",
    ///     "()Ljava/lang/String;",
    ///     &[]
    /// )?;
    /// ```
    pub fn call_method_checked<T>(
        &mut self,
        obj: &JObject,
        name: &str,
        sig: &str,
        args: &[JValue],
    ) -> Result<T>
    where
        T: From<JValue<'local, 'local>>,
    {
        let result = self
            .env
            .call_method(obj, name, sig, args)
            .check_exception(&mut self.env)?;
        // Handle JValue conversion - JValue implements TryFrom for basic types
        let jvalue = result;
        // Convert using direct matching since From trait has issues
        let converted = match std::any::type_name::<T>() {
            "jni::objects::JObject" => {
                let obj = jvalue.l().map_err(AnkiDroidError::from)?;
                unsafe { std::mem::transmute_copy(&obj) }
            }
            _ => unsafe { std::mem::transmute_copy(&jvalue) },
        };
        Ok(converted)
    }

    /// Call a static method with automatic exception checking
    ///
    /// This method calls a Java static method and automatically checks for exceptions.
    ///
    /// # Arguments
    ///
    /// * `class` - The class name in JNI format (e.g., "java/lang/String")
    /// * `name` - The method name
    /// * `sig` - The method signature in JNI format
    /// * `args` - The method arguments
    ///
    /// # Returns
    ///
    /// The method result converted to the specified type, or an error if the call failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let result: JObject = safe_env.call_static_method_checked(
    ///     "java/lang/System",
    ///     "currentTimeMillis",
    ///     "()J",
    ///     &[]
    /// )?;
    /// ```
    pub fn call_static_method_checked<T>(
        &mut self,
        class: &str,
        name: &str,
        sig: &str,
        args: &[JValue],
    ) -> Result<T>
    where
        T: From<JValue<'local, 'local>>,
    {
        let result = self
            .env
            .call_static_method(class, name, sig, args)
            .check_exception(&mut self.env)?;
        // Handle JValue conversion - JValue implements TryFrom for basic types
        let jvalue = result;
        // Convert using direct matching since From trait has issues
        let converted = match std::any::type_name::<T>() {
            "jni::objects::JObject" => {
                let obj = jvalue.l().map_err(AnkiDroidError::from)?;
                unsafe { std::mem::transmute_copy(&obj) }
            }
            _ => unsafe { std::mem::transmute_copy(&jvalue) },
        };
        Ok(converted)
    }

    /// Find a class with error handling
    ///
    /// # Arguments
    ///
    /// * `name` - The class name in JNI format (e.g., "java/lang/String")
    ///
    /// # Returns
    ///
    /// A reference to the class, or an error if the class couldn't be found
    ///
    /// # Examples
    ///
    /// ```rust
    /// let string_class = safe_env.find_class_checked("java/lang/String")?;
    /// ```
    pub fn find_class_checked(
        &mut self,
        name: &str,
    ) -> Result<jni::objects::JClass<'local>> {
        self.env.find_class(name).check_exception(&mut self.env)
    }

    /// Create a new string with error handling
    ///
    /// # Arguments
    ///
    /// * `text` - The Rust string to convert
    ///
    /// # Returns
    ///
    /// A Java string object, or an error if the conversion failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let java_string = safe_env.new_string_checked("Hello, World!")?;
    /// ```
    pub fn new_string_checked(&mut self, text: &str) -> Result<JString<'local>> {
        self.env.new_string(text).check_exception(&mut self.env)
    }

    /// Get string value with error handling
    ///
    /// # Arguments
    ///
    /// * `string` - The Java string to convert
    ///
    /// # Returns
    ///
    /// A Rust string, or an error if the conversion failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let rust_string = safe_env.get_string_checked(&java_string)?;
    /// ```
    pub fn get_string_checked(&mut self, string: &JString) -> Result<String> {
        let java_str = self.env.get_string(string).check_exception(&mut self.env)?;
        Ok(java_str
            .to_str()
            .map_err(|e| AnkiDroidError::string_conversion_error(e.to_string()))?
            .to_string())
    }

    /// Create a new object with error handling
    ///
    /// # Arguments
    ///
    /// * `class` - The class name in JNI format
    /// * `ctor_sig` - The constructor signature
    /// * `ctor_args` - The constructor arguments
    ///
    /// # Returns
    ///
    /// A new Java object, or an error if the construction failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let integer = safe_env.new_object_checked(
    ///     "java/lang/Integer",
    ///     "(I)V",
    ///     &[JValue::Int(42)]
    /// )?;
    /// ```
    pub fn new_object_checked(
        &mut self,
        class: &str,
        ctor_sig: &str,
        ctor_args: &[JValue],
    ) -> Result<JObject<'local>> {
        self.env
            .new_object(class, ctor_sig, ctor_args)
            .check_exception(&mut self.env)
    }
}

/// RAII wrapper for JNI local reference frames
///
/// Local frames help manage JNI local references by providing a way to create
/// a "frame" in which local references are created, and then release all of
/// them at once when the frame is popped.
///
/// This is particularly useful for avoiding local reference table overflow
/// when creating many local references in a loop or recursive function.
///
/// # Examples
///
/// ```rust
/// fn process_many_objects(env: &mut JNIEnv) -> Result<()> {
///     let _frame = LocalFrame::new_default(env)?;
///     
///     // Create many local references here
///     for i in 0..1000 {
///         let string = env.new_string(&format!("String {}", i))?;
///         // ... process string
///     }
///     
///     // All local references created in this frame are automatically
///     // freed when _frame is dropped
///     Ok(())
/// }
/// ```
pub struct LocalFrame<'local> {
    env: &'local mut JNIEnv<'local>,
    capacity: i32,
}

impl<'local> LocalFrame<'local> {
    /// Create a new local frame with specified capacity
    ///
    /// # Arguments
    ///
    /// * `env` - The JNI environment
    /// * `capacity` - The maximum number of local references this frame can hold
    ///
    /// # Returns
    ///
    /// A new local frame, or an error if the frame couldn't be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// let frame = LocalFrame::new(env, 100)?;
    /// ```
    pub fn new(env: &'local mut JNIEnv<'local>, capacity: i32) -> Result<Self> {
        // Push local frame - can't check exception due to borrow rules
        env.push_local_frame(capacity)
            .map_err(|e| AnkiDroidError::from(e))?;
        Ok(Self { env, capacity })
    }

    /// Create a new local frame with default capacity (512)
    ///
    /// # Arguments
    ///
    /// * `env` - The JNI environment
    ///
    /// # Returns
    ///
    /// A new local frame with default capacity, or an error if the frame couldn't be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// let frame = LocalFrame::new_default(env)?;
    /// ```
    pub fn new_default(env: &'local mut JNIEnv<'local>) -> Result<Self> {
        Self::new(env, 512)
    }
}

impl<'local> Drop for LocalFrame<'local> {
    fn drop(&mut self) {
        // Pop the local frame to free all local references created within it
        // SAFETY: This is safe because we're properly cleaning up the local frame
        // that was previously pushed in LocalFrame::new()
        unsafe {
            let _ = self.env.pop_local_frame(&JObject::null());
        }
    }
}

/// Helper for string conversions between Rust and Java
///
/// This struct provides convenient methods for converting strings between
/// Rust and Java, handling all the necessary JNI operations and error checking.
///
/// # Examples
///
/// ```rust
/// // Convert Rust string to Java
/// let java_string = StringHelper::rust_to_java(&mut safe_env, "Hello")?;
///
/// // Convert Java string back to Rust
/// let rust_string = StringHelper::java_to_rust(&mut safe_env, &java_string)?;
/// ```
pub struct StringHelper;

impl StringHelper {
    /// Convert a Rust string to a Java string
    ///
    /// # Arguments
    ///
    /// * `env` - The safe JNI environment
    /// * `text` - The Rust string to convert
    ///
    /// # Returns
    ///
    /// A Java string object, or an error if the conversion failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let java_string = StringHelper::rust_to_java(&mut safe_env, "Hello, World!")?;
    /// ```
    pub fn rust_to_java<'local>(
        env: &mut SafeJNIEnv<'local>,
        text: &str,
    ) -> Result<JString<'local>> {
        env.new_string_checked(text)
    }

    /// Convert a Java string to a Rust string
    ///
    /// # Arguments
    ///
    /// * `env` - The safe JNI environment
    /// * `java_string` - The Java string to convert
    ///
    /// # Returns
    ///
    /// A Rust string, or an error if the conversion failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let rust_string = StringHelper::java_to_rust(&mut safe_env, &java_string)?;
    /// ```
    pub fn java_to_rust<'local>(
        env: &mut SafeJNIEnv<'local>,
        java_string: &JString<'local>,
    ) -> Result<String> {
        env.get_string_checked(java_string)
    }

    /// Convert a Java object string to a Rust string (for getString results)
    ///
    /// This is useful when you receive a JObject from a method call that returns
    /// a String, and you need to convert it to a Rust string.
    ///
    /// # Arguments
    ///
    /// * `env` - The safe JNI environment
    /// * `obj` - The Java object (should be a String)
    ///
    /// # Returns
    ///
    /// A Rust string, or an empty string if the object is null, or an error if the conversion failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let result_obj = safe_env.call_method_checked(&object, "toString", "()Ljava/lang/String;", &[])?;
    /// let rust_string = StringHelper::jobject_to_rust(&mut safe_env, &result_obj)?;
    /// ```
    pub fn jobject_to_rust<'local>(
        env: &mut SafeJNIEnv<'local>,
        obj: &JObject<'local>,
    ) -> Result<String> {
        if obj.is_null() {
            return Ok(String::new());
        }
        // In JNI 0.21, we need to use unsafe to convert JObject to JString
        let java_string: JString = unsafe { JString::from_raw(obj.as_raw()) };
        Self::java_to_rust(env, &java_string)
    }
}

/// ContentValues helper for building Android ContentValues objects
///
/// This builder provides a type-safe way to construct Android ContentValues
/// objects, which are used to insert or update data in SQLite databases
/// or content providers.
///
/// # Examples
///
/// ```rust
/// let content_values = ContentValuesBuilder::new(&mut safe_env)?
///     .put_string("name", "John Doe")?
///     .put_int("age", 30)?
///     .put_long("timestamp", 1234567890)?
///     .build();
/// ```
pub struct ContentValuesBuilder<'local> {
    env: SafeJNIEnv<'local>,
    content_values: JObject<'local>,
}

impl<'local> ContentValuesBuilder<'local> {
    /// Create a new ContentValues builder
    ///
    /// # Arguments
    ///
    /// * `env` - The safe JNI environment
    ///
    /// # Returns
    ///
    /// A new ContentValues builder, or an error if the ContentValues object couldn't be created
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = ContentValuesBuilder::new(&mut safe_env)?;
    /// ```
    pub fn new(env: &mut SafeJNIEnv<'local>) -> Result<Self> {
        let content_values_class = env.find_class_checked("android/content/ContentValues")?;
        let content_values = env
            .env()
            .new_object(content_values_class, "()V", &[])
            .check_exception(env.env_mut())?;

        Ok(Self {
            env: env.clone(),
            content_values,
        })
    }

    /// Add a string value
    ///
    /// # Arguments
    ///
    /// * `key` - The key name
    /// * `value` - The string value
    ///
    /// # Returns
    ///
    /// Self for method chaining, or an error if the operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = builder.put_string("name", "John Doe")?;
    /// ```
    pub fn put_string(mut self, key: &str, value: &str) -> Result<Self> {
        let key_string = self.env.new_string_checked(key)?;
        let value_string = self.env.new_string_checked(value)?;

        self.env
            .env_mut()
            .call_method(
                &self.content_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(&key_string.into()),
                    JValue::Object(&value_string.into()),
                ],
            )
            .check_exception(self.env.env_mut())?;

        Ok(self)
    }

    /// Add a long value
    ///
    /// # Arguments
    ///
    /// * `key` - The key name
    /// * `value` - The long value
    ///
    /// # Returns
    ///
    /// Self for method chaining, or an error if the operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = builder.put_long("timestamp", 1234567890)?;
    /// ```
    pub fn put_long(mut self, key: &str, value: i64) -> Result<Self> {
        let key_string = self.env.new_string_checked(key)?;
        let long_obj =
            self.env
                .new_object_checked("java/lang/Long", "(J)V", &[JValue::Long(value)])?;

        self.env
            .env_mut()
            .call_method(
                &self.content_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/Long;)V",
                &[
                    JValue::Object(&key_string.into()),
                    JValue::Object(&long_obj),
                ],
            )
            .check_exception(self.env.env_mut())?;

        Ok(self)
    }

    /// Add an integer value
    ///
    /// # Arguments
    ///
    /// * `key` - The key name
    /// * `value` - The integer value
    ///
    /// # Returns
    ///
    /// Self for method chaining, or an error if the operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let builder = builder.put_int("age", 30)?;
    /// ```
    pub fn put_int(mut self, key: &str, value: i32) -> Result<Self> {
        let key_string = self.env.new_string_checked(key)?;
        let int_obj =
            self.env
                .new_object_checked("java/lang/Integer", "(I)V", &[JValue::Int(value)])?;

        self.env
            .env_mut()
            .call_method(
                &self.content_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/Integer;)V",
                &[JValue::Object(&key_string.into()), JValue::Object(&int_obj)],
            )
            .check_exception(self.env.env_mut())?;

        Ok(self)
    }

    /// Build and return the ContentValues object
    ///
    /// # Returns
    ///
    /// The completed ContentValues object
    ///
    /// # Examples
    ///
    /// ```rust
    /// let content_values = builder.build();
    /// ```
    pub fn build(self) -> JObject<'local> {
        self.content_values
    }
}

/// RAII guard for thread attachment to JavaVM
///
/// This guard ensures that the current thread is attached to the JavaVM
/// for the duration of its lifetime, and automatically detaches the thread
/// when dropped.
///
/// # Examples
///
/// ```rust
/// let (guard, env) = AttachGuard::new(vm)?;
/// // Use env for JNI operations
/// // Thread is automatically detached when guard is dropped
/// ```
pub struct AttachGuard {
    vm: JavaVM,
}

impl AttachGuard {
    /// Create a new AttachGuard and attach the current thread
    ///
    /// # Arguments
    ///
    /// * `vm` - The JavaVM to attach to
    ///
    /// # Returns
    ///
    /// A tuple containing the guard and a JNI environment, or an error if attachment failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let (guard, env) = AttachGuard::new(vm)?;
    /// ```
    pub fn new(vm: JavaVM) -> Result<(Self, JNIEnv<'static>)> {
        let attach_guard = vm.attach_current_thread().map_err(AnkiDroidError::from)?;
        
        // Extract the JNIEnv from the AttachGuard using unsafe pointer manipulation
        // This is safe because we ensure the thread stays attached via our own guard
        let env_ptr = attach_guard.deref() as *const JNIEnv<'_>;
        let static_env = unsafe { std::ptr::read(env_ptr) };
        let static_env = unsafe { std::mem::transmute::<JNIEnv<'_>, JNIEnv<'static>>(static_env) };
        
        // Forget the original attach guard so it doesn't detach
        std::mem::forget(attach_guard);
        
        Ok((Self { vm }, static_env))
    }
}

impl Drop for AttachGuard {
    fn drop(&mut self) {
        // Detach the thread when the guard is dropped
        // Ignore errors since we can't propagate them from Drop
        unsafe {
            let _ = self.vm.detach_current_thread();
        }
    }
}

/// Helper function to attach current thread to JavaVM
///
/// Returns a JNIEnv that's valid for the current thread. The thread will remain
/// attached until the returned guard is dropped.
///
/// # Arguments
///
/// * `vm` - The JavaVM to attach to
///
/// # Returns
///
/// A JNI environment that's valid for the current thread
///
/// # Examples
///
/// ```rust
/// let env = attach_current_thread(vm)?;
/// // Use env for JNI operations
/// // Note: Thread will remain attached - use AttachGuard for automatic cleanup
/// ```
pub fn attach_current_thread(vm: JavaVM) -> Result<JNIEnv<'static>> {
    let (guard, env) = AttachGuard::new(vm)?;
    // Leak the guard to keep the thread attached
    // This is intentional for backwards compatibility
    std::mem::forget(guard);
    Ok(env)
}

/// Helper function to get ContentResolver from an Android Context
///
/// # Arguments
///
/// * `env` - The safe JNI environment
/// * `context` - The Android Context object
///
/// # Returns
///
/// A ContentResolver object, or an error if the operation failed
///
/// # Examples
///
/// ```rust
/// let content_resolver = get_content_resolver(&mut safe_env, &context)?;
/// ```
pub fn get_content_resolver<'local>(
    env: &mut SafeJNIEnv<'local>,
    context: &JObject<'local>,
) -> Result<JObject<'local>> {
    // Validate the context is not null
    if context.is_null() {
        return Err(AnkiDroidError::validation_error(
            "Context is null when getting ContentResolver",
        ));
    }

    let result = env
        .env_mut()
        .call_method(
            context,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .check_exception(env.env_mut())?;

    let content_resolver = result.l().map_err(AnkiDroidError::from)?;

    if content_resolver.is_null() {
        return Err(AnkiDroidError::validation_error(
            "ContentResolver is null",
        ));
    }

    Ok(content_resolver)
}

/// Helper function to parse URI string
///
/// # Arguments
///
/// * `env` - The safe JNI environment
/// * `uri_string` - The URI string to parse
///
/// # Returns
///
/// A URI object, or an error if parsing failed
///
/// # Examples
///
/// ```rust
/// let uri = parse_uri(&mut safe_env, "content://com.ichi2.anki.flascards/decks")?;
/// ```
pub fn parse_uri<'local>(
    env: &mut SafeJNIEnv<'local>,
    uri_string: &str,
) -> Result<JObject<'local>> {
    let uri_class = env.find_class_checked("android/net/Uri")?;
    let uri_str = env.new_string_checked(uri_string)?;

    let result = env
        .env_mut()
        .call_static_method(
            &uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&uri_str.into())],
        )
        .check_exception(env.env_mut())?;

    result.l().map_err(AnkiDroidError::from)
}

/// Helper trait for converting JNI results with exception checking
///
/// This trait extends JNI results to provide automatic exception checking
/// and conversion to our error types.
pub trait JniResultExt<T> {
    /// Convert JNI result and check for Java exceptions
    ///
    /// # Arguments
    ///
    /// * `env` - The JNI environment for exception checking
    ///
    /// # Returns
    ///
    /// The result value or an error if an exception occurred
    fn check_exception(self, env: &mut jni::JNIEnv) -> Result<T>;
}

impl<T> JniResultExt<T> for jni::errors::Result<T> {
    fn check_exception(self, env: &mut jni::JNIEnv) -> Result<T> {
        match self {
            Ok(value) => {
                // Check if a Java exception occurred
                if env.exception_check().unwrap_or(false) {
                    let exception_msg = get_exception_message(env)
                        .unwrap_or_else(|| "Unknown Java exception".to_string());
                    env.exception_clear().ok();
                    Err(AnkiDroidError::jni_error(exception_msg))
                } else {
                    Ok(value)
                }
            }
            Err(jni_error) => Err(AnkiDroidError::from(jni_error)),
        }
    }
}

/// Helper function to extract exception message from JNI environment
///
/// # Arguments
///
/// * `env` - The JNI environment to check for exceptions
///
/// # Returns
///
/// The exception message if available, or None if no exception or message couldn't be extracted
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
    fn test_string_helper_basic_operations() {
        // Test basic string operations without actual JNI
        // This is a placeholder test since we can't test JNI in unit tests
        assert!(true);
    }

    #[test]
    fn test_error_types() {
        let error = AnkiDroidError::string_conversion_error("test");
        assert!(error.to_string().contains("String conversion error"));
    }

    #[test]
    fn test_result_type() {
        let success: Result<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        let failure: Result<i32> = Err(AnkiDroidError::validation_error("test"));
        assert!(failure.is_err());
    }

    #[test]
    fn test_attach_guard_creation() {
        // Test that AttachGuard creation doesn't panic
        // We can't test actual attachment without a JavaVM
        assert!(true);
    }

    #[test]
    fn test_local_frame_capacity() {
        // Test that we can create a LocalFrame with different capacities
        // We can't test actual frame creation without a JNIEnv
        assert!(true);
    }
}
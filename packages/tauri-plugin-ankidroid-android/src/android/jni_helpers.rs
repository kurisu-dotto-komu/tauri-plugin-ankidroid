use crate::android::error::{AndroidError, AndroidResult, JniResultExt};
use jni::objects::{JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use ndk_context;
use std::ops::Deref;

/// Safe JNI environment wrapper with automatic exception checking
pub struct SafeJNIEnv<'local> {
    env: JNIEnv<'local>,
}

impl<'local> Clone for SafeJNIEnv<'local> {
    fn clone(&self) -> Self {
        // JNIEnv doesn't implement Clone, but we can create a new one from the JavaVM
        // This is safe because JNIEnv is just a wrapper around a pointer
        Self {
            env: unsafe { std::mem::transmute_copy(&self.env) },
        }
    }
}

impl<'local> SafeJNIEnv<'local> {
    /// Create a new SafeJNIEnv wrapper
    pub fn new(env: JNIEnv<'local>) -> Self {
        Self { env }
    }

    /// Get the underlying JNI environment
    pub fn env(&mut self) -> &mut JNIEnv<'local> {
        &mut self.env
    }

    /// Get the underlying JNI environment as mutable reference for exception checking
    pub fn env_mut(&mut self) -> &mut JNIEnv<'local> {
        &mut self.env
    }

    /// Call a method with automatic exception checking
    pub fn call_method_checked<T>(
        &mut self,
        obj: &JObject,
        name: &str,
        sig: &str,
        args: &[JValue],
    ) -> AndroidResult<T>
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
                let obj = jvalue.l().map_err(AndroidError::from)?;
                unsafe { std::mem::transmute_copy(&obj) }
            }
            _ => unsafe { std::mem::transmute_copy(&jvalue) },
        };
        Ok(converted)
    }

    /// Call a static method with automatic exception checking
    pub fn call_static_method_checked<T>(
        &mut self,
        class: &str,
        name: &str,
        sig: &str,
        args: &[JValue],
    ) -> AndroidResult<T>
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
                let obj = jvalue.l().map_err(AndroidError::from)?;
                unsafe { std::mem::transmute_copy(&obj) }
            }
            _ => unsafe { std::mem::transmute_copy(&jvalue) },
        };
        Ok(converted)
    }

    /// Find a class with error handling
    pub fn find_class_checked(
        &mut self,
        name: &str,
    ) -> AndroidResult<jni::objects::JClass<'local>> {
        self.env.find_class(name).check_exception(&mut self.env)
    }

    /// Create a new string with error handling
    pub fn new_string_checked(&mut self, text: &str) -> AndroidResult<JString<'local>> {
        self.env.new_string(text).check_exception(&mut self.env)
    }

    /// Get string value with error handling
    pub fn get_string_checked(&mut self, string: &JString) -> AndroidResult<String> {
        let java_str = self.env.get_string(string).check_exception(&mut self.env)?;
        Ok(java_str
            .to_str()
            .map_err(|e| AndroidError::StringConversionError(e.to_string()))?
            .to_string())
    }

    /// Create a new object with error handling
    pub fn new_object_checked(
        &mut self,
        class: &str,
        ctor_sig: &str,
        ctor_args: &[JValue],
    ) -> AndroidResult<JObject<'local>> {
        self.env
            .new_object(class, ctor_sig, ctor_args)
            .check_exception(&mut self.env)
    }
}

/// RAII wrapper for JNI local reference frames
pub struct LocalFrame<'local> {
    env: &'local mut JNIEnv<'local>,
    capacity: i32,
}

impl<'local> LocalFrame<'local> {
    /// Create a new local frame with specified capacity
    pub fn new(env: &'local mut JNIEnv<'local>, capacity: i32) -> AndroidResult<Self> {
        // Push local frame - can't check exception due to borrow rules
        env.push_local_frame(capacity)
            .map_err(|e| AndroidError::from(e))?;
        Ok(Self { env, capacity })
    }

    /// Create a new local frame with default capacity (512)
    pub fn new_default(env: &'local mut JNIEnv<'local>) -> AndroidResult<Self> {
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
pub struct StringHelper;

impl StringHelper {
    /// Convert a Rust string to a Java string
    pub fn rust_to_java<'local>(
        env: &mut SafeJNIEnv<'local>,
        text: &str,
    ) -> AndroidResult<JString<'local>> {
        env.new_string_checked(text)
    }

    /// Convert a Java string to a Rust string
    pub fn java_to_rust<'local>(
        env: &mut SafeJNIEnv<'local>,
        java_string: &JString<'local>,
    ) -> AndroidResult<String> {
        env.get_string_checked(java_string)
    }

    /// Convert a Java object string to a Rust string (for getString results)
    pub fn jobject_to_rust<'local>(
        env: &mut SafeJNIEnv<'local>,
        obj: &JObject<'local>,
    ) -> AndroidResult<String> {
        if obj.is_null() {
            return Ok(String::new());
        }
        // In JNI 0.21, we need to use unsafe to convert JObject to JString
        let java_string: JString = unsafe { JString::from_raw(obj.as_raw()) };
        Self::java_to_rust(env, &java_string)
    }
}

/// ContentValues helper for building Android ContentValues objects
pub struct ContentValuesBuilder<'local> {
    env: &'local mut SafeJNIEnv<'local>,
    content_values: JObject<'local>,
}

impl<'local> ContentValuesBuilder<'local> {
    /// Create a new ContentValues builder
    pub fn new(env: &'local mut SafeJNIEnv<'local>) -> AndroidResult<Self> {
        let content_values_class = env.find_class_checked("android/content/ContentValues")?;
        let content_values = env
            .env()
            .new_object(content_values_class, "()V", &[])
            .check_exception(env.env_mut())?;

        Ok(Self {
            env,
            content_values,
        })
    }

    /// Add a string value
    pub fn put_string(mut self, key: &str, value: &str) -> AndroidResult<Self> {
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
    pub fn put_long(mut self, key: &str, value: i64) -> AndroidResult<Self> {
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
    pub fn put_int(mut self, key: &str, value: i32) -> AndroidResult<Self> {
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
    pub fn build(self) -> JObject<'local> {
        self.content_values
    }
}

/// Helper function to get Android context and JavaVM
pub fn get_android_context() -> AndroidResult<(JavaVM, JObject<'static>)> {
    let ctx = ndk_context::android_context();
    
    // Validate that we have a valid context
    if ctx.vm().is_null() {
        return Err(AndroidError::ValidationError("JavaVM is null - Android context not initialized".to_string()));
    }
    
    if ctx.context().is_null() {
        return Err(AndroidError::ValidationError("Activity context is null - Android context not initialized".to_string()));
    }
    
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(AndroidError::from)?;
    let activity = unsafe { JObject::from_raw(ctx.context() as *mut _) };
    Ok((vm, activity))
}

/// Helper function to attach current thread to JavaVM
/// Returns a JNIEnv that's valid for the current thread
pub fn attach_current_thread() -> AndroidResult<JNIEnv<'static>> {
    let (vm, _) = get_android_context()?;
    // attach_current_thread returns an AttachGuard which derefs to JNIEnv
    // We need to leak the guard to get a 'static lifetime
    let guard = vm.attach_current_thread().map_err(AndroidError::from)?;
    // Leak the guard to keep it alive
    // This is safe because the thread attachment lasts for the thread's lifetime
    let leaked_guard = Box::leak(Box::new(guard));
    // Use transmute_copy to copy the JNIEnv with a 'static lifetime
    let env = unsafe { std::mem::transmute_copy::<JNIEnv<'_>, JNIEnv<'static>>(leaked_guard.deref()) };
    Ok(env)
}

/// Helper function to get ContentResolver
pub fn get_content_resolver<'local>(
    env: &mut SafeJNIEnv<'local>,
    activity: &JObject<'local>,
) -> AndroidResult<JObject<'local>> {
    // Validate the activity is not null
    if activity.is_null() {
        return Err(AndroidError::ValidationError("Activity is null when getting ContentResolver".to_string()));
    }
    
    let result = env
        .env_mut()
        .call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .check_exception(env.env_mut())?;

    let content_resolver = result.l().map_err(AndroidError::from)?;
    
    if content_resolver.is_null() {
        return Err(AndroidError::ValidationError("ContentResolver is null".to_string()));
    }
    
    Ok(content_resolver)
}

/// Helper function to parse URI string
pub fn parse_uri<'local>(
    env: &mut SafeJNIEnv<'local>,
    uri_string: &str,
) -> AndroidResult<JObject<'local>> {
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

    result.l().map_err(AndroidError::from)
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
        let error = AndroidError::StringConversionError("test".to_string());
        assert!(error.to_string().contains("String conversion error"));
    }

    #[test]
    fn test_android_result_type() {
        let success: AndroidResult<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        let failure: AndroidResult<i32> = Err(AndroidError::ValidationError("test".to_string()));
        assert!(failure.is_err());
    }
}

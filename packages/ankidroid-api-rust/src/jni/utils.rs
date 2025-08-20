//! JNI utility functions

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JString, JValue},
    JNIEnv,
};

#[cfg(target_os = "android")]
use crate::error::{Error, Result};

#[cfg(target_os = "android")]
/// Convert a Java string to a Rust string
pub fn jstring_to_string(env: &mut JNIEnv, jstr: &JString) -> Result<String> {
    let java_str = env.get_string(jstr)?;
    Ok(java_str.into())
}

#[cfg(target_os = "android")]
/// Convert a Rust string to a Java string
pub fn string_to_jstring<'a>(env: &mut JNIEnv<'a>, s: &str) -> Result<JString<'a>> {
    Ok(env.new_string(s)?)
}

#[cfg(target_os = "android")]
/// Get a long value from a Java object field
pub fn get_long_field(env: &mut JNIEnv, obj: &JObject, field_name: &str) -> Result<i64> {
    let _class = env.get_object_class(obj)?;
    let value = env.get_field(obj, field_name, "J")?;
    Ok(value.j().map_err(|_| {
        Error::validation_error(format!("Expected long field: {}", field_name))
    })?)
}

#[cfg(target_os = "android")]
/// Get a string value from a Java object field
pub fn get_string_field(env: &mut JNIEnv, obj: &JObject, field_name: &str) -> Result<String> {
    let _class = env.get_object_class(obj)?;
    let value = env.get_field(obj, field_name, "Ljava/lang/String;")?;
    let obj = value.l().map_err(|_| {
        Error::validation_error(format!("Expected string field: {}", field_name))
    })?;
    let jstr = JString::from(obj);
    jstring_to_string(env, &jstr)
}

#[cfg(target_os = "android")]
/// Get an int value from a Java object field
pub fn get_int_field(env: &mut JNIEnv, obj: &JObject, field_name: &str) -> Result<i32> {
    let _class = env.get_object_class(obj)?;
    let value = env.get_field(obj, field_name, "I")?;
    Ok(value.i().map_err(|_| {
        Error::validation_error(format!("Expected int field: {}", field_name))
    })?)
}
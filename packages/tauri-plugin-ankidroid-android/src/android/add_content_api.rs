/// JNI wrapper for AnkiDroid's AddContentApi
/// This module provides access to the high-level AnkiDroid API for adding content
use crate::android::error::{AndroidError, AndroidResult, JniResultExt};
use crate::android::jni_helpers::SafeJNIEnv;
use jni::objects::{JObject, JValue};

/// Wrapper for AnkiDroid's AddContentApi class
pub struct AddContentApi<'local> {
    api_instance: JObject<'local>,
    env: SafeJNIEnv<'local>,
}

impl<'local> AddContentApi<'local> {
    /// Create a new instance of AddContentApi wrapper
    /// Note: This doesn't instantiate the actual AddContentApi class (which is in AnkiDroid app)
    /// Instead, it prepares a wrapper for ContentResolver-based operations
    pub fn new(env: SafeJNIEnv<'local>, context: &JObject<'local>) -> AndroidResult<Self> {
        log::info!("Creating AddContentApi wrapper (ContentResolver-based)...");
        
        // We don't instantiate AddContentApi class here since it's in AnkiDroid app
        // Instead, we'll use ContentResolver for all operations
        // For now, just store a copy of the context reference as our "api_instance"
        let api_instance = unsafe { JObject::from_raw(context.as_raw()) };
        
        log::info!("✅ AddContentApi wrapper created");
        Ok(Self {
            api_instance,
            env,
        })
    }

    /// Add a note to AnkiDroid
    /// Returns the note ID on success, or None if the note couldn't be added
    pub fn add_note(
        &mut self,
        _model_id: i64,
        _deck_id: i64,
        _fields: &[&str],
        _tags: Option<&[&str]>,
    ) -> AndroidResult<Option<i64>> {
        // This method can't be used from external apps - AddContentApi class is in AnkiDroid
        Err(AndroidError::AnkiDroidNotAvailable(
            "AddContentApi methods cannot be called from external apps. Use ContentResolver API instead.".to_string()
        ))
    }

    /// Add a new deck to AnkiDroid
    /// Returns the deck ID
    pub fn add_new_deck(&mut self, _deck_name: &str) -> AndroidResult<i64> {
        // This method can't be used from external apps - AddContentApi class is in AnkiDroid
        Err(AndroidError::AnkiDroidNotAvailable(
            "AddContentApi methods cannot be called from external apps. Use ContentResolver API instead.".to_string()
        ))
    }

    /// Add a new basic model (note type) to AnkiDroid
    /// Returns the model ID
    pub fn add_new_basic_model(&mut self, _model_name: &str) -> AndroidResult<i64> {
        // This method can't be used from external apps - AddContentApi class is in AnkiDroid
        Err(AndroidError::AnkiDroidNotAvailable(
            "AddContentApi methods cannot be called from external apps. Use ContentResolver API instead.".to_string()
        ))
    }

    /// Check if AnkiDroid is available
    pub fn is_available(env: &mut SafeJNIEnv, context: &JObject) -> bool {
        // For external apps, check if AnkiDroid is installed via PackageManager
        // We don't check for AddContentApi class since it's in AnkiDroid app, not ours
        
        // Try to get PackageManager to check if AnkiDroid is installed
        let pm_result = env.env_mut().call_method(
            context,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        );
        
        let package_manager = match pm_result {
            Ok(result) => match result.l() {
                Ok(pm) if !pm.is_null() => pm,
                _ => return false,
            },
            Err(_) => return false,
        };
        
        // Check if AnkiDroid package is installed
        let package_name = match env.new_string_checked("com.ichi2.anki") {
            Ok(name) => name,
            Err(_) => return false,
        };
        
        // Try to get package info (will fail if not installed)
        let package_info_result = env.env_mut().call_method(
            &package_manager,
            "getPackageInfo",
            "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;",
            &[JValue::Object(&package_name.into()), JValue::Int(0)],
        );
        
        match package_info_result {
            Ok(result) => match result.l() {
                Ok(info) => {
                    log::debug!("AnkiDroid package found: com.ichi2.anki");
                    !info.is_null()
                },
                Err(_) => {
                    log::debug!("AnkiDroid package not found");
                    false
                },
            },
            Err(_) => {
                log::debug!("Failed to check for AnkiDroid package");
                false
            },
        }
    }
}
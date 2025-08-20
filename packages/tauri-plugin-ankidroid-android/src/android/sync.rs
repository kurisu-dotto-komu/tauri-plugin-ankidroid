use crate::android::api_check::check_ankidroid_available;
use crate::android::error::{AndroidError, AndroidResult, JniResultExt};
use crate::android::jni_helpers::SafeJNIEnv;
use jni::objects::{JObject, JValue};
use std::time::{SystemTime, UNIX_EPOCH};

const SYNC_INTENT_ACTION: &str = "com.ichi2.anki.DO_SYNC";
const SYNC_COOLDOWN_SECONDS: u64 = 300; // 5 minutes

static mut LAST_SYNC_TIME: Option<u64> = None;

/// Trigger a sync operation in AnkiDroid
pub fn trigger_sync(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<bool> {
    log::info!("Triggering AnkiDroid sync");

    // Check if AnkiDroid is available
    if !check_ankidroid_available(env, context)? {
        return Err(AndroidError::AnkiDroidNotAvailable(
            "AnkiDroid is not installed".to_string(),
        ));
    }

    // Check cooldown
    if !check_sync_cooldown()? {
        log::warn!("Sync cooldown not met, skipping sync");
        return Ok(false);
    }

    // Create intent with DO_SYNC action
    let intent_class = env.find_class_checked("android/content/Intent")?;
    let intent = env
        .env_mut()
        .new_object(&intent_class, "()V", &[])
        .check_exception(env.env_mut())?;

    // Set action
    let action_string = env.new_string_checked(SYNC_INTENT_ACTION)?;
    env.env_mut()
        .call_method(
            &intent,
            "setAction",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&action_string.into())],
        )
        .check_exception(env.env_mut())?;

    // Set package to AnkiDroid
    let package_name = env.new_string_checked("com.ichi2.anki")?;
    env.env_mut()
        .call_method(
            &intent,
            "setPackage",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&package_name.into())],
        )
        .check_exception(env.env_mut())?;

    // Send broadcast
    env.env_mut()
        .call_method(
            context,
            "sendBroadcast",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )
        .check_exception(env.env_mut())?;

    // Update last sync time
    update_last_sync_time();

    log::info!("Sync broadcast sent successfully");
    Ok(true)
}

/// Check if enough time has passed since the last sync
fn check_sync_cooldown() -> AndroidResult<bool> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AndroidError::SyncError(format!("Time error: {}", e)))?
        .as_secs();

    unsafe {
        if let Some(last_sync) = LAST_SYNC_TIME {
            let time_since_last_sync = current_time - last_sync;
            if time_since_last_sync < SYNC_COOLDOWN_SECONDS {
                log::debug!(
                    "Sync cooldown active: {} seconds remaining",
                    SYNC_COOLDOWN_SECONDS - time_since_last_sync
                );
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Update the last sync time to current time
fn update_last_sync_time() {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    unsafe {
        LAST_SYNC_TIME = Some(current_time);
    }
}

/// Get the time remaining until the next sync is allowed (in seconds)
pub fn get_sync_cooldown_remaining() -> u64 {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    unsafe {
        if let Some(last_sync) = LAST_SYNC_TIME {
            let time_since_last_sync = current_time - last_sync;
            if time_since_last_sync < SYNC_COOLDOWN_SECONDS {
                return SYNC_COOLDOWN_SECONDS - time_since_last_sync;
            }
        }
    }

    0
}

/// Force a sync operation (bypasses cooldown)
pub fn force_sync(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<bool> {
    log::info!("Force triggering AnkiDroid sync (bypassing cooldown)");

    // Reset last sync time to allow immediate sync
    unsafe {
        LAST_SYNC_TIME = None;
    }

    trigger_sync(env, context)
}

/// Check the last sync time from AnkiDroid preferences
pub fn check_last_sync_time(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<Option<u64>> {
    log::debug!("Checking last sync time from AnkiDroid");

    // Get SharedPreferences for AnkiDroid
    let prefs_name = env.new_string_checked("AnkiDroid")?;
    let mode = 0; // MODE_PRIVATE

    let prefs = env
        .env_mut()
        .call_method(
            context,
            "getSharedPreferences",
            "(Ljava/lang/String;I)Landroid/content/SharedPreferences;",
            &[JValue::Object(&prefs_name.into()), JValue::Int(mode)],
        )
        .check_exception(env.env_mut())?
        .l()
        .map_err(AndroidError::from)?;

    if prefs.is_null() {
        log::warn!("Could not access AnkiDroid SharedPreferences");
        return Ok(None);
    }

    // Get the last sync time value
    let key = env.new_string_checked("lastSyncTime")?;
    let default_value = 0i64;

    let last_sync = env
        .env_mut()
        .call_method(
            &prefs,
            "getLong",
            "(Ljava/lang/String;J)J",
            &[JValue::Object(&key.into()), JValue::Long(default_value)],
        )
        .check_exception(env.env_mut())?
        .j()
        .map_err(AndroidError::from)?;

    if last_sync > 0 {
        Ok(Some(last_sync as u64))
    } else {
        Ok(None)
    }
}

/// Start AnkiDroid with sync intent
pub fn start_ankidroid_with_sync(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<()> {
    log::info!("Starting AnkiDroid with sync intent");

    // Check if AnkiDroid is available
    if !check_ankidroid_available(env, context)? {
        return Err(AndroidError::AnkiDroidNotAvailable(
            "AnkiDroid is not installed".to_string(),
        ));
    }

    // Create intent to launch AnkiDroid
    let intent_class = env.find_class_checked("android/content/Intent")?;
    let intent = env
        .env_mut()
        .new_object(&intent_class, "()V", &[])
        .check_exception(env.env_mut())?;

    // Set action to MAIN
    let action_main = env.new_string_checked("android.intent.action.MAIN")?;
    env.env_mut()
        .call_method(
            &intent,
            "setAction",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&action_main.into())],
        )
        .check_exception(env.env_mut())?;

    // Set package and component to AnkiDroid's DeckPicker
    let package_name = env.new_string_checked("com.ichi2.anki")?;
    let class_name = env.new_string_checked("com.ichi2.anki.DeckPicker")?;

    env.env_mut()
        .call_method(
            &intent,
            "setClassName",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            &[
                JValue::Object(&package_name.into()),
                JValue::Object(&class_name.into()),
            ],
        )
        .check_exception(env.env_mut())?;

    // Add extra to trigger sync
    let extra_key = env.new_string_checked("DO_SYNC")?;
    env.env_mut()
        .call_method(
            &intent,
            "putExtra",
            "(Ljava/lang/String;Z)Landroid/content/Intent;",
            &[JValue::Object(&extra_key.into()), JValue::Bool(1)],
        )
        .check_exception(env.env_mut())?;

    // Add FLAG_ACTIVITY_NEW_TASK
    let flag = 0x10000000; // FLAG_ACTIVITY_NEW_TASK
    env.env_mut()
        .call_method(
            &intent,
            "addFlags",
            "(I)Landroid/content/Intent;",
            &[JValue::Int(flag)],
        )
        .check_exception(env.env_mut())?;

    // Start the activity
    env.env_mut()
        .call_method(
            context,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&intent)],
        )
        .check_exception(env.env_mut())?;

    log::info!("AnkiDroid started with sync intent");
    Ok(())
}

/// Sync status information
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub can_sync: bool,
    pub cooldown_remaining: u64,
    pub last_sync_time: Option<u64>,
}

/// Get the current sync status
pub fn get_sync_status(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<SyncStatus> {
    let cooldown_remaining = get_sync_cooldown_remaining();
    let can_sync = check_ankidroid_available(env, context)? && cooldown_remaining == 0;
    let last_sync_time = check_last_sync_time(env, context)?;

    Ok(SyncStatus {
        can_sync,
        cooldown_remaining,
        last_sync_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_cooldown() {
        // Reset sync time
        unsafe {
            LAST_SYNC_TIME = None;
        }

        // Should be able to sync initially
        assert!(check_sync_cooldown().unwrap());

        // Update sync time
        update_last_sync_time();

        // Should not be able to sync immediately after
        assert!(!check_sync_cooldown().unwrap());

        // Reset for other tests
        unsafe {
            LAST_SYNC_TIME = None;
        }
    }

    #[test]
    fn test_cooldown_remaining() {
        // Reset sync time
        unsafe {
            LAST_SYNC_TIME = None;
        }

        // No cooldown initially
        assert_eq!(get_sync_cooldown_remaining(), 0);

        // Update sync time
        update_last_sync_time();

        // Should have cooldown
        let remaining = get_sync_cooldown_remaining();
        assert!(remaining > 0 && remaining <= SYNC_COOLDOWN_SECONDS);

        // Reset for other tests
        unsafe {
            LAST_SYNC_TIME = None;
        }
    }
}

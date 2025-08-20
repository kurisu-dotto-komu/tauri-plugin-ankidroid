use crate::android::error::{AndroidError, AndroidResult, JniResultExt};
use crate::android::jni_helpers::SafeJNIEnv;
use jni::objects::{JObject, JValue};

/// Check if AnkiDroid is available on the device
pub fn check_ankidroid_available(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<bool> {
    log::debug!("Checking if AnkiDroid is available");

    // Try to find the AddContentApi class
    let api_class = match env.find_class_checked("com/ichi2/anki/api/AddContentApi") {
        Ok(class) => class,
        Err(_) => {
            log::warn!("AddContentApi class not found");
            return Ok(false);
        }
    };

    // Call AddContentApi.getAnkiDroidPackageName(context)
    let package_name = env
        .env_mut()
        .call_static_method(
            &api_class,
            "getAnkiDroidPackageName",
            "(Landroid/content/Context;)Ljava/lang/String;",
            &[JValue::Object(context)],
        )
        .check_exception(env.env_mut())?
        .l()
        .map_err(AndroidError::from)?;

    let is_available = !package_name.is_null();
    log::info!("AnkiDroid available: {}", is_available);

    Ok(is_available)
}

/// Check if we have permission to access AnkiDroid
pub fn check_permission(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<bool> {
    log::debug!("Checking AnkiDroid permission");

    // Check if AnkiDroid is available first
    if !check_ankidroid_available(env, context)? {
        return Ok(false);
    }

    // Check for permission using checkSelfPermission
    let permission_string =
        env.new_string_checked("com.ichi2.anki.permission.READ_WRITE_DATABASE")?;

    let permission_result = env
        .env_mut()
        .call_method(
            context,
            "checkSelfPermission",
            "(Ljava/lang/String;)I",
            &[JValue::Object(&permission_string.into())],
        )
        .check_exception(env.env_mut())?
        .i()
        .map_err(AndroidError::from)?;

    // PackageManager.PERMISSION_GRANTED = 0
    let has_permission = permission_result == 0;
    log::info!("AnkiDroid permission granted: {}", has_permission);

    Ok(has_permission)
}

/// Request permission to access AnkiDroid
pub fn request_permission(env: &mut SafeJNIEnv, activity: &JObject) -> AndroidResult<()> {
    log::info!("Requesting AnkiDroid permission");

    // Create string array with the permission
    let permission_string =
        env.new_string_checked("com.ichi2.anki.permission.READ_WRITE_DATABASE")?;
    let permission_string_2 =
        env.new_string_checked("com.ichi2.anki.permission.READ_WRITE_DATABASE")?;
    let _string_array_class = env.find_class_checked("[Ljava/lang/String;")?;
    let permissions_array = env
        .env_mut()
        .new_object_array(1, "java/lang/String", &JObject::from(permission_string))
        .check_exception(env.env_mut())?;

    env.env_mut()
        .set_object_array_element(&permissions_array, 0, JObject::from(permission_string_2))
        .check_exception(env.env_mut())?;

    // Request the permission
    let request_code = 1337; // Arbitrary request code
    env.env_mut()
        .call_method(
            activity,
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[
                JValue::Object(&permissions_array.into()),
                JValue::Int(request_code),
            ],
        )
        .check_exception(env.env_mut())?;

    log::info!("Permission request initiated");
    Ok(())
}

/// Get the API version of AnkiDroid
pub fn get_api_version(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<Option<i32>> {
    log::debug!("Getting AnkiDroid API version");

    if !check_ankidroid_available(env, context)? {
        return Ok(None);
    }

    // Try to get the API version from AddContentApi
    let api_class = env.find_class_checked("com/ichi2/anki/api/AddContentApi")?;

    // Call AddContentApi.getApiVersion(context)
    let version = env
        .env_mut()
        .call_static_method(
            &api_class,
            "getApiVersion",
            "(Landroid/content/Context;)I",
            &[JValue::Object(context)],
        )
        .check_exception(env.env_mut())?
        .i()
        .map_err(AndroidError::from)?;

    log::info!("AnkiDroid API version: {}", version);
    Ok(Some(version))
}

/// Get the AnkiDroid package name if available
pub fn get_ankidroid_package_name(
    env: &mut SafeJNIEnv,
    context: &JObject,
) -> AndroidResult<Option<String>> {
    log::debug!("Getting AnkiDroid package name");

    // Try to find the AddContentApi class
    let api_class = match env.find_class_checked("com/ichi2/anki/api/AddContentApi") {
        Ok(class) => class,
        Err(_) => {
            log::warn!("AddContentApi class not found");
            return Ok(None);
        }
    };

    // Call AddContentApi.getAnkiDroidPackageName(context)
    let package_name_obj = env
        .env_mut()
        .call_static_method(
            &api_class,
            "getAnkiDroidPackageName",
            "(Landroid/content/Context;)Ljava/lang/String;",
            &[JValue::Object(context)],
        )
        .check_exception(env.env_mut())?
        .l()
        .map_err(AndroidError::from)?;

    if package_name_obj.is_null() {
        return Ok(None);
    }

    let package_name = env.get_string_checked(&package_name_obj.into())?;
    log::info!("AnkiDroid package name: {}", package_name);

    Ok(Some(package_name))
}

/// Check if AnkiDroid is running
pub fn is_ankidroid_running(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<bool> {
    log::debug!("Checking if AnkiDroid is running");

    if !check_ankidroid_available(env, context)? {
        return Ok(false);
    }

    // Get ActivityManager
    let activity_string = env.new_string_checked("activity")?;
    let activity_manager = env
        .env_mut()
        .call_method(
            context,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::Object(&activity_string.into())],
        )
        .check_exception(env.env_mut())?
        .l()
        .map_err(AndroidError::from)?;

    if activity_manager.is_null() {
        return Ok(false);
    }

    // Get running app processes
    let running_processes = env
        .env_mut()
        .call_method(
            &activity_manager,
            "getRunningAppProcesses",
            "()Ljava/util/List;",
            &[],
        )
        .check_exception(env.env_mut())?
        .l()
        .map_err(AndroidError::from)?;

    if running_processes.is_null() {
        return Ok(false);
    }

    // Check if AnkiDroid is in the list
    let size = env
        .env_mut()
        .call_method(&running_processes, "size", "()I", &[])
        .check_exception(env.env_mut())?
        .i()
        .map_err(AndroidError::from)?;

    for i in 0..size {
        let process_info = env
            .env_mut()
            .call_method(
                &running_processes,
                "get",
                "(I)Ljava/lang/Object;",
                &[JValue::Int(i)],
            )
            .check_exception(env.env_mut())?
            .l()
            .map_err(AndroidError::from)?;

        if !process_info.is_null() {
            let process_name_obj = env
                .env_mut()
                .get_field(&process_info, "processName", "Ljava/lang/String;")
                .check_exception(env.env_mut())?
                .l()
                .map_err(AndroidError::from)?;

            if !process_name_obj.is_null() {
                let process_name = env.get_string_checked(&process_name_obj.into())?;
                if process_name == "com.ichi2.anki" {
                    log::info!("AnkiDroid is running");
                    return Ok(true);
                }
            }
        }
    }

    log::info!("AnkiDroid is not running");
    Ok(false)
}

/// Launch AnkiDroid if it's not running
pub fn launch_ankidroid(env: &mut SafeJNIEnv, context: &JObject) -> AndroidResult<()> {
    log::info!("Launching AnkiDroid");

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

    log::info!("AnkiDroid launch initiated");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_check_module() {
        // Basic test to ensure the module compiles
        assert!(true);
    }
}

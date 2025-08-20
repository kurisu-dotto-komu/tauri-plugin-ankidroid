use crate::android::constants::{
    media_columns, ANKIDROID_DEBUG_PACKAGE, ANKIDROID_PACKAGE, MEDIA_URI,
};
use crate::android::error::{AndroidError, AndroidResult, JniResultExt};
use crate::android::jni_helpers::{
    attach_current_thread, get_android_context, get_content_resolver, parse_uri, SafeJNIEnv,
    StringHelper,
};
use jni::objects::{JObject, JValue};
use log::{debug, info, warn};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Media operations handler for AnkiDroid integration
pub struct MediaHandler;

impl MediaHandler {
    /// Add media from raw bytes data
    ///
    /// # Arguments
    /// * `data` - Raw bytes of the media file
    /// * `filename` - Preferred filename for the media
    /// * `mime_type` - MIME type of the media (e.g., "image/png", "audio/mpeg")
    ///
    /// # Returns
    /// * `AndroidResult<MediaAddResult>` - Result containing the URI and actual filename used
    pub fn add_media(
        data: Vec<u8>,
        filename: &str,
        mime_type: &str,
    ) -> AndroidResult<MediaAddResult> {
        info!(
            "Adding media from bytes: filename={}, size={} bytes, mime_type={}",
            filename,
            data.len(),
            mime_type
        );

        if data.is_empty() {
            return Err(AndroidError::MediaError("Media data is empty".to_string()));
        }

        if filename.is_empty() {
            return Err(AndroidError::MediaError(
                "Filename cannot be empty".to_string(),
            ));
        }

        // Save to cache directory first
        let cache_file_path = Self::save_to_cache(data, filename)?;
        debug!("Media saved to cache: {}", cache_file_path.display());

        // Get FileProvider URI and add to AnkiDroid
        let file_uri = Self::get_file_provider_uri(&cache_file_path)?;
        debug!("FileProvider URI created: {}", file_uri);

        // Grant permission to AnkiDroid
        Self::grant_uri_permission(&file_uri)?;
        debug!("URI permission granted to AnkiDroid");

        // Add to AnkiDroid using ContentProvider
        let actual_filename = Self::insert_media_content(&file_uri, filename)?;
        debug!(
            "Media added to AnkiDroid with filename: {}",
            actual_filename
        );

        Ok(MediaAddResult {
            uri: file_uri,
            filename: actual_filename,
        })
    }

    /// Add media from URL by downloading it first
    ///
    /// # Arguments
    /// * `url` - URL to download the media from
    /// * `filename` - Preferred filename for the media
    ///
    /// # Returns
    /// * `AndroidResult<MediaAddResult>` - Result containing the URI and actual filename used
    pub fn add_media_from_url(url: &str, filename: &str) -> AndroidResult<MediaAddResult> {
        info!("Adding media from URL: {} -> {}", url, filename);

        if url.is_empty() {
            return Err(AndroidError::MediaError("URL cannot be empty".to_string()));
        }

        // Download the media data
        let data = Self::download_from_url(url)?;
        debug!("Downloaded {} bytes from URL", data.len());

        // Determine MIME type from file extension or URL
        let mime_type = Self::determine_mime_type_from_filename(filename)
            .or_else(|| Self::determine_mime_type_from_url(url))
            .unwrap_or("application/octet-stream");

        // Use the existing add_media function
        Self::add_media(data, filename, mime_type)
    }

    /// Add media from base64 encoded data
    ///
    /// # Arguments
    /// * `base64_data` - Base64 encoded media data
    /// * `filename` - Preferred filename for the media
    /// * `mime_type` - MIME type of the media
    ///
    /// # Returns
    /// * `AndroidResult<MediaAddResult>` - Result containing the URI and actual filename used
    pub fn add_media_from_base64(
        base64_data: &str,
        filename: &str,
        mime_type: &str,
    ) -> AndroidResult<MediaAddResult> {
        info!(
            "Adding media from base64: filename={}, mime_type={}",
            filename, mime_type
        );

        if base64_data.is_empty() {
            return Err(AndroidError::MediaError(
                "Base64 data cannot be empty".to_string(),
            ));
        }

        // Decode base64 data
        let data = Self::decode_base64(base64_data)?;
        debug!("Decoded {} bytes from base64", data.len());

        // Use the existing add_media function
        Self::add_media(data, filename, mime_type)
    }

    /// Get the app's cache directory
    ///
    /// # Returns
    /// * `AndroidResult<PathBuf>` - Path to the cache directory
    pub fn get_cache_dir() -> AndroidResult<PathBuf> {
        debug!("Getting app cache directory");

        let env = attach_current_thread()?;
        let mut safe_env = SafeJNIEnv::new(env);
        let (_, activity) = get_android_context()?;

        // Get cache directory from Android context
        let result = safe_env
            .env_mut()
            .call_method(&activity, "getCacheDir", "()Ljava/io/File;", &[])
            .check_exception(safe_env.env_mut())?;

        let cache_dir_file = result.l().map_err(AndroidError::from)?;

        if cache_dir_file.is_null() {
            return Err(AndroidError::MediaError(
                "Cache directory is null".to_string(),
            ));
        }

        // Get the absolute path
        let path_result = safe_env
            .env_mut()
            .call_method(
                &cache_dir_file,
                "getAbsolutePath",
                "()Ljava/lang/String;",
                &[],
            )
            .check_exception(safe_env.env_mut())?;

        let path_string = StringHelper::jobject_to_rust(
            &mut safe_env,
            &path_result.l().map_err(AndroidError::from)?,
        )?;
        let cache_path = PathBuf::from(path_string);

        debug!("Cache directory: {}", cache_path.display());
        Ok(cache_path)
    }

    /// Get FileProvider URI for a file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file
    ///
    /// # Returns
    /// * `AndroidResult<String>` - FileProvider URI as string
    pub fn get_file_provider_uri(file_path: &Path) -> AndroidResult<String> {
        debug!("Getting FileProvider URI for: {}", file_path.display());

        let env = attach_current_thread()?;
        let mut safe_env = SafeJNIEnv::new(env);
        let (_, activity) = get_android_context()?;

        // Create File object
        let _file_class = safe_env.find_class_checked("java/io/File")?;
        let file_path_str = file_path
            .to_str()
            .ok_or_else(|| AndroidError::MediaError("Invalid file path".to_string()))?;
        let file_path_jstring = safe_env.new_string_checked(file_path_str)?;

        let file_obj = safe_env.new_object_checked(
            "java/io/File",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&file_path_jstring.into())],
        )?;

        // Get FileProvider URI using androidx.core.content.FileProvider
        let file_provider_class =
            safe_env.find_class_checked("androidx/core/content/FileProvider")?;
        let authority_jstring =
            safe_env.new_string_checked("com.tauri.plugin.ankidroid.fileprovider")?;

        let uri_result = safe_env
            .env_mut()
            .call_static_method(
                &file_provider_class,
                "getUriForFile",
                "(Landroid/content/Context;Ljava/lang/String;Ljava/io/File;)Landroid/net/Uri;",
                &[
                    JValue::Object(&activity),
                    JValue::Object(&authority_jstring.into()),
                    JValue::Object(&file_obj),
                ],
            )
            .check_exception(safe_env.env_mut())?;

        let uri_obj = uri_result.l().map_err(AndroidError::from)?;
        if uri_obj.is_null() {
            return Err(AndroidError::MediaError(
                "Failed to create FileProvider URI".to_string(),
            ));
        }

        // Convert URI to string
        let uri_string_result = safe_env
            .env_mut()
            .call_method(&uri_obj, "toString", "()Ljava/lang/String;", &[])
            .check_exception(safe_env.env_mut())?;

        let uri_string = StringHelper::jobject_to_rust(
            &mut safe_env,
            &uri_string_result.l().map_err(AndroidError::from)?,
        )?;
        debug!("FileProvider URI created: {}", uri_string);

        Ok(uri_string)
    }

    /// Grant URI permission to AnkiDroid package
    ///
    /// # Arguments
    /// * `uri` - URI to grant permission for
    ///
    /// # Returns
    /// * `AndroidResult<()>` - Success or error
    pub fn grant_uri_permission(uri: &str) -> AndroidResult<()> {
        debug!("Granting URI permission to AnkiDroid for: {}", uri);

        let env = attach_current_thread()?;
        let mut safe_env = SafeJNIEnv::new(env);
        let (_, activity) = get_android_context()?;

        let uri_obj = parse_uri(&mut safe_env, uri)?;

        // Try both regular and debug packages
        let packages = [ANKIDROID_PACKAGE, ANKIDROID_DEBUG_PACKAGE];
        let mut granted = false;

        for package in &packages {
            match Self::grant_permission_to_package(&mut safe_env, &activity, &uri_obj, package) {
                Ok(()) => {
                    debug!("Permission granted to package: {}", package);
                    granted = true;
                    break;
                }
                Err(e) => {
                    warn!("Failed to grant permission to {}: {}", package, e);
                }
            }
        }

        if !granted {
            return Err(AndroidError::MediaError(
                "Failed to grant permission to any AnkiDroid package".to_string(),
            ));
        }

        info!("URI permission granted to AnkiDroid");
        Ok(())
    }

    // Private helper methods

    /// Save media data to cache directory
    fn save_to_cache(data: Vec<u8>, filename: &str) -> AndroidResult<PathBuf> {
        let cache_dir = Self::get_cache_dir()?;
        let media_cache_dir = cache_dir.join("media");

        // Create media subdirectory if it doesn't exist
        create_dir_all(&media_cache_dir).map_err(|e| {
            AndroidError::MediaError(format!("Failed to create media cache directory: {}", e))
        })?;

        // Generate unique filename to avoid conflicts
        let file_path = Self::generate_unique_filename(&media_cache_dir, filename)?;

        // Write data to file
        let mut file = File::create(&file_path)
            .map_err(|e| AndroidError::MediaError(format!("Failed to create cache file: {}", e)))?;

        file.write_all(&data).map_err(|e| {
            AndroidError::MediaError(format!("Failed to write to cache file: {}", e))
        })?;

        file.sync_all()
            .map_err(|e| AndroidError::MediaError(format!("Failed to sync cache file: {}", e)))?;

        debug!("Media saved to cache: {}", file_path.display());
        Ok(file_path)
    }

    /// Generate a unique filename in the specified directory
    fn generate_unique_filename(dir: &Path, preferred_name: &str) -> AndroidResult<PathBuf> {
        let mut file_path = dir.join(preferred_name);
        let mut counter = 1;

        // If file doesn't exist, use the preferred name
        if !file_path.exists() {
            return Ok(file_path);
        }

        // Extract name and extension
        let stem = Path::new(preferred_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("media");
        let extension = Path::new(preferred_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Generate unique name with counter
        while file_path.exists() {
            let unique_name = if extension.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, extension)
            };
            file_path = dir.join(unique_name);
            counter += 1;

            if counter > 1000 {
                return Err(AndroidError::MediaError(
                    "Failed to generate unique filename after 1000 attempts".to_string(),
                ));
            }
        }

        Ok(file_path)
    }

    /// Insert media content into AnkiDroid via ContentProvider
    fn insert_media_content(file_uri: &str, preferred_name: &str) -> AndroidResult<String> {
        debug!(
            "Inserting media content: uri={}, preferred_name={}",
            file_uri, preferred_name
        );

        let env = attach_current_thread()?;
        let mut safe_env = SafeJNIEnv::new(env);
        let (_, activity) = get_android_context()?;

        let content_resolver = get_content_resolver(&mut safe_env, &activity)?;
        let media_uri = parse_uri(&mut safe_env, MEDIA_URI)?;

        // Build ContentValues manually to avoid borrowing issues
        let content_values_class = safe_env.find_class_checked("android/content/ContentValues")?;
        let content_values = safe_env
            .env_mut()
            .new_object(content_values_class, "()V", &[])
            .check_exception(safe_env.env_mut())?;

        // Add FILE_URI
        let uri_key = safe_env.new_string_checked(media_columns::FILE_URI)?;
        let uri_value = safe_env.new_string_checked(file_uri)?;
        safe_env
            .env_mut()
            .call_method(
                &content_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(&uri_key.into()),
                    JValue::Object(&uri_value.into()),
                ],
            )
            .check_exception(safe_env.env_mut())?;

        // Add PREFERRED_NAME
        let name_key = safe_env.new_string_checked(media_columns::PREFERRED_NAME)?;
        let name_value = safe_env.new_string_checked(preferred_name)?;
        safe_env
            .env_mut()
            .call_method(
                &content_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(&name_key.into()),
                    JValue::Object(&name_value.into()),
                ],
            )
            .check_exception(safe_env.env_mut())?;

        // Insert into ContentProvider
        let result_uri = safe_env
            .env_mut()
            .call_method(
                &content_resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[JValue::Object(&media_uri), JValue::Object(&content_values)],
            )
            .check_exception(safe_env.env_mut())?;

        let inserted_uri = result_uri.l().map_err(AndroidError::from)?;

        if inserted_uri.is_null() {
            return Err(AndroidError::MediaError(
                "Failed to insert media into AnkiDroid".to_string(),
            ));
        }

        // Extract the actual filename from the result URI
        let actual_filename = Self::extract_filename_from_uri(&mut safe_env, &inserted_uri)?;
        debug!("Media inserted with actual filename: {}", actual_filename);

        Ok(actual_filename)
    }

    /// Extract filename from result URI
    fn extract_filename_from_uri(env: &mut SafeJNIEnv, uri: &JObject) -> AndroidResult<String> {
        // Get the last path segment which should contain the filename
        let last_segment_result = env
            .env_mut()
            .call_method(uri, "getLastPathSegment", "()Ljava/lang/String;", &[])
            .check_exception(env.env_mut())?;

        let segment_obj = last_segment_result.l().map_err(AndroidError::from)?;

        if segment_obj.is_null() {
            return Ok("unknown".to_string());
        }

        StringHelper::jobject_to_rust(env, &segment_obj)
    }

    /// Grant permission to a specific package
    fn grant_permission_to_package(
        env: &mut SafeJNIEnv,
        activity: &JObject,
        uri: &JObject,
        package_name: &str,
    ) -> AndroidResult<()> {
        let package_jstring = env.new_string_checked(package_name)?;
        let read_write_flag = 3i32; // Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION

        env.env_mut()
            .call_method(
                activity,
                "grantUriPermission",
                "(Ljava/lang/String;Landroid/net/Uri;I)V",
                &[
                    JValue::Object(&package_jstring.into()),
                    JValue::Object(uri),
                    JValue::Int(read_write_flag),
                ],
            )
            .check_exception(env.env_mut())?;

        Ok(())
    }

    /// Download data from URL - simplified implementation
    fn download_from_url(url: &str) -> AndroidResult<Vec<u8>> {
        debug!("Downloading from URL: {}", url);

        // For now, return an error indicating this needs to be implemented
        // A full HTTP implementation would be quite complex in JNI
        Err(AndroidError::MediaError(
            "URL download not yet implemented - use add_media or add_media_from_base64 instead"
                .to_string(),
        ))
    }

    /// Decode base64 data
    fn decode_base64(base64_data: &str) -> AndroidResult<Vec<u8>> {
        use base64::{engine::general_purpose, Engine as _};

        general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| AndroidError::MediaError(format!("Base64 decode error: {}", e)))
    }

    /// Determine MIME type from filename extension
    fn determine_mime_type_from_filename(filename: &str) -> Option<&'static str> {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())?;

        match extension.as_str() {
            "jpg" | "jpeg" => Some("image/jpeg"),
            "png" => Some("image/png"),
            "gif" => Some("image/gif"),
            "webp" => Some("image/webp"),
            "svg" => Some("image/svg+xml"),
            "mp3" => Some("audio/mpeg"),
            "wav" => Some("audio/wav"),
            "ogg" => Some("audio/ogg"),
            "m4a" => Some("audio/mp4"),
            "flac" => Some("audio/flac"),
            "mp4" => Some("video/mp4"),
            "webm" => Some("video/webm"),
            "avi" => Some("video/x-msvideo"),
            "mov" => Some("video/quicktime"),
            "pdf" => Some("application/pdf"),
            "txt" => Some("text/plain"),
            "html" | "htm" => Some("text/html"),
            "css" => Some("text/css"),
            "js" => Some("application/javascript"),
            "json" => Some("application/json"),
            "xml" => Some("application/xml"),
            _ => None,
        }
    }

    /// Determine MIME type from URL extension
    fn determine_mime_type_from_url(url: &str) -> Option<&'static str> {
        // Extract filename from URL
        if let Some(filename) = url.split('/').last() {
            // Remove query parameters
            let filename = filename.split('?').next().unwrap_or(filename);
            Self::determine_mime_type_from_filename(filename)
        } else {
            None
        }
    }
}

/// Result of media addition operation
#[derive(Debug, Clone)]
pub struct MediaAddResult {
    /// The URI of the added media (FileProvider URI)
    pub uri: String,
    /// The actual filename used by AnkiDroid (may differ from preferred name)
    pub filename: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_type_detection() {
        assert_eq!(
            MediaHandler::determine_mime_type_from_filename("image.jpg"),
            Some("image/jpeg")
        );
        assert_eq!(
            MediaHandler::determine_mime_type_from_filename("audio.mp3"),
            Some("audio/mpeg")
        );
        assert_eq!(
            MediaHandler::determine_mime_type_from_filename("document.pdf"),
            Some("application/pdf")
        );
        assert_eq!(
            MediaHandler::determine_mime_type_from_filename("unknown.xyz"),
            None
        );
    }

    #[test]
    fn test_mime_type_from_url() {
        assert_eq!(
            MediaHandler::determine_mime_type_from_url("https://example.com/image.png"),
            Some("image/png")
        );
        assert_eq!(
            MediaHandler::determine_mime_type_from_url("https://example.com/audio.mp3?param=value"),
            Some("audio/mpeg")
        );
        assert_eq!(
            MediaHandler::determine_mime_type_from_url("https://example.com/"),
            None
        );
    }

    #[test]
    fn test_media_add_result() {
        let result = MediaAddResult {
            uri: "content://com.tauri.plugin.ankidroid.fileprovider/files/media/image.jpg"
                .to_string(),
            filename: "image_1.jpg".to_string(),
        };

        assert!(!result.uri.is_empty());
        assert!(!result.filename.is_empty());
        assert!(result.uri.contains("fileprovider"));
    }

    #[test]
    fn test_validation_errors() {
        // Test empty data
        let result = MediaHandler::add_media(vec![], "test.jpg", "image/jpeg");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Media data is empty"));

        // Test empty filename
        let result = MediaHandler::add_media(vec![1, 2, 3], "", "image/jpeg");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Filename cannot be empty"));
    }

    #[test]
    fn test_base64_validation() {
        let result = MediaHandler::add_media_from_base64("", "test.jpg", "image/jpeg");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Base64 data cannot be empty"));
    }

    #[test]
    fn test_url_validation() {
        let result = MediaHandler::add_media_from_url("", "test.jpg");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("URL cannot be empty"));
    }
}

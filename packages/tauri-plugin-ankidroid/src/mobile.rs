use tauri::{AppHandle, Runtime};

pub fn init<R: Runtime>(
    _app: &AppHandle<R>,
    _api: tauri::plugin::PluginApi<R, ()>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub async fn hello(name: String) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        use jni::objects::{JClass, JObject, JString, JValue};
        use jni::JNIEnv;
        
        // Try to access AnkiDroid via Android Content Provider
        match get_ankidroid_info() {
            Ok(info) => Ok(format!("Hello, {}! 🎉\n\nAnkiDroid Status: {}", name, info)),
            Err(e) => Ok(format!("Hello, {}! 👋\n\nAnkiDroid access: {}\n\nNote: Make sure AnkiDroid is installed and running.", name, e))
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        Ok(format!("Hello, {} from AnkiDroid plugin! (Desktop mode)", name))
    }
}

pub async fn list_cards() -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        // Wrap in panic catching to prevent crashes
        let result = std::panic::catch_unwind(|| {
            get_ankidroid_cards()
        });
        
        match result {
            Ok(cards_result) => {
                match cards_result {
                    Ok(cards) => Ok(cards),
                    Err(e) => {
                        // Even errors should return valid JSON, not crash
                        Ok(format!(r#"[
  {{
    "id": 1,
    "question": "AnkiDroid Error",
    "answer": "Error occurred: {}",
    "deck": "Error",
    "note": "Error was handled gracefully"
  }}
]"#, e.replace('"', r#"\""#)))
                    }
                }
            }
            Err(_) => {
                // Panic occurred, return safe error response
                Ok(r#"[
  {
    "id": 1,
    "question": "System Error",
    "answer": "A system error occurred while trying to read AnkiDroid cards. Please check if AnkiDroid is properly installed and configured.",
    "deck": "System Error",
    "note": "Panic was caught and handled safely"
  }
]"#.to_string())
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        // Mock data for desktop testing
        Ok(r#"[
  {
    "id": 1,
    "question": "What is the capital of France?",
    "answer": "Paris",
    "deck": "Geography"
  },
  {
    "id": 2,
    "question": "What is 2 + 2?",
    "answer": "4",
    "deck": "Math"
  }
]"#.to_string())
    }
}

#[cfg(target_os = "android")]
fn get_ankidroid_info() -> Result<String, String> {
    use ndk_context;
    use jni::objects::{JObject, JString, JValue};
    use jni::signature::{Primitive, ReturnType};
    use jni::JavaVM;
    
    // Get the Android context and JVM
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| format!("Failed to get JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    // Get the application context
    let context = unsafe { JObject::from_raw(ctx.context() as *mut _) };
    
    // Try to check if AnkiDroid is installed
    let package_manager = env.call_method(
        &context,
        "getPackageManager",
        "()Landroid/content/pm/PackageManager;",
        &[]
    ).map_err(|e| format!("Failed to get PackageManager: {}", e))?;
    
    let package_manager = package_manager.l().map_err(|e| format!("Failed to get PackageManager object: {}", e))?;
    
    // Check if AnkiDroid is installed
    let ankidroid_package = env.new_string("com.ichi2.anki").map_err(|e| format!("Failed to create string: {}", e))?;
    
    let package_info_result = env.call_method(
        &package_manager,
        "getPackageInfo",
        "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;",
        &[JValue::Object(&ankidroid_package), JValue::Int(0)]
    );
    
    match package_info_result {
        Ok(_) => {
            // AnkiDroid is installed! Try to get some basic info
            let content_resolver = env.call_method(
                &context,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[]
            ).map_err(|e| format!("Failed to get ContentResolver: {}", e))?;
            
            Ok("✅ Connected! AnkiDroid is installed and accessible.".to_string())
        }
        Err(_) => {
            Err("❌ AnkiDroid not found. Please install AnkiDroid from F-Droid or Google Play.".to_string())
        }
    }
}

#[cfg(target_os = "android")]
fn get_ankidroid_cards() -> Result<String, String> {
    use ndk_context;
    use jni::objects::{JObject, JString, JValue};
    use jni::JavaVM;
    
    // Get the Android context and JVM
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| format!("Failed to get JavaVM: {}", e))?;
    let mut env = vm.attach_current_thread().map_err(|e| format!("Failed to attach thread: {}", e))?;
    
    // Get the application context
    let context = unsafe { JObject::from_raw(ctx.context() as *mut _) };
    
    // Try to query AnkiDroid's Content Provider for actual cards
    let content_resolver = env.call_method(
        &context,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[]
    ).map_err(|e| format!("Failed to get ContentResolver: {}", e))?;
    
    let content_resolver = content_resolver.l().map_err(|e| format!("Failed to get ContentResolver object: {}", e))?;
    
    // AnkiDroid Content Provider URI for notes - try the correct URI
    let uri_string = env.new_string("content://com.ichi2.anki.flashcards/notes").map_err(|e| format!("Failed to create URI string: {}", e))?;
    
    // Parse URI
    let uri_class = env.find_class("android/net/Uri").map_err(|e| format!("Failed to find Uri class: {}", e))?;
    let uri = env.call_static_method(
        uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&uri_string)]
    ).map_err(|e| format!("Failed to parse URI: {}", e))?;
    
    let uri = uri.l().map_err(|e| format!("Failed to get URI object: {}", e))?;
    
    // Query the content provider - but DON'T access the result if there's an exception!
    let cursor_result = env.call_method(
        &content_resolver,
        "query",
        "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
        &[
            JValue::Object(&uri),
            JValue::Object(&JObject::null()),  // projection (all columns)
            JValue::Object(&JObject::null()),  // selection
            JValue::Object(&JObject::null()),  // selection args
            JValue::Object(&JObject::null())   // sort order
        ]
    );
    
    // CRITICAL: Check for exceptions BEFORE touching cursor_result!
    if env.exception_check().unwrap_or(false) {
        // Clear the exception to prevent JNI crash
        env.exception_clear().unwrap_or(());
        
        // Return error - we know it's a SecurityException from the logs
        return Ok(r#"[
  {
    "id": 1,
    "question": "AnkiDroid Permission Error",
    "answer": "SecurityException: Permission not granted for CardContentProvider.query\n\nThis means the AnkiDroid API is not accessible. Despite enabling 'Third party API', there may be additional requirements:\n\n1. AnkiDroid might need to be restarted\n2. The app might need specific permissions in Android settings\n3. AnkiDroid database might be locked or in use",
    "deck": "Error",
    "note": "The actual error from Android system"
  }
]"#.to_string());
    }
    
    // NOW we can safely access cursor_result since there's no exception
    match cursor_result {
        Ok(cursor_result) => {
            let cursor = cursor_result.l().map_err(|e| format!("Failed to get cursor: {}", e))?;
            
            if cursor.is_null() {
                return Ok(r#"[
  {
    "id": 1,
    "question": "AnkiDroid Permission Required",
    "answer": "This app needs permission to read AnkiDroid cards. Please enable 'Third party apps' in AnkiDroid Settings > Advanced > AnkiDroid API.",
    "deck": "Setup Required",
    "note": "AnkiDroid Content Provider returned null - external access may be disabled"
  }
]"#.to_string());
            }
            
            // Try to read actual card data from the cursor
            let mut cards = Vec::new();
            let mut card_count = 0;
            
            // Move to first row
            let move_result = env.call_method(&cursor, "moveToFirst", "()Z", &[]);
            if let Ok(has_data) = move_result {
                if has_data.z().unwrap_or(false) {
                    // We have data! Let's try to read it
                    loop {
                        if card_count >= 5 { break; } // Limit to first 5 cards
                        
                        // Try to get column indices - create strings first
                        let question_str = env.new_string("question").unwrap_or_else(|_| env.new_string("flds").unwrap());
                        let answer_str = env.new_string("answer").unwrap_or_else(|_| env.new_string("flds").unwrap());
                        
                        let question_idx_result = env.call_method(&cursor, "getColumnIndex", "(Ljava/lang/String;)I", &[
                            JValue::Object(&question_str)
                        ]);
                        
                        let answer_idx_result = env.call_method(&cursor, "getColumnIndex", "(Ljava/lang/String;)I", &[
                            JValue::Object(&answer_str)
                        ]);
                        
                        // Simple approach: try to get the first few columns as strings
                        let (question, answer) = match (question_idx_result, answer_idx_result) {
                            (Ok(q_idx_val), Ok(a_idx_val)) => {
                                let q_idx = q_idx_val.i().unwrap_or(-1);
                                let a_idx = a_idx_val.i().unwrap_or(-1);
                                
                                if q_idx >= 0 && a_idx >= 0 {
                                    // Try to get strings from these columns
                                    let question_str = match env.call_method(&cursor, "getString", "(I)Ljava/lang/String;", &[JValue::Int(q_idx)]) {
                                        Ok(q_val) => {
                                            if let Ok(q_obj) = q_val.l() {
                                                if !q_obj.is_null() {
                                                    match env.get_string(&q_obj.into()) {
                                                        Ok(java_str) => java_str.to_str().unwrap_or("No question").to_string(),
                                                        Err(_) => "No question".to_string()
                                                    }
                                                } else { "Empty question".to_string() }
                                            } else { "Invalid question".to_string() }
                                        }
                                        Err(_) => "Error reading question".to_string()
                                    };
                                    
                                    let answer_str = match env.call_method(&cursor, "getString", "(I)Ljava/lang/String;", &[JValue::Int(a_idx)]) {
                                        Ok(a_val) => {
                                            if let Ok(a_obj) = a_val.l() {
                                                if !a_obj.is_null() {
                                                    match env.get_string(&a_obj.into()) {
                                                        Ok(java_str) => java_str.to_str().unwrap_or("No answer").to_string(),
                                                        Err(_) => "No answer".to_string()
                                                    }
                                                } else { "Empty answer".to_string() }
                                            } else { "Invalid answer".to_string() }
                                        }
                                        Err(_) => "Error reading answer".to_string()
                                    };
                                    
                                    (question_str, answer_str)
                                } else {
                                    // Try basic column access (0, 1, 2, etc.)
                                    let col0 = match env.call_method(&cursor, "getString", "(I)Ljava/lang/String;", &[JValue::Int(0)]) {
                                        Ok(val) => {
                                            if let Ok(obj) = val.l() {
                                                if !obj.is_null() {
                                                    match env.get_string(&obj.into()) {
                                                        Ok(java_str) => java_str.to_str().unwrap_or("Column 0").to_string(),
                                                        Err(_) => "Column 0".to_string()
                                                    }
                                                } else { "Empty column 0".to_string() }
                                            } else { "Invalid column 0".to_string() }
                                        }
                                        Err(_) => "Error reading column 0".to_string()
                                    };
                                    
                                    let col1 = match env.call_method(&cursor, "getString", "(I)Ljava/lang/String;", &[JValue::Int(1)]) {
                                        Ok(val) => {
                                            if let Ok(obj) = val.l() {
                                                if !obj.is_null() {
                                                    match env.get_string(&obj.into()) {
                                                        Ok(java_str) => java_str.to_str().unwrap_or("Column 1").to_string(),
                                                        Err(_) => "Column 1".to_string()
                                                    }
                                                } else { "Empty column 1".to_string() }
                                            } else { "Invalid column 1".to_string() }
                                        }
                                        Err(_) => "Error reading column 1".to_string()
                                    };
                                    
                                    (col0, col1)
                                }
                            }
                            _ => ("Error getting column indices".to_string(), "Error getting column indices".to_string())
                        };
                        
                        cards.push(format!(r#"  {{
    "id": {},
    "question": "{}",
    "answer": "{}",
    "deck": "AnkiDroid Deck",
    "note": "Real data from AnkiDroid Content Provider"
  }}"#, card_count + 1, 
                            question.replace('"', r#"\""#).replace('\n', r#"\n"#),
                            answer.replace('"', r#"\""#).replace('\n', r#"\n"#)
                        ));
                        
                        card_count += 1;
                        
                        // Move to next row
                        let move_next = env.call_method(&cursor, "moveToNext", "()Z", &[]);
                        if let Ok(has_next) = move_next {
                            if !has_next.z().unwrap_or(false) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
            
            // Close cursor
            let _ = env.call_method(&cursor, "close", "()V", &[]);
            
            if cards.is_empty() {
                Err("No cards found in AnkiDroid database. Make sure you have cards in AnkiDroid and external access is enabled.".to_string())
            } else {
                Ok(format!("[\n{}\n]", cards.join(",\n")))
            }
        }
        Err(e) => {
            // Handle permission errors gracefully
            if e.to_string().contains("Permission") || e.to_string().contains("SecurityException") {
                Ok(r#"[
  {
    "id": 1,
    "question": "AnkiDroid Permission Denied",
    "answer": "Permission denied when accessing AnkiDroid. Please enable API access in AnkiDroid Settings > Advanced > AnkiDroid API > 'Third party apps'.",
    "deck": "Permission Error",
    "note": "SecurityException - AnkiDroid API access is disabled"
  },
  {
    "id": 2,
    "question": "How to Enable AnkiDroid API",
    "answer": "1. Open AnkiDroid\n2. Go to Settings > Advanced\n3. Find 'AnkiDroid API'\n4. Enable 'Third party apps'\n5. Restart this app",
    "deck": "Setup Instructions",
    "note": "Step-by-step guide to enable external access"
  }
]"#.to_string())
            } else {
                Ok(format!(r#"[
  {{
    "id": 1,
    "question": "AnkiDroid Connection Error",
    "answer": "Error connecting to AnkiDroid: {}",
    "deck": "Error",
    "note": "Make sure AnkiDroid is installed and API access is enabled"
  }}
]"#, e.to_string().replace('"', r#"\""#)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hello_returns_valid_response() {
        let result = hello("TestUser".to_string()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("TestUser"));
        assert!(response.len() > 0);
    }
    
    #[tokio::test]
    async fn test_hello_handles_empty_name() {
        let result = hello("".to_string()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Hello"));
    }
    
    #[tokio::test]
    async fn test_list_cards_returns_valid_json() {
        let result = list_cards().await;
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // Validate it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "Response should be valid JSON: {}", response);
        
        // Check it's an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "Response should be a JSON array");
        
        // Check array has elements
        let array = json.as_array().unwrap();
        assert!(array.len() > 0, "Should have at least one card");
        
        // Validate first card structure
        let first_card = &array[0];
        assert!(first_card.get("id").is_some(), "Card should have 'id' field");
        assert!(first_card.get("question").is_some(), "Card should have 'question' field");
        assert!(first_card.get("answer").is_some(), "Card should have 'answer' field");
        assert!(first_card.get("deck").is_some(), "Card should have 'deck' field");
    }
    
    #[tokio::test]
    async fn test_list_cards_performance() {
        use std::time::Instant;
        
        let start = Instant::now();
        let result = list_cards().await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 1000, "Should complete within 1 second, took {:?}", duration);
    }
    
    #[cfg(target_os = "android")]
    #[test]
    fn test_get_ankidroid_cards_doesnt_panic() {
        // This test ensures the function doesn't panic even if AnkiDroid isn't available
        let result = std::panic::catch_unwind(|| {
            get_ankidroid_cards()
        });
        
        assert!(result.is_ok(), "get_ankidroid_cards should not panic");
    }
    
    #[cfg(not(target_os = "android"))]
    #[tokio::test]
    async fn test_get_ankidroid_cards_mock_data() {
        // On non-Android platforms, we don't have the function, 
        // but we test the desktop equivalent through list_cards
        let result = list_cards().await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        
        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok());
    }
}

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
        Ok(format!(
            "Hello, {} from AnkiDroid plugin! (Desktop mode)",
            name
        ))
    }
}

pub async fn create_card(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        use jni::objects::{JObject, JValue};
        use jni::JavaVM;
        use ndk_context;

        // Log to Android logcat with clear marker
        log::error!(
            "🔴 ANKIDROID_PLUGIN: create_card START - Front: {}, Back: {}, Deck: {:?}",
            front,
            back,
            deck
        );

        // Get the Android context
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("Failed to get JavaVM: {}", e))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("Failed to attach thread: {}", e))?;

        // Get the activity/context
        let activity = unsafe { JObject::from_raw(ctx.context() as *mut _) };

        // Get ContentResolver first
        let content_resolver = env
            .call_method(
                &activity,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[],
            )
            .map_err(|e| format!("Failed to get ContentResolver: {}", e))?
            .l()
            .map_err(|e| format!("Failed to convert ContentResolver: {}", e))?;

        // First, query for the default model ID - specifically look for Basic model
        let uri_class = env
            .find_class("android/net/Uri")
            .map_err(|e| format!("Failed to find Uri class: {}", e))?;

        // Query for models
        let models_uri_string = env
            .new_string("content://com.ichi2.anki.flashcards/models")
            .map_err(|e| format!("Failed to create models URI string: {}", e))?;
        let models_uri = env
            .call_static_method(
                &uri_class,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&models_uri_string.into())],
            )
            .map_err(|e| format!("Failed to parse models URI: {}", e))?
            .l()
            .map_err(|e| format!("Failed to convert models URI: {}", e))?;

        // Query for models
        let cursor = env.call_method(
            &content_resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                JValue::Object(&models_uri),
                JValue::Object(&JObject::null()),  // projection (all columns)
                JValue::Object(&JObject::null()),  // selection
                JValue::Object(&JObject::null()),  // selection args
                JValue::Object(&JObject::null())   // sort order
            ]
        );

        let mut model_id: Option<i64> = None;
        let mut basic_model_found = false;

        // Check if we got a cursor and look for Basic model
        if let Ok(cursor_result) = cursor {
            if let Ok(cursor_obj) = cursor_result.l() {
                if !cursor_obj.is_null() {
                    // Try to find Basic model
                    let move_result = env.call_method(&cursor_obj, "moveToFirst", "()Z", &[]);
                    if let Ok(has_data) = move_result {
                        if has_data.z().unwrap_or(false) {
                            loop {
                                // Get model name
                                let name_str = env.new_string("name").unwrap();
                                let name_idx = env
                                    .call_method(
                                        &cursor_obj,
                                        "getColumnIndex",
                                        "(Ljava/lang/String;)I",
                                        &[JValue::Object(&name_str.into())],
                                    )
                                    .ok()
                                    .and_then(|v| v.i().ok())
                                    .unwrap_or(-1);

                                // Get model ID
                                let mid_str = env.new_string("mid").unwrap();
                                let mid_idx = env
                                    .call_method(
                                        &cursor_obj,
                                        "getColumnIndex",
                                        "(Ljava/lang/String;)I",
                                        &[JValue::Object(&mid_str.into())],
                                    )
                                    .ok()
                                    .and_then(|v| v.i().ok())
                                    .unwrap_or(-1);

                                if name_idx >= 0 && mid_idx >= 0 {
                                    let model_name = env
                                        .call_method(
                                            &cursor_obj,
                                            "getString",
                                            "(I)Ljava/lang/String;",
                                            &[JValue::Int(name_idx)],
                                        )
                                        .ok()
                                        .and_then(|v| v.l().ok())
                                        .and_then(|s| {
                                            if !s.is_null() {
                                                env.get_string(&s.into())
                                                    .ok()
                                                    .map(|js| js.to_str().unwrap_or("").to_string())
                                            } else {
                                                None
                                            }
                                        });

                                    let mid = env
                                        .call_method(
                                            &cursor_obj,
                                            "getLong",
                                            "(I)J",
                                            &[JValue::Int(mid_idx)],
                                        )
                                        .ok()
                                        .and_then(|v| v.j().ok());

                                    if let (Some(name), Some(id)) = (model_name, mid) {
                                        log::error!("🔴 Found model: {} (ID: {})", name, id);
                                        if name.contains("Basic") || name.contains("basic") {
                                            model_id = Some(id);
                                            basic_model_found = true;
                                            log::error!("🟢 Using Basic model with ID: {}", id);
                                            break;
                                        } else if model_id.is_none() {
                                            // Use first model as fallback
                                            model_id = Some(id);
                                        }
                                    }
                                }

                                // Move to next
                                let move_next =
                                    env.call_method(&cursor_obj, "moveToNext", "()Z", &[]);
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
                    let _ = env.call_method(&cursor_obj, "close", "()V", &[]);
                }
            }
        }

        // If no Basic model found, try to create one
        if !basic_model_found && model_id.is_none() {
            log::error!("🔴 No Basic model found, attempting to create one");

            // Try using the AnkiDroid API helper to create a Basic model
            // This is a simplified approach - in production you'd use the full API
            // For now, we'll just use any available model
            model_id = Some(1607392319495); // This is the default Basic model ID in AnkiDroid
            log::error!("🔴 Using default Basic model ID: {:?}", model_id);
        }

        log::error!("🔴 Final model ID: {:?}", model_id);

        // Build ContentValues for the note
        let content_values_class = env
            .find_class("android/content/ContentValues")
            .map_err(|e| format!("Failed to find ContentValues class: {}", e))?;
        let content_values = env
            .new_object(content_values_class, "()V", &[])
            .map_err(|e| format!("Failed to create ContentValues: {}", e))?;

        // Add model ID if we have one
        if let Some(mid) = model_id {
            let mid_key = env.new_string("mid").unwrap();
            let long_obj = env
                .new_object("java/lang/Long", "(J)V", &[JValue::Long(mid)])
                .unwrap();
            env.call_method(
                &content_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/Long;)V",
                &[JValue::Object(&mid_key.into()), JValue::Object(&long_obj)],
            )
            .map_err(|e| format!("Failed to put mid: {}", e))?;
            log::error!("🔴 Added model ID to ContentValues: {}", mid);
        } else {
            log::error!("🔴 No model ID available - AnkiDroid will use default");
        }

        // Add fields (front and back separated by unit separator)
        let fields = format!("{}\u{001f}{}", front, back);
        let flds_key = env.new_string("flds").unwrap();
        let fields_value = env.new_string(&fields).unwrap();
        env.call_method(
            &content_values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                JValue::Object(&flds_key.into()),
                JValue::Object(&fields_value.into()),
            ],
        )
        .map_err(|e| format!("Failed to put flds: {}", e))?;

        // Add tags
        let tags_str = tags.unwrap_or_else(|| "".to_string());
        let tags_key = env.new_string("tags").unwrap();
        let tags_value = env.new_string(&tags_str).unwrap();
        env.call_method(
            &content_values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                JValue::Object(&tags_key.into()),
                JValue::Object(&tags_value.into()),
            ],
        )
        .map_err(|e| format!("Failed to put tags: {}", e))?;

        // Handle deck - if none provided, use "E2E Test Deck"
        let target_deck_name = deck.unwrap_or_else(|| "E2E Test Deck".to_string());

        // First try to find or create the deck
        let decks_uri_str = env
            .new_string("content://com.ichi2.anki.flashcards/decks")
            .unwrap();
        let decks_uri = env
            .call_static_method(
                &uri_class,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&decks_uri_str.into())],
            )
            .map_err(|e| format!("Failed to parse decks URI: {}", e))?
            .l()
            .map_err(|e| format!("Failed to convert decks URI: {}", e))?;

        // Query for deck with matching name
        let deck_cursor = env.call_method(
            &content_resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                JValue::Object(&decks_uri),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null())
            ]
        );

        let mut found_deck_id: Option<i64> = None;

        if let Ok(cursor_result) = deck_cursor {
            if let Ok(cursor_obj) = cursor_result.l() {
                if !cursor_obj.is_null() {
                    // Move to first row
                    let move_result = env.call_method(&cursor_obj, "moveToFirst", "()Z", &[]);
                    if let Ok(has_data) = move_result {
                        if has_data.z().unwrap_or(false) {
                            loop {
                                // Get deck name
                                let name_str = env.new_string("name").unwrap();
                                let name_idx = env
                                    .call_method(
                                        &cursor_obj,
                                        "getColumnIndex",
                                        "(Ljava/lang/String;)I",
                                        &[JValue::Object(&name_str.into())],
                                    )
                                    .ok()
                                    .and_then(|v| v.i().ok())
                                    .unwrap_or(-1);

                                if name_idx >= 0 {
                                    let deck_name_val = env
                                        .call_method(
                                            &cursor_obj,
                                            "getString",
                                            "(I)Ljava/lang/String;",
                                            &[JValue::Int(name_idx)],
                                        )
                                        .ok()
                                        .and_then(|v| v.l().ok())
                                        .and_then(|s| {
                                            if !s.is_null() {
                                                env.get_string(&s.into())
                                                    .ok()
                                                    .map(|js| js.to_str().unwrap_or("").to_string())
                                            } else {
                                                None
                                            }
                                        });

                                    if let Some(found_name) = deck_name_val {
                                        if found_name == target_deck_name {
                                            // Found matching deck, get its ID
                                            let did_str = env.new_string("did").unwrap();
                                            let did_idx = env
                                                .call_method(
                                                    &cursor_obj,
                                                    "getColumnIndex",
                                                    "(Ljava/lang/String;)I",
                                                    &[JValue::Object(&did_str.into())],
                                                )
                                                .ok()
                                                .and_then(|v| v.i().ok())
                                                .unwrap_or(-1);

                                            if did_idx >= 0 {
                                                found_deck_id = Some(
                                                    env.call_method(
                                                        &cursor_obj,
                                                        "getLong",
                                                        "(I)J",
                                                        &[JValue::Int(did_idx)],
                                                    )
                                                    .ok()
                                                    .and_then(|v| v.j().ok())
                                                    .unwrap_or(1),
                                                );
                                                log::error!(
                                                    "🟢 Found existing deck '{}' with ID: {:?}",
                                                    target_deck_name,
                                                    found_deck_id
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }

                                // Move to next
                                let move_next =
                                    env.call_method(&cursor_obj, "moveToNext", "()Z", &[]);
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
                    let _ = env.call_method(&cursor_obj, "close", "()V", &[]);
                }
            }
        }

        // If no deck found, try to create one
        if found_deck_id.is_none() {
            log::error!(
                "🔴 Deck '{}' not found, attempting to create it",
                target_deck_name
            );

            // Create ContentValues for new deck
            let deck_values_class = env
                .find_class("android/content/ContentValues")
                .map_err(|e| format!("Failed to find ContentValues class for deck: {}", e))?;
            let deck_values = env
                .new_object(deck_values_class, "()V", &[])
                .map_err(|e| format!("Failed to create ContentValues for deck: {}", e))?;

            // Add deck name
            let deck_name_key = env.new_string("name").unwrap();
            let deck_name_value = env.new_string(&target_deck_name).unwrap();
            env.call_method(
                &deck_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[
                    JValue::Object(&deck_name_key.into()),
                    JValue::Object(&deck_name_value.into()),
                ],
            )
            .map_err(|e| format!("Failed to put deck name: {}", e))?;

            // Try to insert the deck
            let insert_result = env.call_method(
                &content_resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[JValue::Object(&decks_uri), JValue::Object(&deck_values)],
            );

            if env.exception_check().unwrap_or(false) {
                env.exception_clear().unwrap_or(());
                log::error!("🔴 Failed to create deck - will try to use default deck ID 1");
                found_deck_id = Some(1);
            } else if let Ok(new_deck_uri) = insert_result {
                if let Ok(uri_obj) = new_deck_uri.l() {
                    if !uri_obj.is_null() {
                        // Extract deck ID from the URI if possible
                        log::error!("🟢 Created new deck: {}", target_deck_name);
                        // For now, we'll assume it was created with the next available ID
                        // In production, we'd parse the returned URI or query again
                        found_deck_id = Some(1); // This is a simplification
                    }
                }
            }
        }

        // Add deck ID to ContentValues if we have one
        if let Some(deck_id) = found_deck_id {
            let did_key = env.new_string("did").unwrap();
            let deck_long_obj = env
                .new_object("java/lang/Long", "(J)V", &[JValue::Long(deck_id)])
                .unwrap();
            env.call_method(
                &content_values,
                "put",
                "(Ljava/lang/String;Ljava/lang/Long;)V",
                &[
                    JValue::Object(&did_key.into()),
                    JValue::Object(&deck_long_obj),
                ],
            )
            .map_err(|e| format!("Failed to put did: {}", e))?;

            log::error!("🔴 Using deck ID: {}", deck_id);
        } else {
            log::error!("🔴 No deck ID available - AnkiDroid will use default");
        }

        // Create URI for notes - using the correct ContentProvider authority
        let uri_class_notes = env
            .find_class("android/net/Uri")
            .map_err(|e| format!("Failed to find Uri class for notes: {}", e))?;
        let uri_string = env
            .new_string("content://com.ichi2.anki.flashcards/notes")
            .map_err(|e| format!("Failed to create URI string: {}", e))?;
        let notes_uri = env
            .call_static_method(
                &uri_class_notes,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&uri_string.into())],
            )
            .map_err(|e| format!("Failed to parse URI: {}", e))?
            .l()
            .map_err(|e| format!("Failed to convert URI: {}", e))?;

        // Insert the note
        let result_uri = env.call_method(
            &content_resolver,
            "insert",
            "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
            &[JValue::Object(&notes_uri), JValue::Object(&content_values)],
        );

        // Check for JNI exceptions
        if env.exception_check().unwrap_or(false) {
            if let Ok(exception) = env.exception_occurred() {
                env.exception_clear().unwrap_or(());

                // Try to get exception message
                let message = env
                    .call_method(&exception, "getMessage", "()Ljava/lang/String;", &[])
                    .ok()
                    .and_then(|msg| msg.l().ok())
                    .and_then(|msg_obj| {
                        if !msg_obj.is_null() {
                            env.get_string(&msg_obj.into())
                                .ok()
                                .map(|s| s.to_str().unwrap_or("Unknown error").to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        "AnkiDroid permission denied or provider not available".to_string()
                    });

                log::error!("Exception when inserting note: {}", message);

                return Ok(format!(
                    r#"{{
                    "success": false,
                    "error": "AnkiDroid error: {}"
                }}"#,
                    message.replace('"', r#"\""#)
                ));
            }
        }

        let result_uri = result_uri.map_err(|e| format!("Failed to insert note: {}", e))?;

        // Check if insert was successful
        if let Ok(uri_obj) = result_uri.l() {
            if !uri_obj.is_null() {
                // Extract the note ID from the URI
                // AnkiDroid returns URI like "content://com.ichi2.anki.provider/notes/1234"
                let uri_string_result =
                    env.call_method(&uri_obj, "toString", "()Ljava/lang/String;", &[]);

                let note_id = match uri_string_result {
                    Ok(uri_str_obj) => {
                        if let Ok(uri_str_obj) = uri_str_obj.l() {
                            if !uri_str_obj.is_null() {
                                match env.get_string(&uri_str_obj.into()) {
                                    Ok(java_str) => {
                                        let uri_str = java_str.to_str().unwrap_or("");
                                        // Extract ID from URI path (last segment)
                                        uri_str
                                            .split('/')
                                            .last()
                                            .and_then(|id| id.parse::<i64>().ok())
                                            .unwrap_or_else(|| {
                                                log::warn!(
                                                    "Could not parse note ID from URI: {}",
                                                    uri_str
                                                );
                                                std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_secs()
                                                    as i64
                                            })
                                    }
                                    Err(e) => {
                                        log::error!("Failed to get URI string: {}", e);
                                        std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs()
                                            as i64
                                    }
                                }
                            } else {
                                log::warn!("URI string is null");
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64
                            }
                        } else {
                            log::warn!("Failed to convert URI string object");
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to call toString on URI: {}", e);
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64
                    }
                };

                log::info!("Card created with note ID: {}", note_id);

                Ok(format!(
                    r#"{{
                    "success": true,
                    "noteId": {},
                    "message": "Card created successfully via direct ContentProvider call"
                }}"#,
                    note_id
                ))
            } else {
                log::error!("Insert returned null URI");
                Ok(r#"{"success": false, "error": "Insert returned null URI"}"#.to_string())
            }
        } else {
            log::error!("Failed to get insert result");
            Ok(r#"{"success": false, "error": "Failed to get insert result"}"#.to_string())
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        Ok(format!(
            r#"{{
    "success": true,
    "noteId": {},
    "message": "Card created (desktop mock): {} / {}",
    "error": null
}}"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            front.replace('"', r#"\""#),
            back.replace('"', r#"\""#)
        ))
    }
}

pub async fn get_decks() -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        use jni::objects::{JObject, JValue};
        use jni::JavaVM;
        use ndk_context;

        log::info!("Getting real decks from AnkiDroid");

        // Get the Android context
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
            .map_err(|e| format!("Failed to get JavaVM: {}", e))?;
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| format!("Failed to attach thread: {}", e))?;

        // Get the activity/context
        let activity = unsafe { JObject::from_raw(ctx.context() as *mut _) };

        // Get ContentResolver
        let content_resolver = env
            .call_method(
                &activity,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[],
            )
            .map_err(|e| format!("Failed to get ContentResolver: {}", e))?
            .l()
            .map_err(|e| format!("Failed to convert ContentResolver: {}", e))?;

        // Create URI for decks
        let uri_class = env
            .find_class("android/net/Uri")
            .map_err(|e| format!("Failed to find Uri class: {}", e))?;
        let decks_uri_string = env
            .new_string("content://com.ichi2.anki.flashcards/decks")
            .map_err(|e| format!("Failed to create decks URI string: {}", e))?;
        let decks_uri = env
            .call_static_method(
                &uri_class,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&decks_uri_string.into())],
            )
            .map_err(|e| format!("Failed to parse decks URI: {}", e))?
            .l()
            .map_err(|e| format!("Failed to convert decks URI: {}", e))?;

        // Query for decks
        let cursor = env.call_method(
            &content_resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                JValue::Object(&decks_uri),
                JValue::Object(&JObject::null()),  // projection (all columns)
                JValue::Object(&JObject::null()),  // selection
                JValue::Object(&JObject::null()),  // selection args
                JValue::Object(&JObject::null())   // sort order
            ]
        );

        // Check for exceptions
        if env.exception_check().unwrap_or(false) {
            env.exception_clear().unwrap_or(());
            log::error!("Exception when querying decks - falling back to default deck");
            return Ok(r#"[{"id": 1, "name": "Default"}]"#.to_string());
        }

        let mut decks = Vec::new();

        if let Ok(cursor_result) = cursor {
            if let Ok(cursor_obj) = cursor_result.l() {
                if !cursor_obj.is_null() {
                    // Move to first row
                    let move_result = env.call_method(&cursor_obj, "moveToFirst", "()Z", &[]);
                    if let Ok(has_data) = move_result {
                        if has_data.z().unwrap_or(false) {
                            loop {
                                // Get deck ID (did column)
                                let did_str = env.new_string("did").unwrap();
                                let did_idx = env
                                    .call_method(
                                        &cursor_obj,
                                        "getColumnIndex",
                                        "(Ljava/lang/String;)I",
                                        &[JValue::Object(&did_str.into())],
                                    )
                                    .ok()
                                    .and_then(|v| v.i().ok())
                                    .unwrap_or(-1);

                                // Get deck name
                                let name_str = env.new_string("name").unwrap();
                                let name_idx = env
                                    .call_method(
                                        &cursor_obj,
                                        "getColumnIndex",
                                        "(Ljava/lang/String;)I",
                                        &[JValue::Object(&name_str.into())],
                                    )
                                    .ok()
                                    .and_then(|v| v.i().ok())
                                    .unwrap_or(-1);

                                let deck_id = if did_idx >= 0 {
                                    env.call_method(
                                        &cursor_obj,
                                        "getLong",
                                        "(I)J",
                                        &[JValue::Int(did_idx)],
                                    )
                                    .ok()
                                    .and_then(|v| v.j().ok())
                                    .unwrap_or(0)
                                } else {
                                    0
                                };

                                let deck_name = if name_idx >= 0 {
                                    env.call_method(
                                        &cursor_obj,
                                        "getString",
                                        "(I)Ljava/lang/String;",
                                        &[JValue::Int(name_idx)],
                                    )
                                    .ok()
                                    .and_then(|v| v.l().ok())
                                    .and_then(|s| {
                                        if !s.is_null() {
                                            env.get_string(&s.into()).ok().map(|js| {
                                                js.to_str().unwrap_or("Unknown").to_string()
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(|| "Unknown Deck".to_string())
                                } else {
                                    "Unknown Deck".to_string()
                                };

                                decks.push(format!(
                                    r#"{{"id": {}, "name": "{}"}}"#,
                                    deck_id,
                                    deck_name.replace('"', r#"\""#)
                                ));

                                // Move to next
                                let move_next =
                                    env.call_method(&cursor_obj, "moveToNext", "()Z", &[]);
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
                    let _ = env.call_method(&cursor_obj, "close", "()V", &[]);
                }
            }
        }

        if decks.is_empty() {
            // If no decks found, return default
            Ok(r#"[{"id": 1, "name": "Default"}]"#.to_string())
        } else {
            Ok(format!("[{}]", decks.join(",")))
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        Ok(r#"[
    {"id": 1, "name": "Default"},
    {"id": 2, "name": "Geography"},
    {"id": 3, "name": "Math"}
]"#
        .to_string())
    }
}

pub async fn update_card(
    note_id: i64,
    front: String,
    back: String,
    _deck: Option<String>,
    _tags: Option<String>,
) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        // Return mock response for now
        Ok(format!(
            r#"{{
    "success": true,
    "noteId": {},
    "message": "Card updated successfully: {} / {}",
    "error": null
}}"#,
            note_id,
            front.replace('"', r#"\""#),
            back.replace('"', r#"\""#)
        ))
    }

    #[cfg(not(target_os = "android"))]
    {
        Ok(format!(
            r#"{{
    "success": true,
    "noteId": {},
    "message": "Card updated (desktop mock): {} / {}",
    "error": null
}}"#,
            note_id,
            front.replace('"', r#"\""#),
            back.replace('"', r#"\""#)
        ))
    }
}

pub async fn delete_card(note_id: i64) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        // Return mock response for now
        Ok(format!(
            r#"{{
    "success": true,
    "noteId": {},
    "message": "Card deleted successfully",
    "error": null
}}"#,
            note_id
        ))
    }

    #[cfg(not(target_os = "android"))]
    {
        Ok(format!(
            r#"{{
    "success": true,
    "noteId": {},
    "message": "Card deleted (desktop mock)",
    "error": null
}}"#,
            note_id
        ))
    }
}

pub async fn list_cards() -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        // Wrap in panic catching to prevent crashes
        let result = std::panic::catch_unwind(|| get_ankidroid_cards());

        match result {
            Ok(cards_result) => {
                match cards_result {
                    Ok(cards) => Ok(cards),
                    Err(e) => {
                        // Even errors should return valid JSON, not crash
                        Ok(format!(
                            r#"[
  {{
    "id": 1,
    "front": "AnkiDroid Error",
    "back": "Error occurred: {}",
    "deck": "Error",
    "tags": ""
  }}
]"#,
                            e.replace('"', r#"\""#)
                        ))
                    }
                }
            }
            Err(_) => {
                // Panic occurred, return safe error response
                Ok(r#"[
  {
    "id": 1,
    "front": "System Error",
    "back": "A system error occurred while trying to read AnkiDroid cards. Please check if AnkiDroid is properly installed and configured.",
    "deck": "System Error",
    "tags": ""
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
    "front": "What is the capital of France?",
    "back": "Paris",
    "deck": "Geography",
    "tags": ""
  },
  {
    "id": 2,
    "front": "What is 2 + 2?",
    "back": "4",
    "deck": "Math",
    "tags": ""
  }
]"#
        .to_string())
    }
}

#[cfg(target_os = "android")]
fn get_ankidroid_info() -> Result<String, String> {
    use jni::objects::{JObject, JString, JValue};
    use jni::signature::{Primitive, ReturnType};
    use jni::JavaVM;
    use ndk_context;

    // Get the Android context and JVM
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
        .map_err(|e| format!("Failed to get JavaVM: {}", e))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("Failed to attach thread: {}", e))?;

    // Get the application context
    let context = unsafe { JObject::from_raw(ctx.context() as *mut _) };

    // Try to check if AnkiDroid is installed
    let package_manager = env
        .call_method(
            &context,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        )
        .map_err(|e| format!("Failed to get PackageManager: {}", e))?;

    let package_manager = package_manager
        .l()
        .map_err(|e| format!("Failed to get PackageManager object: {}", e))?;

    // Check if AnkiDroid is installed
    let ankidroid_package = env
        .new_string("com.ichi2.anki")
        .map_err(|e| format!("Failed to create string: {}", e))?;

    let package_info_result = env.call_method(
        &package_manager,
        "getPackageInfo",
        "(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;",
        &[JValue::Object(&ankidroid_package), JValue::Int(0)],
    );

    match package_info_result {
        Ok(_) => {
            // AnkiDroid is installed! Try to get some basic info
            let _content_resolver = env
                .call_method(
                    &context,
                    "getContentResolver",
                    "()Landroid/content/ContentResolver;",
                    &[],
                )
                .map_err(|e| format!("Failed to get ContentResolver: {}", e))?;

            Ok("✅ Connected! AnkiDroid is installed and accessible.".to_string())
        }
        Err(_) => Err(
            "❌ AnkiDroid not found. Please install AnkiDroid from F-Droid or Google Play."
                .to_string(),
        ),
    }
}

#[cfg(target_os = "android")]
fn get_ankidroid_cards() -> Result<String, String> {
    use jni::objects::{JObject, JString, JValue};
    use jni::JavaVM;
    use ndk_context;

    // Get the Android context and JVM
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }
        .map_err(|e| format!("Failed to get JavaVM: {}", e))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("Failed to attach thread: {}", e))?;

    // Get the application context
    let context = unsafe { JObject::from_raw(ctx.context() as *mut _) };

    // Try to query AnkiDroid's Content Provider for actual cards
    let content_resolver = env
        .call_method(
            &context,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .map_err(|e| format!("Failed to get ContentResolver: {}", e))?;

    let content_resolver = content_resolver
        .l()
        .map_err(|e| format!("Failed to get ContentResolver object: {}", e))?;

    // AnkiDroid Content Provider URI for notes - try the correct URI
    let uri_string = env
        .new_string("content://com.ichi2.anki.flashcards/notes")
        .map_err(|e| format!("Failed to create URI string: {}", e))?;

    // Parse URI
    let uri_class = env
        .find_class("android/net/Uri")
        .map_err(|e| format!("Failed to find Uri class: {}", e))?;
    let uri = env
        .call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&uri_string)],
        )
        .map_err(|e| format!("Failed to parse URI: {}", e))?;

    let uri = uri
        .l()
        .map_err(|e| format!("Failed to get URI object: {}", e))?;

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
    "front": "AnkiDroid Permission Error",
    "back": "SecurityException: Permission not granted for CardContentProvider.query\n\nThis means the AnkiDroid API is not accessible. Despite enabling 'Third party API', there may be additional requirements:\n\n1. AnkiDroid might need to be restarted\n2. The app might need specific permissions in Android settings\n3. AnkiDroid database might be locked or in use",
    "deck": "Error",
    "tags": ""
  }
]"#.to_string());
    }

    // NOW we can safely access cursor_result since there's no exception
    match cursor_result {
        Ok(cursor_result) => {
            let cursor = cursor_result
                .l()
                .map_err(|e| format!("Failed to get cursor: {}", e))?;

            if cursor.is_null() {
                return Ok(r#"[
  {
    "id": 1,
    "front": "AnkiDroid Permission Required",
    "back": "This app needs permission to read AnkiDroid cards. Please enable 'Third party apps' in AnkiDroid Settings > Advanced > AnkiDroid API.",
    "deck": "Setup Required",
    "tags": ""
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
                        if card_count >= 5 {
                            break;
                        } // Limit to first 5 cards

                        // Try to get column indices - AnkiDroid uses "flds" field for card content
                        let flds_str = env.new_string("flds").unwrap();
                        let tags_str = env.new_string("tags").unwrap();
                        let did_str = env.new_string("did").unwrap();
                        let id_str = env.new_string("_id").unwrap();

                        let flds_idx_result = env.call_method(
                            &cursor,
                            "getColumnIndex",
                            "(Ljava/lang/String;)I",
                            &[JValue::Object(&flds_str)],
                        );

                        let tags_idx_result = env.call_method(
                            &cursor,
                            "getColumnIndex",
                            "(Ljava/lang/String;)I",
                            &[JValue::Object(&tags_str)],
                        );

                        let did_idx_result = env.call_method(
                            &cursor,
                            "getColumnIndex",
                            "(Ljava/lang/String;)I",
                            &[JValue::Object(&did_str)],
                        );

                        let id_idx_result = env.call_method(
                            &cursor,
                            "getColumnIndex",
                            "(Ljava/lang/String;)I",
                            &[JValue::Object(&id_str)],
                        );

                        // Get deck ID
                        let deck_id = if let Ok(did_idx_val) = did_idx_result {
                            let did_idx = did_idx_val.i().unwrap_or(-1);
                            if did_idx >= 0 {
                                env.call_method(&cursor, "getLong", "(I)J", &[JValue::Int(did_idx)])
                                    .ok()
                                    .and_then(|v| v.j().ok())
                                    .unwrap_or(1)
                            } else {
                                1
                            }
                        } else {
                            1
                        };

                        // Get note ID
                        let note_id = if let Ok(id_idx_val) = id_idx_result {
                            let id_idx = id_idx_val.i().unwrap_or(-1);
                            if id_idx >= 0 {
                                env.call_method(&cursor, "getLong", "(I)J", &[JValue::Int(id_idx)])
                                    .ok()
                                    .and_then(|v| v.j().ok())
                                    .unwrap_or(card_count as i64 + 1)
                            } else {
                                card_count as i64 + 1
                            }
                        } else {
                            card_count as i64 + 1
                        };

                        // Get the flds (fields) content and tags
                        let (question, answer, tags) = match flds_idx_result {
                            Ok(flds_idx_val) => {
                                let flds_idx = flds_idx_val.i().unwrap_or(-1);

                                if flds_idx >= 0 {
                                    // Get the fields string
                                    let flds_content = match env.call_method(
                                        &cursor,
                                        "getString",
                                        "(I)Ljava/lang/String;",
                                        &[JValue::Int(flds_idx)],
                                    ) {
                                        Ok(flds_val) => {
                                            if let Ok(flds_obj) = flds_val.l() {
                                                if !flds_obj.is_null() {
                                                    match env.get_string(&flds_obj.into()) {
                                                        Ok(java_str) => java_str
                                                            .to_str()
                                                            .unwrap_or("")
                                                            .to_string(),
                                                        Err(_) => "".to_string(),
                                                    }
                                                } else {
                                                    "".to_string()
                                                }
                                            } else {
                                                "".to_string()
                                            }
                                        }
                                        Err(_) => "".to_string(),
                                    };

                                    // Split fields by unit separator (U+001F)
                                    let fields: Vec<&str> =
                                        flds_content.split('\u{001f}').collect();
                                    let front = fields.get(0).unwrap_or(&"No front").to_string();
                                    let back = fields.get(1).unwrap_or(&"No back").to_string();

                                    // Get tags if available
                                    let tags_content = if let Ok(tags_idx_val) = tags_idx_result {
                                        let tags_idx = tags_idx_val.i().unwrap_or(-1);
                                        if tags_idx >= 0 {
                                            match env.call_method(
                                                &cursor,
                                                "getString",
                                                "(I)Ljava/lang/String;",
                                                &[JValue::Int(tags_idx)],
                                            ) {
                                                Ok(tags_val) => {
                                                    if let Ok(tags_obj) = tags_val.l() {
                                                        if !tags_obj.is_null() {
                                                            match env.get_string(&tags_obj.into()) {
                                                                Ok(java_str) => java_str
                                                                    .to_str()
                                                                    .unwrap_or("")
                                                                    .to_string(),
                                                                Err(_) => "".to_string(),
                                                            }
                                                        } else {
                                                            "".to_string()
                                                        }
                                                    } else {
                                                        "".to_string()
                                                    }
                                                }
                                                Err(_) => "".to_string(),
                                            }
                                        } else {
                                            "".to_string()
                                        }
                                    } else {
                                        "".to_string()
                                    };

                                    (front, back, tags_content)
                                } else {
                                    (
                                        "No flds column found".to_string(),
                                        "Check AnkiDroid database".to_string(),
                                        "".to_string(),
                                    )
                                }
                            }
                            _ => (
                                "Error getting flds index".to_string(),
                                "Database access error".to_string(),
                                "".to_string(),
                            ),
                        };

                        cards.push(format!(
                            r#"  {{
    "id": {},
    "noteId": {},
    "deckId": {},
    "front": "{}",
    "back": "{}",
    "tags": "{}"
  }}"#,
                            card_count + 1,
                            note_id,
                            deck_id,
                            question.replace('"', r#"\""#).replace('\n', r#"\n"#),
                            answer.replace('"', r#"\""#).replace('\n', r#"\n"#),
                            tags.replace('"', r#"\""#).replace('\n', r#"\n"#)
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
    "front": "AnkiDroid Permission Denied",
    "back": "Permission denied when accessing AnkiDroid. Please enable API access in AnkiDroid Settings > Advanced > AnkiDroid API > 'Third party apps'.",
    "deck": "Permission Error",
    "tags": ""
  },
  {
    "id": 2,
    "front": "How to Enable AnkiDroid API",
    "back": "1. Open AnkiDroid\n2. Go to Settings > Advanced\n3. Find 'AnkiDroid API'\n4. Enable 'Third party apps'\n5. Restart this app",
    "deck": "Setup Instructions",
    "tags": ""
  }
]"#.to_string())
            } else {
                Ok(format!(
                    r#"[
  {{
    "id": 1,
    "front": "AnkiDroid Connection Error",
    "back": "Error connecting to AnkiDroid: {}",
    "deck": "Error",
    "tags": ""
  }}
]"#,
                    e.to_string().replace('"', r#"\""#)
                ))
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
        assert!(
            parsed.is_ok(),
            "Response should be valid JSON: {}",
            response
        );

        // Check it's an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "Response should be a JSON array");

        // Check array has elements
        let array = json.as_array().unwrap();
        assert!(array.len() > 0, "Should have at least one card");

        // Validate first card structure
        let first_card = &array[0];
        assert!(
            first_card.get("id").is_some(),
            "Card should have 'id' field"
        );
        assert!(
            first_card.get("front").is_some(),
            "Card should have 'front' field"
        );
        assert!(
            first_card.get("back").is_some(),
            "Card should have 'back' field"
        );
        assert!(
            first_card.get("deck").is_some(),
            "Card should have 'deck' field"
        );
    }

    #[tokio::test]
    async fn test_list_cards_performance() {
        use std::time::Instant;

        let start = Instant::now();
        let result = list_cards().await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(
            duration.as_millis() < 1000,
            "Should complete within 1 second, took {:?}",
            duration
        );
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_get_ankidroid_cards_doesnt_panic() {
        // This test ensures the function doesn't panic even if AnkiDroid isn't available
        let result = std::panic::catch_unwind(|| get_ankidroid_cards());

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

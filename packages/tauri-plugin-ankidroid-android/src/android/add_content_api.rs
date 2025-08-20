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
    /// Create a new instance of AddContentApi
    /// The context should be the Android Activity or Application context
    pub fn new(mut env: SafeJNIEnv<'local>, context: &JObject<'local>) -> AndroidResult<Self> {
        log::info!("Creating AddContentApi instance...");
        
        // Try to find the AddContentApi class
        let api_class = match env.find_class_checked("com/ichi2/anki/api/AddContentApi") {
            Ok(class) => class,
            Err(_) => {
                log::error!("AddContentApi class not found - AnkiDroid may not be installed");
                return Err(AndroidError::AnkiDroidNotAvailable(
                    "AddContentApi class not found. AnkiDroid may not be installed or the API is not available.".to_string()
                ));
            }
        };

        // Create an instance of AddContentApi with the context
        let api_instance = env.new_object_checked(
            "com/ichi2/anki/api/AddContentApi",
            "(Landroid/content/Context;)V",
            &[JValue::Object(context)],
        )?;

        log::info!("✅ AddContentApi instance created successfully");
        Ok(Self {
            api_instance,
            env,
        })
    }

    /// Add a note to AnkiDroid
    /// Returns the note ID on success, or None if the note couldn't be added
    pub fn add_note(
        &mut self,
        model_id: i64,
        deck_id: i64,
        fields: &[&str],
        tags: Option<&[&str]>,
    ) -> AndroidResult<Option<i64>> {
        log::info!("AddContentApi.addNote called with model_id={}, deck_id={}", model_id, deck_id);
        
        // Create the fields array
        let fields_array = self.env.env().new_object_array(
            fields.len() as i32,
            "java/lang/String",
            &JObject::null(),
        ).check_exception(self.env.env_mut())?;
        
        // Populate the fields array
        for (i, field) in fields.iter().enumerate() {
            let field_string = self.env.new_string_checked(field)?;
            let field_obj: JObject = field_string.into();
            self.env.env().set_object_array_element(&fields_array, i as i32, field_obj)
                .check_exception(self.env.env_mut())?;
        }

        // Create tags Set if provided
        let tags_set = if let Some(tags_array) = tags {
            // Create a HashSet for tags
            let set_class = self.env.find_class_checked("java/util/HashSet")?;
            let tags_set = self.env.env().new_object(set_class, "()V", &[])
                .check_exception(self.env.env_mut())?;
            
            // Add each tag to the set
            for tag in tags_array {
                let tag_string = self.env.new_string_checked(tag)?;
                self.env.env().call_method(
                    &tags_set,
                    "add",
                    "(Ljava/lang/Object;)Z",
                    &[JValue::Object(&tag_string.into())],
                ).check_exception(self.env.env_mut())?;
            }
            
            tags_set
        } else {
            JObject::null()
        };

        // Call addNote(modelId, deckId, fields, tags)
        log::info!("Calling AddContentApi.addNote...");
        let result = self.env.env().call_method(
            &self.api_instance,
            "addNote",
            "(JJ[Ljava/lang/String;Ljava/util/Set;)Ljava/lang/Long;",
            &[
                JValue::Long(model_id),
                JValue::Long(deck_id),
                JValue::Object(&fields_array),
                JValue::Object(&tags_set),
            ],
        ).check_exception(self.env.env_mut())?;

        // Extract the note ID from the Long object
        let note_id_obj = result.l().map_err(AndroidError::from)?;
        if note_id_obj.is_null() {
            log::warn!("AddContentApi.addNote returned null - note creation may have failed");
            return Ok(None);
        }

        // Get the long value from the Long object
        let note_id = self.env.env().call_method(
            &note_id_obj,
            "longValue",
            "()J",
            &[],
        ).check_exception(self.env.env_mut())?
            .j()
            .map_err(AndroidError::from)?;

        log::info!("✅ Note created successfully with ID: {}", note_id);
        Ok(Some(note_id))
    }

    /// Add a new deck to AnkiDroid
    /// Returns the deck ID
    pub fn add_new_deck(&mut self, deck_name: &str) -> AndroidResult<i64> {
        log::info!("AddContentApi.addNewDeck called with name: {}", deck_name);
        
        let deck_name_string = self.env.new_string_checked(deck_name)?;
        
        // Call addNewDeck(String deckName)
        let result = self.env.env().call_method(
            &self.api_instance,
            "addNewDeck",
            "(Ljava/lang/String;)Ljava/lang/Long;",
            &[JValue::Object(&deck_name_string.into())],
        ).check_exception(self.env.env_mut())?;

        // Extract the deck ID
        let deck_id_obj = result.l().map_err(AndroidError::from)?;
        if deck_id_obj.is_null() {
            return Err(AndroidError::database_error("Failed to create deck - addNewDeck returned null"));
        }

        let deck_id = self.env.env().call_method(
            &deck_id_obj,
            "longValue",
            "()J",
            &[],
        ).check_exception(self.env.env_mut())?
            .j()
            .map_err(AndroidError::from)?;

        log::info!("✅ Deck created successfully with ID: {}", deck_id);
        Ok(deck_id)
    }

    /// Add a new basic model (note type) to AnkiDroid
    /// Returns the model ID
    pub fn add_new_basic_model(&mut self, model_name: &str) -> AndroidResult<i64> {
        log::info!("AddContentApi.addNewBasicModel called with name: {}", model_name);
        
        let model_name_string = self.env.new_string_checked(model_name)?;
        
        // Call addNewBasicModel(String modelName)
        let result = self.env.env().call_method(
            &self.api_instance,
            "addNewBasicModel",
            "(Ljava/lang/String;)Ljava/lang/Long;",
            &[JValue::Object(&model_name_string.into())],
        ).check_exception(self.env.env_mut())?;

        // Extract the model ID
        let model_id_obj = result.l().map_err(AndroidError::from)?;
        if model_id_obj.is_null() {
            return Err(AndroidError::database_error("Failed to create model - addNewBasicModel returned null"));
        }

        let model_id = self.env.env().call_method(
            &model_id_obj,
            "longValue",
            "()J",
            &[],
        ).check_exception(self.env.env_mut())?
            .j()
            .map_err(AndroidError::from)?;

        log::info!("✅ Model created successfully with ID: {}", model_id);
        Ok(model_id)
    }

    /// Check if AnkiDroid is available
    pub fn is_available(env: &mut SafeJNIEnv, context: &JObject) -> bool {
        // Try to call the static method getAnkiDroidPackageName
        let api_class = match env.find_class_checked("com/ichi2/anki/api/AddContentApi") {
            Ok(class) => class,
            Err(_) => {
                log::debug!("AddContentApi class not found");
                return false;
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
            );

        match package_name {
            Ok(result) => {
                match result.l() {
                    Ok(obj) => !obj.is_null(),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }
}
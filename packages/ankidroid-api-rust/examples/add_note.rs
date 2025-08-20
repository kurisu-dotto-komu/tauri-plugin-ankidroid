//! Complete example showing how to add notes to AnkiDroid
//! 
//! This example demonstrates:
//! - Checking AnkiDroid availability
//! - Creating a custom model if needed
//! - Adding a single note with fields and tags
//! - Error handling best practices
//! 
//! Note: This example only works on Android targets where AnkiDroid is installed

#[cfg(target_os = "android")]
use ankidroid_api_rust::{AnkiDroidApi, AnkiDroidError, Result, DEFAULT_DECK_ID};

#[cfg(target_os = "android")]
use android_logger;

#[cfg(target_os = "android")]
use log::{info, warn, error, debug};

/// Complete workflow for adding a note to AnkiDroid
#[cfg(target_os = "android")]
fn main() -> Result<()> {
    // Initialize Android logging
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
    );
    
    info!("Starting AnkiDroid note addition example");
    
    // Get the JNI JavaVM - in a real Android app, this would be provided
    // by the Android runtime or through JNI native method parameters
    let vm = get_java_vm()?;
    
    // Initialize the AnkiDroid API
    let api = AnkiDroidApi::new(vm);
    
    // Step 1: Check if AnkiDroid is available
    info!("Checking AnkiDroid availability...");
    if !api.is_available()? {
        error!("AnkiDroid is not available on this device");
        return Err(AnkiDroidError::not_available(
            "AnkiDroid is not installed or the API is not accessible. Please install AnkiDroid from the Play Store."
        ));
    }
    info!("AnkiDroid is available!");
    
    // Step 2: Get or create a suitable model
    let model_id = ensure_model_exists(&api)?;
    info!("Using model ID: {}", model_id);
    
    // Step 3: Create and add a single note
    add_sample_note(&api, model_id)?;
    
    // Step 4: Demonstrate bulk note addition
    bulk_add_sample_notes(&api, model_id)?;
    
    // Step 5: Demonstrate note querying and updating
    demonstrate_note_operations(&api)?;
    
    info!("Example completed successfully!");
    Ok(())
}

/// Ensure a suitable model exists, creating one if necessary
#[cfg(target_os = "android")]
fn ensure_model_exists(api: &AnkiDroidApi) -> Result<i64> {
    // First, try to get the current model
    match api.current_model_id() {
        Ok(model_id) => {
            info!("Using existing current model: {}", model_id);
            
            // Verify the model has at least 2 fields (front/back)
            let fields = api.get_field_list(model_id)?;
            if fields.len() >= 2 {
                info!("Model has {} fields: {:?}", fields.len(), fields);
                return Ok(model_id);
            } else {
                warn!("Current model only has {} fields, creating a new one", fields.len());
            }
        }
        Err(e) => {
            warn!("Could not get current model: {}", e);
        }
    }
    
    // If we don't have a suitable model, create one
    info!("Creating a new basic model...");
    let model_id = api.add_new_basic_model("Rust API Example Cards")?
        .ok_or_else(|| AnkiDroidError::operation_failed("Failed to create basic model"))?;
    
    info!("Created new model with ID: {}", model_id);
    Ok(model_id)
}

/// Add a single sample note
#[cfg(target_os = "android")]
fn add_sample_note(api: &AnkiDroidApi, model_id: i64) -> Result<()> {
    info!("Creating a sample note...");
    
    // Create a note using the builder pattern
    let note = Note::builder(model_id)
        .field("What is the capital of France?".to_string())
        .field("Paris".to_string())
        .tag("geography".to_string())
        .tag("capitals".to_string())
        .tag("europe".to_string())
        .build();
    
    info!("Adding note with {} fields and {} tags", note.fields.len(), note.tags.len());
    debug!("Note fields: {:?}", note.fields);
    debug!("Note tags: {:?}", note.tags);
    
    // Add the note to the default deck
    let note_id = api.add_note(&note)?;
    
    info!("Successfully added note with ID: {}", note_id);
    Ok(())
}

/// Demonstrate bulk note addition
#[cfg(target_os = "android")]
fn bulk_add_sample_notes(api: &AnkiDroidApi, model_id: i64) -> Result<()> {
    info!("Creating multiple notes for bulk addition...");
    
    let notes = vec![
        Note::builder(model_id)
            .field("What is the capital of Germany?".to_string())
            .field("Berlin".to_string())
            .tag("geography".to_string())
            .tag("capitals".to_string())
            .tag("europe".to_string())
            .build(),
            
        Note::builder(model_id)
            .field("What is the capital of Japan?".to_string())
            .field("Tokyo".to_string())
            .tag("geography".to_string())
            .tag("capitals".to_string())
            .tag("asia".to_string())
            .build(),
            
        Note::builder(model_id)
            .field("What is the capital of Australia?".to_string())
            .field("Canberra".to_string())
            .tag("geography".to_string())
            .tag("capitals".to_string())
            .tag("oceania".to_string())
            .build(),
    ];
    
    info!("Adding {} notes in bulk...", notes.len());
    let added_count = api.add_notes(DEFAULT_DECK_ID, &notes)?;
    
    info!("Successfully added {} out of {} notes", added_count, notes.len());
    
    if added_count != notes.len() {
        warn!("Not all notes were added successfully. Check for duplicates or validation errors.");
    }
    
    Ok(())
}

/// Demonstrate note querying and updating operations
#[cfg(target_os = "android")]
fn demonstrate_note_operations(api: &AnkiDroidApi) -> Result<()> {
    info!("Demonstrating note query and update operations...");
    
    // For this example, we'll assume we have note IDs from previous operations
    // In a real application, you would get these from the add_note results
    // or by querying the database
    
    // This is a simplified example - in practice you'd query for existing notes
    info!("Note operations demonstration completed (simplified)");
    
    Ok(())
}

/// Get the JNI JavaVM instance
/// 
/// In a real Android application, this would typically be:
/// 1. Passed as a parameter to a JNI native method
/// 2. Obtained through the android runtime
/// 3. Stored globally during JNI_OnLoad
#[cfg(target_os = "android")]
fn get_java_vm() -> Result<jni::JavaVM> {
    // This is a placeholder - in a real app, you would get this from:
    // - JNI native method parameters
    // - Global storage set during JNI_OnLoad
    // - Android NDK context
    
    // For the example, we'll return an error with instructions
    Err(AnkiDroidError::system_error(
        "JavaVM not available in this context. In a real Android app, obtain the JavaVM from JNI method parameters or global storage."
    ))
}

/// Error handling example showing different error types
#[cfg(target_os = "android")]
fn demonstrate_error_handling() {
    use ankidroid_api_rust::{AnkiDroidError, ErrorCategory};
    
    // Example of handling different error types
    let errors = vec![
        AnkiDroidError::not_available("AnkiDroid not installed"),
        AnkiDroidError::permission_denied("Missing READ_WRITE_PERMISSION"),
        AnkiDroidError::validation_error("Invalid field count"),
        AnkiDroidError::not_found("Model not found"),
        AnkiDroidError::database_error("SQL constraint violation"),
        AnkiDroidError::operation_failed("Network timeout"),
        AnkiDroidError::system_error("Out of memory"),
    ];
    
    for error in errors {
        println!("Error: {}", error);
        println!("Category: {:?}", error.category());
        println!("Recoverable: {}", error.is_recoverable());
        println!("---");
    }
}

/// Non-Android stub for compilation
#[cfg(not(target_os = "android"))]
fn main() {
    println!("This example is designed for Android targets only.");
    println!("To run this example:");
    println!("1. Set up Android NDK and build tools");
    println!("2. Build for an Android target:");
    println!("   cargo build --target aarch64-linux-android");
    println!("3. Deploy and run on an Android device with AnkiDroid installed");
}
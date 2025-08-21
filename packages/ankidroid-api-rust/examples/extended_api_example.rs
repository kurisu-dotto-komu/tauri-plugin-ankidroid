//! Extended AnkiDroid API Usage Example
//!
//! This example demonstrates how to use the extended AnkiDroid API functionality
//! for comprehensive note management including listing, updating, and deleting notes.
//!
//! # Platform
//!
//! This example only works on Android targets with AnkiDroid installed.

#[cfg(target_os = "android")]
use ankidroid_api_rust::{AnkiDroidApi, AnkiDroidApiExtended, AnkiDroidError, Result};

#[cfg(target_os = "android")]
use jni::{objects::JObject, JNIEnv};

/// Example function demonstrating extended AnkiDroid API usage
///
/// This function shows how to:
/// 1. Create an API instance
/// 2. List all existing notes
/// 3. Update note fields
/// 4. Delete notes
///
/// # Arguments
///
/// * `env` - JNI environment (from Android activity)
/// * `context` - Android Context (Activity or Application context)
///
/// # Returns
///
/// Result indicating success or failure of the operations
#[cfg(target_os = "android")]
pub fn extended_api_example(env: JNIEnv, context: &JObject) -> Result<()> {
    println!("🚀 Starting Extended AnkiDroid API Example");

    // Create API instance (it will check availability internally)
    let mut api = AnkiDroidApi::try_new(env, context)?;
    let api_version = api.get_api_host_spec_version()?;
    println!("📱 Connected to AnkiDroid API version: {}", api_version);

    // === EXTENDED API: List Notes ===
    println!("\n📋 Listing all notes...");
    let notes = api.list_notes()?;
    println!("Found {} notes in the database", notes.len());

    // Display first few notes
    for (i, note) in notes.iter().take(5).enumerate() {
        println!(
            "  {}. Note ID: {}, Model: {}, Fields: {:?}, Tags: {:?}",
            i + 1,
            note.id,
            note.mid,
            note.fields.iter().take(2).collect::<Vec<_>>(), // Show first 2 fields
            note.tags
        );
    }

    // === EXTENDED API: Update Notes ===
    if let Some(first_note) = notes.first() {
        println!("\n✏️  Updating note {}...", first_note.id);
        
        // Prepare updated fields (same count as original)
        let mut updated_fields = first_note.fields.clone();
        if !updated_fields.is_empty() {
            // Update the first field to include a timestamp
            updated_fields[0] = format!("{} (Updated at {})", 
                updated_fields[0], 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
        }

        let field_refs: Vec<&str> = updated_fields.iter().map(|s| s.as_str()).collect();
        api.update_note(first_note.id, &field_refs)?;
        println!("✅ Note {} updated successfully", first_note.id);

        // Verify the update by listing notes again
        let updated_notes = api.list_notes()?;
        if let Some(updated_note) = updated_notes.iter().find(|n| n.id == first_note.id) {
            println!("📝 Updated note fields: {:?}", updated_note.fields);
        }
    }

    // === EXTENDED API: Delete Notes (Demo - commented out for safety) ===
    println!("\n🗑️  Note deletion example (commented out for safety)");
    println!("   To delete a note, use: api.delete_note(note_id)?");
    
    // Uncomment the following lines to actually delete notes:
    /*
    if notes.len() > 10 { // Only delete if there are many notes
        if let Some(last_note) = notes.last() {
            println!("Deleting note {}...", last_note.id);
            let deleted = api.delete_note(last_note.id)?;
            if deleted {
                println!("✅ Note {} deleted successfully", last_note.id);
            } else {
                println!("❌ Note {} could not be deleted", last_note.id);
            }
        }
    }
    */

    // === Final Statistics ===
    let final_notes = api.list_notes()?;
    println!("\n📊 Final Statistics:");
    println!("   Total notes: {}", final_notes.len());
    
    // Group notes by model
    let mut model_counts = std::collections::HashMap::new();
    for note in &final_notes {
        *model_counts.entry(note.mid).or_insert(0) += 1;
    }
    
    println!("   Notes by model:");
    for (model_id, count) in model_counts {
        println!("     Model {}: {} notes", model_id, count);
    }

    println!("\n🎉 Extended API example completed successfully!");
    Ok(())
}

/// Example showing error handling with extended API
#[cfg(target_os = "android")]
pub fn error_handling_example(env: JNIEnv, context: &JObject) -> Result<()> {
    println!("🛡️  Extended API Error Handling Example");

    let mut api = AnkiDroidApi::try_new(env, context)?;

    // Try to update a non-existent note
    println!("Testing update with invalid note ID...");
    match api.update_note(9999999, &["Field 1", "Field 2"]) {
        Ok(_) => println!("❓ Unexpected success"),
        Err(e) => {
            println!("✅ Expected error: {}", e);
            println!("   Error category: {}", e.category());
            println!("   Is recoverable: {}", e.is_recoverable());
        }
    }

    // Try to delete a non-existent note
    println!("Testing delete with invalid note ID...");
    match api.delete_note(9999999) {
        Ok(deleted) => {
            if deleted {
                println!("❓ Unexpected deletion success");
            } else {
                println!("✅ Note not found (expected behavior)");
            }
        }
        Err(e) => {
            println!("⚠️  Unexpected error: {}", e);
        }
    }

    println!("🛡️  Error handling example completed!");
    Ok(())
}

/// Non-Android platform message
#[cfg(not(target_os = "android"))]
pub fn extended_api_example() {
    println!("⚠️  This example only works on Android with AnkiDroid installed.");
    println!("   Please compile and run this code on an Android device or emulator.");
}

/// Non-Android platform message for error handling
#[cfg(not(target_os = "android"))]
pub fn error_handling_example() {
    println!("⚠️  This example only works on Android with AnkiDroid installed.");
    println!("   Please compile and run this code on an Android device or emulator.");
}

/// Main function for the example
///
/// This example is designed to be run on Android, so the main function
/// just provides instructions for non-Android platforms.
fn main() {
    #[cfg(target_os = "android")]
    {
        println!("🤖 This example should be integrated into an Android application.");
        println!("   Use the extended_api_example() function from your Android code.");
    }
    
    #[cfg(not(target_os = "android"))]
    {
        extended_api_example();
        error_handling_example();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_compiles() {
        // This test just ensures the example code compiles correctly
        // Actual functionality testing requires an Android environment
        assert!(true);
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_android_functions_exist() {
        // Verify that the Android-specific functions are available
        // This would require actual JNI setup to test functionality
        use ankidroid_api_rust::AnkiDroidApiExtended;
        // The trait should be available for import
        assert!(true);
    }
}
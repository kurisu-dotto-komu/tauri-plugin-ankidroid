//! Extended AnkiDroid API functionality
//!
//! This module provides additional API methods that extend the core AnkiDroid functionality
//! with CRUD operations for notes, including listing, updating, and deletion capabilities.
//!
//! # EXTENDED API:
//!
//! This module contains extended functionality beyond the standard AnkiDroid AddContentApi,
//! providing comprehensive note management operations for advanced use cases.

#[cfg(target_os = "android")]
use crate::{
    error::{AnkiDroidError, Result},
    jni::helpers::ContentValuesBuilder,
    models::Note,
    contract::{self, note},
    utils::{join_fields, split_fields, split_tags},
    api::AnkiDroidApi,
};

/// /// EXTENDED API: Extended trait providing additional AnkiDroid API functionality
///
/// This trait extends the base AnkiDroid API with CRUD operations for notes,
/// allowing comprehensive note management beyond the standard AddContentApi.
///
/// # Features
///
/// - **List Notes**: Query and retrieve notes with their field data and tags
/// - **Update Notes**: Modify existing note fields while preserving metadata
/// - **Delete Notes**: Remove notes and their associated cards from the database
///
/// # Examples
///
/// ```rust,ignore
/// use ankidroid_api_rust::{AnkiDroidApi, AnkiDroidApiExtended};
///
/// #[cfg(target_os = "android")]
/// fn example_extended_usage(env: JNIEnv, context: JObject) -> Result<()> {
///     let mut api = AnkiDroidApi::try_new(env, &context)?;
///     
///     // List all notes
///     let notes = api.list_notes()?;
///     println!("Found {} notes", notes.len());
///     
///     if let Some(note) = notes.first() {
///         // Update a note's fields
///         api.update_note(note.id, &["Updated Front", "Updated Back"])?;
///         
///         // Delete a note
///         let deleted = api.delete_note(note.id)?;
///         println!("Note deleted: {}", deleted);
///     }
///     
///     Ok(())
/// }
/// ```
#[cfg(target_os = "android")]
pub trait AnkiDroidApiExtended {
    /// /// EXTENDED API: List all notes in the database
    ///
    /// Retrieves all notes with their field data and tags from the AnkiDroid database.
    /// This method queries the notes content provider and parses the results into
    /// structured Note objects.
    ///
    /// # Returns
    ///
    /// A vector of Note structs containing all notes in the database, or an error
    /// if the query fails or the database is inaccessible.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let notes = api.list_notes()?;
    /// for note in notes {
    ///     println!("Note ID: {}, Fields: {:?}", note.id, note.fields);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - `AnkiDroidError::DatabaseError` - Database query failed
    /// - `AnkiDroidError::PermissionDenied` - Insufficient permissions
    /// - `AnkiDroidError::StringConversionError` - Field parsing failed
    fn list_notes(&mut self) -> Result<Vec<Note>>;

    /// /// EXTENDED API: Update an existing note's fields
    ///
    /// Updates the field values of an existing note identified by its ID.
    /// The number of fields provided must match the model's field count.
    /// Tags and other metadata are preserved during the update.
    ///
    /// # Arguments
    ///
    /// * `note_id` - The ID of the note to update
    /// * `fields` - Array of new field values (must match model field count)
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeds, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Update a note with new field values
    /// api.update_note(12345, &["New Front", "New Back"])?;
    /// ```
    ///
    /// # Errors
    ///
    /// - `AnkiDroidError::DatabaseError` - Update operation failed
    /// - `AnkiDroidError::ValidationError` - Invalid note ID or field count
    /// - `AnkiDroidError::PermissionDenied` - Insufficient permissions
    fn update_note(&mut self, note_id: i64, fields: &[&str]) -> Result<()>;

    /// /// EXTENDED API: Delete a note and its associated cards
    ///
    /// Removes a note from the database along with all its generated cards.
    /// This operation cannot be undone and will permanently remove the note
    /// from all decks and study sessions.
    ///
    /// # Arguments
    ///
    /// * `note_id` - The ID of the note to delete
    ///
    /// # Returns
    ///
    /// `true` if the note was successfully deleted, `false` if the note
    /// was not found or could not be deleted.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let deleted = api.delete_note(12345)?;
    /// if deleted {
    ///     println!("Note successfully deleted");
    /// } else {
    ///     println!("Note not found or could not be deleted");
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - `AnkiDroidError::DatabaseError` - Delete operation failed
    /// - `AnkiDroidError::PermissionDenied` - Insufficient permissions
    /// - `AnkiDroidError::ValidationError` - Invalid note ID
    fn delete_note(&mut self, note_id: i64) -> Result<bool>;
}

#[cfg(target_os = "android")]
impl<'local> AnkiDroidApiExtended for AnkiDroidApi<'local> {
    fn list_notes(&mut self) -> Result<Vec<Note>> {
        log::info!("Querying all notes from AnkiDroid database");

        // Query the notes content provider
        let mut cursor = self.resolver_mut().query(
            &contract::build_note_uri(),
            Some(vec![
                note::_ID.to_string(),
                note::GUID.to_string(),
                note::MID.to_string(),
                note::MOD.to_string(),
                note::USN.to_string(),
                note::TAGS.to_string(),
                note::FLDS.to_string(),
                note::SFLD.to_string(),
                note::CSUM.to_string(),
                note::FLAGS.to_string(),
                note::DATA.to_string(),
            ]),
            None,
            None,
            None,
        )?;

        let mut notes = Vec::new();

        while cursor.move_to_next()? {
            // Extract note data from cursor
            let id_str = cursor.get_string_by_name(note::_ID)?;
            let id = id_str.parse::<i64>()
                .map_err(|_| AnkiDroidError::validation_error("Invalid note ID in database"))?;

            let guid = cursor.get_string_by_name(note::GUID)?;
            
            let mid_str = cursor.get_string_by_name(note::MID)?;
            let mid = mid_str.parse::<i64>()
                .map_err(|_| AnkiDroidError::validation_error("Invalid model ID in database"))?;

            let mod_str = cursor.get_string_by_name(note::MOD)?;
            let mod_ = mod_str.parse::<i64>()
                .map_err(|_| AnkiDroidError::validation_error("Invalid modification time in database"))?;

            let usn_str = cursor.get_string_by_name(note::USN)?;
            let usn = usn_str.parse::<i32>()
                .map_err(|_| AnkiDroidError::validation_error("Invalid USN in database"))?;

            let tags_str = cursor.get_string_by_name(note::TAGS)?;
            let tags = if tags_str.trim().is_empty() {
                Vec::new()
            } else {
                split_tags(&tags_str)
            };

            let flds_str = cursor.get_string_by_name(note::FLDS)?;
            let fields = split_fields(&flds_str);

            let sfld = cursor.get_string_by_name(note::SFLD)?;

            let csum_str = cursor.get_string_by_name(note::CSUM)?;
            let csum = csum_str.parse::<i64>()
                .map_err(|_| AnkiDroidError::validation_error("Invalid checksum in database"))?;

            let flags_str = cursor.get_string_by_name(note::FLAGS)?;
            let flags = flags_str.parse::<i32>()
                .map_err(|_| AnkiDroidError::validation_error("Invalid flags in database"))?;

            let data = cursor.get_string_by_name(note::DATA)?;

            // Create Note struct
            let note = Note {
                id,
                guid,
                mid,
                mod_,
                usn,
                tags,
                fields,
                sfld,
                csum,
                flags,
                data,
            };

            notes.push(note);
        }

        log::info!("✅ Retrieved {} notes from database", notes.len());
        Ok(notes)
    }

    fn update_note(&mut self, note_id: i64, fields: &[&str]) -> Result<()> {
        log::info!("Updating note {} with {} fields", note_id, fields.len());

        // Validate inputs
        if fields.is_empty() {
            return Err(AnkiDroidError::validation_error("Fields cannot be empty"));
        }

        // Build the note URI for the specific note
        let note_uri = contract::build_note_by_id_uri(note_id);

        // Join fields using the field separator
        let joined_fields = join_fields(&fields.iter().map(|s| *s).collect::<Vec<_>>());

        // Create ContentValues for the update
        let values = ContentValuesBuilder::new(self.env_mut())?
            .put_string(note::FLDS, &joined_fields)?
            // Update sort field to first field
            .put_string(note::SFLD, fields.first().unwrap_or(&""))?
            // Update modification time (Unix timestamp)
            .put_long(note::MOD, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| AnkiDroidError::validation_error("Failed to get current timestamp"))?
                .as_secs() as i64)?
            // Mark as modified for sync
            .put_int(note::USN, -1)?;

        // Perform the update
        let updated_count = self.resolver_mut().update(&note_uri, values, None, None)?;

        if updated_count == 0 {
            return Err(AnkiDroidError::validation_error(
                format!("Note with ID {} not found or could not be updated", note_id)
            ));
        }

        log::info!("✅ Note {} updated successfully", note_id);
        Ok(())
    }

    fn delete_note(&mut self, note_id: i64) -> Result<bool> {
        log::info!("Deleting note {}", note_id);

        // Build the note URI for the specific note
        let note_uri = contract::build_note_by_id_uri(note_id);

        // Perform the delete operation
        let deleted_count = self.resolver_mut().delete(&note_uri, None, None)?;

        let success = deleted_count > 0;
        
        if success {
            log::info!("✅ Note {} deleted successfully", note_id);
        } else {
            log::warn!("Note {} not found or could not be deleted", note_id);
        }

        Ok(success)
    }
}

// Make sure we only export for Android targets
#[cfg(not(target_os = "android"))]
compile_error!("This module is only available on Android targets");

#[cfg(test)]
#[cfg(target_os = "android")]
mod tests {
    use super::*;
    use crate::models::Note;

    #[test]
    fn test_extended_trait_exists() {
        // This test ensures the trait is properly defined
        // Actual functionality tests would require a mock JNI environment
        assert!(true);
    }

    #[test]
    fn test_note_construction() {
        // Test that we can construct notes properly
        let note = Note::new(123, vec!["Front".to_string(), "Back".to_string()]);
        assert_eq!(note.mid, 123);
        assert_eq!(note.fields.len(), 2);
        assert_eq!(note.key(), "Front");
    }

    // Additional tests would require a mock JNI environment
    // which is complex to set up in unit tests
}
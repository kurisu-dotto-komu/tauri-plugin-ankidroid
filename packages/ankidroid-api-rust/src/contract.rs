//! AnkiDroid FlashCards Contract Constants
//! 
//! This module contains all the constants and URI builders needed to interact
//! with the AnkiDroid content provider API. It's a Rust translation of the
//! original Kotlin FlashCardsContract.

/// Authority for the AnkiDroid content provider
pub const AUTHORITY: &str = "com.ichi2.anki.flashcards";

/// Permission required for read/write operations
pub const READ_WRITE_PERMISSION: &str = "com.ichi2.anki.permission.READ_WRITE_DATABASE";

/// Default deck ID
pub const DEFAULT_DECK_ID: i64 = 1;

/// Field separator character used in note fields
pub const FIELD_SEPARATOR: char = '\u{001f}';

/// URI builder functions
pub fn build_authority_uri() -> String {
    format!("content://{}", AUTHORITY)
}

pub fn build_note_uri() -> String {
    format!("content://{}/notes", AUTHORITY)
}

pub fn build_note_by_id_uri(note_id: i64) -> String {
    format!("content://{}/notes/{}", AUTHORITY, note_id)
}

pub fn build_cards_for_note_uri(note_id: i64) -> String {
    format!("content://{}/notes/{}/cards", AUTHORITY, note_id)
}

pub fn build_specific_card_uri(note_id: i64, ord: &str) -> String {
    format!("content://{}/notes/{}/cards/{}", AUTHORITY, note_id, ord)
}

pub fn build_notes_v2_uri() -> String {
    format!("content://{}/notes_v2", AUTHORITY)
}

pub fn build_models_uri() -> String {
    format!("content://{}/models", AUTHORITY)
}

pub fn build_model_by_id_uri(model_id: i64) -> String {
    format!("content://{}/models/{}", AUTHORITY, model_id)
}

pub fn build_current_model_uri() -> String {
    format!("content://{}/models/current", AUTHORITY)
}

pub fn build_templates_uri(model_id: i64) -> String {
    format!("content://{}/models/{}/templates", AUTHORITY, model_id)
}

pub fn build_template_uri(model_id: i64, template_ord: i32) -> String {
    format!("content://{}/models/{}/templates/{}", AUTHORITY, model_id, template_ord)
}

pub fn build_decks_uri() -> String {
    format!("content://{}/decks", AUTHORITY)
}

pub fn build_selected_deck_uri() -> String {
    format!("content://{}/selected_deck", AUTHORITY)
}

pub fn build_schedule_uri() -> String {
    format!("content://{}/schedule", AUTHORITY)
}

pub fn build_media_uri() -> String {
    format!("content://{}/media", AUTHORITY)
}

/// Note table column constants
pub mod note {
    pub const _ID: &str = "_id";
    pub const GUID: &str = "guid";
    pub const MID: &str = "mid";
    pub const MOD: &str = "mod";
    pub const USN: &str = "usn";
    pub const TAGS: &str = "tags";
    pub const FLDS: &str = "flds";
    pub const SFLD: &str = "sfld";
    pub const CSUM: &str = "csum";
    pub const FLAGS: &str = "flags";
    pub const DATA: &str = "data";
    pub const ALLOW_EMPTY: &str = "allow_empty";
    pub const DECK_ID_QUERY_PARAM: &str = "deckId";

    /// Default projection for note queries
    pub static DEFAULT_PROJECTION: &[&str] = &[
        _ID, GUID, MID, MOD, USN, TAGS, FLDS, SFLD, CSUM, FLAGS, DATA,
    ];

    /// MIME types
    pub const CONTENT_ITEM_TYPE: &str = "vnd.android.cursor.item/vnd.com.ichi2.anki.note";
    pub const CONTENT_TYPE: &str = "vnd.android.cursor.dir/vnd.com.ichi2.anki.note";
}

/// Card table column constants
pub mod card {
    pub const NOTE_ID: &str = "note_id";
    pub const CARD_ORD: &str = "ord";
    pub const CARD_NAME: &str = "card_name";
    pub const DECK_ID: &str = "deck_id";
    pub const QUESTION: &str = "question";
    pub const ANSWER: &str = "answer";
    pub const QUESTION_SIMPLE: &str = "question_simple";
    pub const ANSWER_SIMPLE: &str = "answer_simple";
    pub const ANSWER_PURE: &str = "answer_pure";

    /// Default projection for card queries
    pub static DEFAULT_PROJECTION: &[&str] = &[
        NOTE_ID, CARD_ORD, CARD_NAME, DECK_ID, QUESTION, ANSWER,
    ];

    /// MIME types
    pub const CONTENT_ITEM_TYPE: &str = "vnd.android.cursor.item/vnd.com.ichi2.anki.card";
    pub const CONTENT_TYPE: &str = "vnd.android.cursor.dir/vnd.com.ichi2.anki.card";
}

/// Deck table column constants
pub mod deck {
    pub const DECK_ID: &str = "deck_id";
    pub const DECK_NAME: &str = "deck_name";
    pub const DECK_DESC: &str = "deck_desc";
    pub const DECK_COUNTS: &str = "deck_count";
    pub const OPTIONS: &str = "options";
    pub const DECK_DYN: &str = "deck_dyn";

    /// Default projection for deck queries
    pub static DEFAULT_PROJECTION: &[&str] = &[
        DECK_NAME, DECK_ID, DECK_COUNTS, OPTIONS, DECK_DYN, DECK_DESC,
    ];

    /// MIME types
    pub const CONTENT_TYPE: &str = "vnd.android.cursor.dir/vnd.com.ichi2.anki.deck";
}

/// Model (Note Type) table column constants
pub mod model {
    pub const _ID: &str = "_id";
    pub const NAME: &str = "name";
    pub const FIELD_NAME: &str = "field_name";
    pub const FIELD_NAMES: &str = "field_names";
    pub const NUM_CARDS: &str = "num_cards";
    pub const CSS: &str = "css";
    pub const DECK_ID: &str = "deck_id";
    pub const SORT_FIELD_INDEX: &str = "sort_field_index";
    pub const TYPE: &str = "type";
    pub const LATEX_POST: &str = "latex_post";
    pub const LATEX_PRE: &str = "latex_pre";
    pub const NOTE_COUNT: &str = "note_count";

    /// Special identifier for current model
    pub const CURRENT_MODEL_ID: &str = "current";

    /// Default projection for model queries
    pub static DEFAULT_PROJECTION: &[&str] = &[
        _ID, NAME, FIELD_NAMES, NUM_CARDS, CSS, DECK_ID, SORT_FIELD_INDEX, TYPE, LATEX_POST, LATEX_PRE,
    ];

    /// MIME types
    pub const CONTENT_ITEM_TYPE: &str = "vnd.android.cursor.item/vnd.com.ichi2.anki.model";
    pub const CONTENT_TYPE: &str = "vnd.android.cursor.dir/vnd.com.ichi2.anki.model";
}

/// Card Template table column constants
pub mod card_template {
    pub const _ID: &str = "_id";
    pub const MODEL_ID: &str = "model_id";
    pub const ORD: &str = "ord";
    pub const NAME: &str = "card_template_name";
    pub const QUESTION_FORMAT: &str = "question_format";
    pub const ANSWER_FORMAT: &str = "answer_format";
    pub const BROWSER_QUESTION_FORMAT: &str = "browser_question_format";
    pub const BROWSER_ANSWER_FORMAT: &str = "browser_answer_format";
    pub const CARD_COUNT: &str = "card_count";

    /// Default projection for card template queries
    pub static DEFAULT_PROJECTION: &[&str] = &[
        _ID, MODEL_ID, ORD, NAME, QUESTION_FORMAT, ANSWER_FORMAT,
    ];

    /// MIME types
    pub const CONTENT_TYPE: &str = "vnd.android.cursor.dir/vnd.com.ichi2.anki.model.template";
    pub const CONTENT_ITEM_TYPE: &str = "vnd.android.cursor.item/vnd.com.ichi2.anki.model.template";
}

/// ReviewInfo table column constants
pub mod review_info {
    pub const NOTE_ID: &str = "note_id";
    pub const CARD_ORD: &str = "ord";
    pub const BUTTON_COUNT: &str = "button_count";
    pub const NEXT_REVIEW_TIMES: &str = "next_review_times";
    pub const MEDIA_FILES: &str = "media_files";
    pub const EASE: &str = "answer_ease";
    pub const TIME_TAKEN: &str = "time_taken";
    pub const BURY: &str = "buried";
    pub const SUSPEND: &str = "suspended";

    /// Default projection for review info queries
    pub static DEFAULT_PROJECTION: &[&str] = &[
        NOTE_ID, CARD_ORD, BUTTON_COUNT, NEXT_REVIEW_TIMES, MEDIA_FILES,
    ];

    /// MIME types
    pub const CONTENT_TYPE: &str = "vnd.android.cursor.dir/vnd.com.ichi2.anki.review_info";
}

/// AnkiMedia table column constants
pub mod anki_media {
    pub const FILE_URI: &str = "file_uri";
    pub const PREFERRED_NAME: &str = "preferred_name";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_builders() {
        assert_eq!(build_authority_uri(), "content://com.ichi2.anki.flashcards");
        assert_eq!(build_note_uri(), "content://com.ichi2.anki.flashcards/notes");
        assert_eq!(build_note_by_id_uri(123), "content://com.ichi2.anki.flashcards/notes/123");
        assert_eq!(build_cards_for_note_uri(456), "content://com.ichi2.anki.flashcards/notes/456/cards");
        assert_eq!(build_specific_card_uri(789, "2"), "content://com.ichi2.anki.flashcards/notes/789/cards/2");
        assert_eq!(build_notes_v2_uri(), "content://com.ichi2.anki.flashcards/notes_v2");
        assert_eq!(build_models_uri(), "content://com.ichi2.anki.flashcards/models");
        assert_eq!(build_model_by_id_uri(101112), "content://com.ichi2.anki.flashcards/models/101112");
        assert_eq!(build_current_model_uri(), "content://com.ichi2.anki.flashcards/models/current");
        assert_eq!(build_templates_uri(131415), "content://com.ichi2.anki.flashcards/models/131415/templates");
        assert_eq!(build_decks_uri(), "content://com.ichi2.anki.flashcards/decks");
        assert_eq!(build_selected_deck_uri(), "content://com.ichi2.anki.flashcards/selected_deck");
        assert_eq!(build_schedule_uri(), "content://com.ichi2.anki.flashcards/schedule");
        assert_eq!(build_media_uri(), "content://com.ichi2.anki.flashcards/media");
    }

    #[test]
    fn test_constants() {
        assert_eq!(AUTHORITY, "com.ichi2.anki.flashcards");
        assert_eq!(READ_WRITE_PERMISSION, "com.ichi2.anki.permission.READ_WRITE_DATABASE");
        assert_eq!(DEFAULT_DECK_ID, 1);
        assert_eq!(FIELD_SEPARATOR, '\u{001f}');
    }

    #[test]
    fn test_column_constants() {
        // Test a few key column constants
        assert_eq!(note::_ID, "_id");
        assert_eq!(note::FLDS, "flds");
        assert_eq!(card::NOTE_ID, "note_id");
        assert_eq!(deck::DECK_NAME, "deck_name");
        assert_eq!(model::NAME, "name");
        assert_eq!(review_info::EASE, "answer_ease");
    }

    #[test]
    fn test_default_projections() {
        // Test that default projections contain expected columns
        assert!(note::DEFAULT_PROJECTION.contains(&note::_ID));
        assert!(note::DEFAULT_PROJECTION.contains(&note::FLDS));
        assert!(card::DEFAULT_PROJECTION.contains(&card::NOTE_ID));
        assert!(deck::DEFAULT_PROJECTION.contains(&deck::DECK_NAME));
        assert!(model::DEFAULT_PROJECTION.contains(&model::NAME));
        assert!(review_info::DEFAULT_PROJECTION.contains(&review_info::NOTE_ID));
    }

    #[test]
    fn test_mime_types() {
        // Test a few key MIME types
        assert_eq!(note::CONTENT_TYPE, "vnd.android.cursor.dir/vnd.com.ichi2.anki.note");
        assert_eq!(note::CONTENT_ITEM_TYPE, "vnd.android.cursor.item/vnd.com.ichi2.anki.note");
        assert_eq!(card::CONTENT_TYPE, "vnd.android.cursor.dir/vnd.com.ichi2.anki.card");
        assert_eq!(model::CONTENT_TYPE, "vnd.android.cursor.dir/vnd.com.ichi2.anki.model");
    }
}
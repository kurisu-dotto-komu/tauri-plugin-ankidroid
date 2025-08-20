/// AnkiDroid ContentProvider constants and URIs
/// Based on FlashCardsContract specification

// ContentProvider authorities
pub const AUTHORITY: &str = "com.ichi2.anki.flashcards";
pub const FALLBACK_AUTHORITY: &str = "com.ichi2.anki.provider";

// Base URIs
pub const NOTES_URI: &str = "content://com.ichi2.anki.flashcards/notes";
pub const CARDS_URI: &str = "content://com.ichi2.anki.flashcards/cards";
pub const DECKS_URI: &str = "content://com.ichi2.anki.flashcards/decks";
pub const MODELS_URI: &str = "content://com.ichi2.anki.flashcards/models";
pub const SCHEDULE_URI: &str = "content://com.ichi2.anki.flashcards/schedule";
pub const REVIEW_INFO_URI: &str = "content://com.ichi2.anki.flashcards/review_info";
pub const MEDIA_URI: &str = "content://com.ichi2.anki.flashcards/media";

// Note table columns
pub mod note_columns {
    pub const ID: &str = "_id";
    pub const GUID: &str = "guid";
    pub const MID: &str = "mid"; // Model ID
    pub const MOD: &str = "mod"; // Modification time
    pub const USQN: &str = "usn"; // Update sequence number (not usqn)
    pub const TAGS: &str = "tags";
    pub const FLDS: &str = "flds"; // Fields (separated by 0x1f)
    pub const FLAGS: &str = "flags";
    pub const DATA: &str = "data";
    pub const SFLD: &str = "sfld"; // Sort field
    pub const CSUM: &str = "csum"; // Checksum
}

// Deck table columns
pub mod deck_columns {
    pub const DECK_ID: &str = "deck_id";
    pub const DECK_NAME: &str = "deck_name";
    pub const DID: &str = "did"; // Alternative deck ID column
    pub const NAME: &str = "name"; // Alternative name column
    pub const DECK_DESC: &str = "deck_desc";
    pub const DECK_DYN: &str = "deck_dyn";
    pub const DECK_COUNTS: &str = "deck_counts";
    pub const OPTIONS: &str = "deck_options";
    pub const DECK_CONF: &str = "deck_conf";
}

// Model table columns
pub mod model_columns {
    pub const MID: &str = "mid"; // Model ID
    pub const NAME: &str = "name";
    pub const FLDS: &str = "flds"; // Fields JSON array
    pub const TMPLS: &str = "tmpls"; // Templates JSON array
    pub const CSS: &str = "css";
    pub const TYPE: &str = "type"; // 0 = standard, 1 = cloze
    pub const LATEX_PRE: &str = "latex_pre";
    pub const LATEX_POST: &str = "latex_post";
    pub const SORT_FIELD: &str = "sort_field";
    pub const REQ: &str = "req"; // Required fields array
    pub const DID: &str = "did"; // Default deck ID
}

// Card table columns
pub mod card_columns {
    pub const ID: &str = "_id";
    pub const NID: &str = "nid"; // Note ID
    pub const DID: &str = "did"; // Deck ID
    pub const ORD: &str = "ord"; // Card ordinal (template index)
    pub const MOD: &str = "mod"; // Modification time
    pub const TYPE: &str = "type"; // Card type
    pub const QUEUE: &str = "queue"; // Card queue
    pub const DUE: &str = "due"; // Due date/time
    pub const IVL: &str = "ivl"; // Interval in days
    pub const FACTOR: &str = "factor"; // Ease factor (x1000)
    pub const REPS: &str = "reps"; // Number of reviews
    pub const LAPSES: &str = "lapses"; // Number of lapses
    pub const LEFT: &str = "left"; // Reviews left today
    pub const ODUE: &str = "odue"; // Original due
    pub const ODID: &str = "odid"; // Original deck ID
    pub const FLAGS: &str = "flags"; // Card flags
    pub const DATA: &str = "data"; // Extra data
}

// Review info columns
pub mod review_columns {
    pub const CARD_ORD: &str = "card_ord";
    pub const BUTTON: &str = "button";
    pub const TIME: &str = "time";
    pub const EASE: &str = "ease";
    pub const NOTE_ID: &str = "note_id";
    pub const CARD_ID: &str = "card_id";
    pub const NEXT_REVIEW: &str = "next_review";
    pub const SUSPEND: &str = "suspend";
}

// Media columns
pub mod media_columns {
    pub const FILE_URI: &str = "file_uri";
    pub const PREFERRED_NAME: &str = "preferred_name";
}

// Field separator for note fields
pub const FIELD_SEPARATOR: char = '\u{001f}';

// Card queue types
pub mod card_queue {
    pub const USER_BURIED: i32 = -3;
    pub const SCHED_BURIED: i32 = -2;
    pub const SUSPENDED: i32 = -1;
    pub const NEW: i32 = 0;
    pub const LEARNING: i32 = 1;
    pub const REVIEW: i32 = 2;
    pub const DAY_LEARN_RELEARN: i32 = 3;
    pub const PREVIEW: i32 = 4;
}

// Card types
pub mod card_type {
    pub const NEW: i32 = 0;
    pub const LEARNING: i32 = 1;
    pub const REVIEW: i32 = 2;
    pub const RELEARNING: i32 = 3;
}

// Model types
pub mod model_type {
    pub const STANDARD: i32 = 0;
    pub const CLOZE: i32 = 1;
}

// Default values
pub const DEFAULT_DECK_ID: i64 = 1;
pub const DEFAULT_MODEL_NAME: &str = "Basic";
pub const DEFAULT_BASIC_MODEL_ID: i64 = 1607392319495;

// Permission constants
pub const READ_WRITE_PERMISSION: &str = "com.ichi2.anki.permission.READ_WRITE_DATABASE";

// Package names
pub const ANKIDROID_PACKAGE: &str = "com.ichi2.anki";
pub const ANKIDROID_DEBUG_PACKAGE: &str = "com.ichi2.anki.debug";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_separator() {
        assert_eq!(FIELD_SEPARATOR, '\u{001f}');
        let fields = format!("front{}back", FIELD_SEPARATOR);
        assert_eq!(fields, "front\u{001f}back");
    }

    #[test]
    fn test_uri_constants() {
        assert!(NOTES_URI.starts_with("content://"));
        assert!(NOTES_URI.contains(AUTHORITY));
        assert_eq!(AUTHORITY, "com.ichi2.anki.flashcards");
    }

    #[test]
    fn test_card_queue_constants() {
        assert_eq!(card_queue::SUSPENDED, -1);
        assert_eq!(card_queue::NEW, 0);
        assert_eq!(card_queue::REVIEW, 2);
    }

    #[test]
    fn test_model_type_constants() {
        assert_eq!(model_type::STANDARD, 0);
        assert_eq!(model_type::CLOZE, 1);
    }
}

//! Example demonstrating how to use the FlashCardsContract constants and URI builders
//! 
//! This example shows how to build URIs for various AnkiDroid content provider operations.

use ankidroid_api_rust::{
    build_authority_uri, build_note_uri, build_note_by_id_uri, build_cards_for_note_uri,
    build_specific_card_uri, build_models_uri, build_decks_uri, build_schedule_uri,
    AUTHORITY, READ_WRITE_PERMISSION, FIELD_SEPARATOR, note, card, deck, model
};

fn main() {
    println!("AnkiDroid FlashCards Contract Examples");
    println!("======================================");
    
    // Basic constants
    println!("\n1. Basic Constants:");
    println!("   Authority: {}", AUTHORITY);
    println!("   Permission: {}", READ_WRITE_PERMISSION);
    println!("   Field Separator: {:?}", FIELD_SEPARATOR);
    
    // URI building examples
    println!("\n2. URI Building:");
    println!("   Authority URI: {}", build_authority_uri());
    println!("   Notes URI: {}", build_note_uri());
    println!("   Specific Note URI: {}", build_note_by_id_uri(12345));
    println!("   Cards for Note URI: {}", build_cards_for_note_uri(12345));
    println!("   Specific Card URI: {}", build_specific_card_uri(12345, "0"));
    println!("   Models URI: {}", build_models_uri());
    println!("   Decks URI: {}", build_decks_uri());
    println!("   Schedule URI: {}", build_schedule_uri());
    
    // Column constants examples
    println!("\n3. Column Constants:");
    println!("   Note columns: {:?}", note::DEFAULT_PROJECTION);
    println!("   Card columns: {:?}", card::DEFAULT_PROJECTION);
    println!("   Deck columns: {:?}", deck::DEFAULT_PROJECTION);
    println!("   Model columns: {:?}", model::DEFAULT_PROJECTION);
    
    // MIME types
    println!("\n4. MIME Types:");
    println!("   Note content type: {}", note::CONTENT_TYPE);
    println!("   Card content type: {}", card::CONTENT_TYPE);
    println!("   Deck content type: {}", deck::CONTENT_TYPE);
    println!("   Model content type: {}", model::CONTENT_TYPE);
    
    println!("\nAll constants and builders are working correctly!");
}
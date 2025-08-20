//! Example demonstrating the ContentResolver and Cursor JNI wrappers
//!
//! This example shows how to use the ContentResolver and Cursor wrappers
//! to query AnkiDroid's content provider. Note that this example requires
//! an Android environment to run.

#[cfg(target_os = "android")]
use ankidroid_api_rust::jni::{ContentResolver, Cursor, ContentValuesBuilder, SafeJNIEnv};
#[cfg(target_os = "android")]
use ankidroid_api_rust::contract::{AUTHORITY, DECKS_URI_PATH};
#[cfg(target_os = "android")]
use ankidroid_api_rust::error::Result;
#[cfg(target_os = "android")]
use jni::objects::JObject;
#[cfg(target_os = "android")]
use jni::JNIEnv;

#[cfg(target_os = "android")]
fn example_query_decks(env: JNIEnv, context: &JObject) -> Result<Vec<(i64, String)>> {
    let mut safe_env = SafeJNIEnv::new(env);
    
    // Create a ContentResolver from the Android context
    let mut content_resolver = ContentResolver::from_context(safe_env.clone(), context)?;
    
    // Build the URI for querying decks
    let uri = format!("content://{}/{}", AUTHORITY, DECKS_URI_PATH);
    
    // Query for deck information
    let mut cursor = content_resolver.query(
        &uri,
        Some(vec!["_id".to_string(), "name".to_string()]), // projection
        Some("active = ?".to_string()),                     // selection
        Some(vec!["1".to_string()]),                        // selection args
        Some("name ASC".to_string()),                       // sort order
    )?;
    
    let mut decks = Vec::new();
    
    // Method 1: Manual iteration
    if cursor.move_to_first()? {
        loop {
            let id = cursor.get_long_by_name("_id")?;
            let name = cursor.get_string_by_name("name")?;
            decks.push((id, name));
            
            if !cursor.move_next()? {
                break;
            }
        }
    }
    
    println!("Found {} decks", decks.len());
    for (id, name) in &decks {
        println!("Deck {}: {}", id, name);
    }
    
    Ok(decks)
}

#[cfg(target_os = "android")]
fn example_query_with_iterator(env: JNIEnv, context: &JObject) -> Result<Vec<String>> {
    let mut safe_env = SafeJNIEnv::new(env);
    let mut content_resolver = ContentResolver::from_context(safe_env.clone(), context)?;
    
    let uri = format!("content://{}/{}", AUTHORITY, DECKS_URI_PATH);
    let mut cursor = content_resolver.query(&uri, None, None, None, None)?;
    
    let mut deck_names = Vec::new();
    
    // Method 2: Using iterator
    for row_result in cursor.iter() {
        row_result?; // Check for iteration errors
        let name = cursor.get_string_by_name("name")?;
        deck_names.push(name);
    }
    
    Ok(deck_names)
}

#[cfg(target_os = "android")]
fn example_insert_deck(env: JNIEnv, context: &JObject, deck_name: &str) -> Result<String> {
    let mut safe_env = SafeJNIEnv::new(env);
    let mut content_resolver = ContentResolver::from_context(safe_env.clone(), context)?;
    
    // Build ContentValues for the new deck
    let values = ContentValuesBuilder::new(&mut safe_env)?
        .put_string("name", deck_name)?
        .put_long("timestamp", chrono::Utc::now().timestamp())?
        .put_int("active", 1)?;
    
    let uri = format!("content://{}/{}", AUTHORITY, DECKS_URI_PATH);
    let inserted_uri = content_resolver.insert(&uri, values)?;
    
    println!("Inserted deck '{}' with URI: {}", deck_name, inserted_uri);
    Ok(inserted_uri)
}

#[cfg(target_os = "android")]
fn example_update_deck(env: JNIEnv, context: &JObject, deck_id: i64, new_name: &str) -> Result<i32> {
    let mut safe_env = SafeJNIEnv::new(env);
    let mut content_resolver = ContentResolver::from_context(safe_env.clone(), context)?;
    
    let values = ContentValuesBuilder::new(&mut safe_env)?
        .put_string("name", new_name)?
        .put_long("timestamp", chrono::Utc::now().timestamp())?;
    
    let uri = format!("content://{}/{}", AUTHORITY, DECKS_URI_PATH);
    let updated_count = content_resolver.update(
        &uri,
        values,
        Some("_id = ?".to_string()),
        Some(vec![deck_id.to_string()]),
    )?;
    
    println!("Updated {} deck(s)", updated_count);
    Ok(updated_count)
}

#[cfg(target_os = "android")]
fn example_delete_deck(env: JNIEnv, context: &JObject, deck_id: i64) -> Result<i32> {
    let mut safe_env = SafeJNIEnv::new(env);
    let mut content_resolver = ContentResolver::from_context(safe_env.clone(), context)?;
    
    let uri = format!("content://{}/{}", AUTHORITY, DECKS_URI_PATH);
    let deleted_count = content_resolver.delete(
        &uri,
        Some("_id = ?".to_string()),
        Some(vec![deck_id.to_string()]),
    )?;
    
    println!("Deleted {} deck(s)", deleted_count);
    Ok(deleted_count)
}

#[cfg(target_os = "android")]
fn example_bulk_insert(env: JNIEnv, context: &JObject, deck_names: &[&str]) -> Result<i32> {
    let mut safe_env = SafeJNIEnv::new(env);
    let mut content_resolver = ContentResolver::from_context(safe_env.clone(), context)?;
    
    let mut values_array = Vec::new();
    for name in deck_names {
        let values = ContentValuesBuilder::new(&mut safe_env)?
            .put_string("name", name)?
            .put_long("timestamp", chrono::Utc::now().timestamp())?
            .put_int("active", 1)?;
        values_array.push(values);
    }
    
    let uri = format!("content://{}/{}", AUTHORITY, DECKS_URI_PATH);
    let inserted_count = content_resolver.bulk_insert(&uri, values_array)?;
    
    println!("Bulk inserted {} deck(s)", inserted_count);
    Ok(inserted_count)
}

#[cfg(target_os = "android")]
fn example_cursor_info(env: JNIEnv, context: &JObject) -> Result<()> {
    let mut safe_env = SafeJNIEnv::new(env);
    let mut content_resolver = ContentResolver::from_context(safe_env.clone(), context)?;
    
    let uri = format!("content://{}/{}", AUTHORITY, DECKS_URI_PATH);
    let mut cursor = content_resolver.query(&uri, None, None, None, None)?;
    
    // Get cursor metadata
    let count = cursor.get_count()?;
    let column_count = cursor.get_column_count()?;
    let column_names = cursor.get_column_names()?;
    
    println!("Cursor info:");
    println!("  Rows: {}", count);
    println!("  Columns: {}", column_count);
    println!("  Column names: {:?}", column_names);
    
    // Check for null values
    if cursor.move_to_first()? {
        for (i, column_name) in column_names.iter().enumerate() {
            let is_null = cursor.is_null(i as i32)?;
            println!("  Column '{}' is null: {}", column_name, is_null);
        }
    }
    
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn main() {
    println!("This example only works on Android targets.");
    println!("To run this example:");
    println!("1. Build for Android target: cargo build --target aarch64-linux-android");
    println!("2. Include in an Android app that uses the ankidroid-api-rust library");
}

#[cfg(target_os = "android")]
fn main() {
    println!("ContentResolver and Cursor example functions are available.");
    println!("These functions need to be called from an Android JNI context with:");
    println!("- A valid JNIEnv");
    println!("- An Android Context object");
    println!("- AnkiDroid app installed and accessible");
}
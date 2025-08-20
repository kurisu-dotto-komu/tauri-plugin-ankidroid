//! Example demonstrating the utils module functionality
//! 
//! This example shows how to use the utility functions for field/tag manipulation
//! and HTML processing that match AnkiDroid's Utils class behavior.

use ankidroid_api_rust::utils::{join_fields, split_fields, join_tags, split_tags, strip_html, strip_html_entities};

#[cfg(target_os = "android")]
use ankidroid_api_rust::utils::field_checksum;

fn main() {
    println!("AnkiDroid API Rust - Utils Examples");
    println!("====================================");
    
    // Field operations
    println!("\n1. Field Operations:");
    let fields = ["Front text", "Back text", "Extra info"];
    let joined = join_fields(&fields);
    println!("Joined fields: {:?}", joined);
    
    let split = split_fields(&joined);
    println!("Split fields: {:?}", split);
    
    // Tag operations
    println!("\n2. Tag Operations:");
    let tags = ["spanish", "vocabulary", "beginner level"];
    let joined_tags = join_tags(&tags);
    println!("Joined tags: {:?}", joined_tags);
    
    let split_tags_result = split_tags(&joined_tags);
    println!("Split tags: {:?}", split_tags_result);
    
    // HTML operations
    println!("\n3. HTML Operations:");
    let html = r#"<b>Bold text</b> with <i>italic</i> and <img src="image.jpg"> media"#;
    println!("Original HTML: {}", html);
    
    let stripped = strip_html(html);
    println!("Stripped HTML: {}", stripped);
    
    let entities = "Text with &amp; entities &lt;tag&gt; and &#65; numeric";
    println!("Original entities: {}", entities);
    
    let decoded = strip_html_entities(entities);
    println!("Decoded entities: {}", decoded);
    
    // Checksum (Android only)
    #[cfg(target_os = "android")]
    {
        println!("\n4. Checksum Operations (Android only):");
        let data = "AnkiDroid";
        let checksum = field_checksum(data);
        println!("Checksum for '{}': {}", data, checksum);
        
        let html_data = "<b>AnkiDroid</b>";
        let html_checksum = field_checksum(html_data);
        println!("Checksum for '{}': {}", html_data, html_checksum);
        println!("Checksums match (HTML stripped): {}", checksum == html_checksum);
    }
    
    #[cfg(not(target_os = "android"))]
    {
        println!("\n4. Checksum Operations:");
        println!("(Only available on Android targets)");
    }
    
    println!("\nAll examples completed successfully!");
}
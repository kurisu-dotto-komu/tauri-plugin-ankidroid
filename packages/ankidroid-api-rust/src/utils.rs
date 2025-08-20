//! Utility functions for field/tag manipulation and checksum calculation
//! 
//! This module provides utilities that match AnkiDroid's AddContentApi.kt Utils class
//! for field and tag manipulation, HTML processing, and checksum calculation.

use regex::Regex;
use std::sync::OnceLock;

#[cfg(target_os = "android")]
use sha1::{Digest, Sha1};

/// Field separator character used in AnkiDroid
const FIELD_SEPARATOR: char = '\u{001f}';

/// Static regex patterns for HTML processing
static STYLE_PATTERN: OnceLock<Regex> = OnceLock::new();
static SCRIPT_PATTERN: OnceLock<Regex> = OnceLock::new();
static TAG_PATTERN: OnceLock<Regex> = OnceLock::new();
#[cfg(target_os = "android")]
static IMG_PATTERN: OnceLock<Regex> = OnceLock::new();
static HTML_ENTITIES_PATTERN: OnceLock<Regex> = OnceLock::new();

fn get_style_pattern() -> &'static Regex {
    STYLE_PATTERN.get_or_init(|| {
        Regex::new(r"(?s)<style.*?>.*?</style>").unwrap()
    })
}

fn get_script_pattern() -> &'static Regex {
    SCRIPT_PATTERN.get_or_init(|| {
        Regex::new(r"(?s)<script.*?>.*?</script>").unwrap()
    })
}

fn get_tag_pattern() -> &'static Regex {
    TAG_PATTERN.get_or_init(|| {
        Regex::new(r"<.*?>").unwrap()
    })
}

#[cfg(target_os = "android")]
fn get_img_pattern() -> &'static Regex {
    IMG_PATTERN.get_or_init(|| {
        Regex::new(r#"<img src=[\"']?([^\"'>]+)[\"']? ?/?>"#).unwrap()
    })
}

fn get_html_entities_pattern() -> &'static Regex {
    HTML_ENTITIES_PATTERN.get_or_init(|| {
        Regex::new(r"&#?\w+;").unwrap()
    })
}

/// Join fields with the field separator character
/// 
/// # Arguments
/// * `fields` - Array of field strings to join
/// 
/// # Returns
/// String with fields joined by the field separator
/// 
/// # Example
/// ```
/// use ankidroid_api_rust::utils::join_fields;
/// 
/// let fields = ["Front", "Back", "Extra"];
/// let result = join_fields(&fields);
/// assert_eq!(result, "Front\u{001f}Back\u{001f}Extra");
/// ```
pub fn join_fields(fields: &[&str]) -> String {
    fields.join(&FIELD_SEPARATOR.to_string())
}

/// Split fields by the field separator character
/// 
/// # Arguments
/// * `fields` - String containing fields separated by field separator
/// 
/// # Returns
/// Vector of field strings
/// 
/// # Example
/// ```
/// use ankidroid_api_rust::utils::split_fields;
/// 
/// let fields_str = "Front\u{001f}Back\u{001f}Extra";
/// let result = split_fields(fields_str);
/// assert_eq!(result, vec!["Front", "Back", "Extra"]);
/// ```
pub fn split_fields(fields: &str) -> Vec<String> {
    fields
        .split(FIELD_SEPARATOR)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .skip_while(|s| s.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// Join tags with spaces
/// 
/// Note: This matches AnkiDroid's Utils.joinTags implementation, which has a bug
/// where it attempts to replace spaces with underscores but doesn't actually do it.
/// For compatibility, this implementation also does not replace spaces.
/// 
/// # Arguments
/// * `tags` - Array of tag strings to join
/// 
/// # Returns
/// String with tags joined by spaces
/// 
/// # Example
/// ```
/// use ankidroid_api_rust::utils::join_tags;
/// 
/// let tags = ["tag1", "tag with space", "tag3"];
/// let result = join_tags(&tags);
/// assert_eq!(result, "tag1 tag with space tag3");
/// ```
pub fn join_tags(tags: &[&str]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    
    // Note: The original Kotlin code has a bug where it calls replace but doesn't store the result
    // We match that behavior for compatibility
    for _tag in tags {
        // Simulate the buggy behavior: call replace but don't use the result
        let _ = _tag.replace(' ', "_");
    }
    
    tags.join(" ")
}

/// Split tags by whitespace
/// 
/// # Arguments
/// * `tags` - String containing tags separated by whitespace
/// 
/// # Returns
/// Vector of tag strings
/// 
/// # Example
/// ```
/// use ankidroid_api_rust::utils::split_tags;
/// 
/// let tags_str = "tag1  tag2\ttag3";
/// let result = split_tags(tags_str);
/// assert_eq!(result, vec!["tag1", "tag2", "tag3"]);
/// ```
pub fn split_tags(tags: &str) -> Vec<String> {
    tags.trim()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Calculate field checksum for duplicate detection
/// 
/// This function strips HTML tags while preserving media filenames,
/// converts HTML entities to text, and calculates a SHA1 hash.
/// The first 8 hex characters are returned as an i64.
/// 
/// # Arguments
/// * `data` - Field data to calculate checksum for
/// 
/// # Returns
/// Checksum as i64 for duplicate detection
/// 
/// # Example
/// ```
/// use ankidroid_api_rust::utils::field_checksum;
/// 
/// let data = "<b>Hello</b> <img src=\"image.jpg\"> &amp; world";
/// let checksum = field_checksum(data);
/// // Returns checksum as i64
/// ```
#[cfg(target_os = "android")]
pub fn field_checksum(data: &str) -> i64 {
    let stripped_data = strip_html_media(data);
    
    let mut hasher = Sha1::new();
    hasher.update(stripped_data.as_bytes());
    let digest = hasher.finalize();
    
    // Convert to hex string
    let hex_string = format!("{:x}", digest);
    
    // Pad to 40 characters (SHA1 is always 40 hex chars, but format! might not pad)
    let padded_hex = format!("{:0>40}", hex_string);
    
    // Take first 8 characters and convert to i64
    let checksum_str = &padded_hex[0..8];
    i64::from_str_radix(checksum_str, 16).unwrap_or(0)
}

/// Strip HTML but keep media filenames
/// 
/// # Arguments
/// * `s` - HTML string to process
/// 
/// # Returns
/// String with HTML removed but media filenames preserved
#[cfg(target_os = "android")]
fn strip_html_media(s: &str) -> String {
    let img_pattern = get_img_pattern();
    let with_media = img_pattern.replace_all(s, " $1 ");
    strip_html(&with_media)
}

/// Strip HTML tags from string
/// 
/// Removes style tags, script tags, and all other HTML tags,
/// then converts HTML entities to text.
/// 
/// # Arguments
/// * `s` - HTML string to strip
/// 
/// # Returns
/// Plain text string
/// 
/// # Example
/// ```
/// use ankidroid_api_rust::utils::strip_html;
/// 
/// let html = "<b>Bold</b> <i>italic</i> &amp; text";
/// let result = strip_html(html);
/// assert_eq!(result, "Bold italic & text");
/// ```
pub fn strip_html(s: &str) -> String {
    let style_pattern = get_style_pattern();
    let script_pattern = get_script_pattern();
    let tag_pattern = get_tag_pattern();
    
    // Remove style tags
    let no_style = style_pattern.replace_all(s, "");
    
    // Remove script tags
    let no_script = script_pattern.replace_all(&no_style, "");
    
    // Remove all other HTML tags
    let no_tags = tag_pattern.replace_all(&no_script, "");
    
    // Convert HTML entities to text
    strip_html_entities(&no_tags)
}

/// Convert HTML entities to text
/// 
/// Replaces HTML entities like &amp;, &lt;, &gt;, &quot;, &#39;, &nbsp; with their text equivalents.
/// 
/// # Arguments
/// * `text` - Text containing HTML entities
/// 
/// # Returns
/// Text with HTML entities converted to their text equivalents
/// 
/// # Example
/// ```
/// use ankidroid_api_rust::utils::strip_html_entities;
/// 
/// let text = "A &amp; B &lt; C &gt; D &quot;quoted&quot; &nbsp; space";
/// let result = strip_html_entities(text);
/// assert_eq!(result, "A & B < C > D \"quoted\"   space");
/// ```
pub fn strip_html_entities(text: &str) -> String {
    // Handle &nbsp; first (replace with regular space)
    let text = text.replace("&nbsp;", " ");
    
    // Handle common HTML entities
    let text = text.replace("&amp;", "&");
    let text = text.replace("&lt;", "<");
    let text = text.replace("&gt;", ">");
    let text = text.replace("&quot;", "\"");
    let text = text.replace("&#39;", "'");
    let text = text.replace("&#x27;", "'");
    let text = text.replace("&#x2F;", "/");
    
    // Handle numeric entities (basic implementation)
    let entities_pattern = get_html_entities_pattern();
    let result = entities_pattern.replace_all(&text, |caps: &regex::Captures| {
        let entity = &caps[0];
        match entity {
            "&amp;" => "&".to_string(),
            "&lt;" => "<".to_string(),
            "&gt;" => ">".to_string(),
            "&quot;" => "\"".to_string(),
            "&#39;" => "'".to_string(),
            "&nbsp;" => " ".to_string(),
            _ => {
                // For other entities, try to parse numeric ones
                if entity.starts_with("&#") {
                    let num_part = &entity[2..entity.len()-1];
                    if let Ok(code) = num_part.parse::<u32>() {
                        if let Some(ch) = char::from_u32(code) {
                            return ch.to_string();
                        }
                    } else if num_part.starts_with('x') || num_part.starts_with('X') {
                        let hex_part = &num_part[1..];
                        if let Ok(code) = u32::from_str_radix(hex_part, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                return ch.to_string();
                            }
                        }
                    }
                }
                // If we can't decode it, just remove the entity
                "".to_string()
            }
        }
    });
    
    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_fields() {
        let fields = ["Front", "Back", "Extra"];
        let result = join_fields(&fields);
        assert_eq!(result, "Front\u{001f}Back\u{001f}Extra");
    }

    #[test]
    fn test_join_fields_empty() {
        let fields: [&str; 0] = [];
        let result = join_fields(&fields);
        assert_eq!(result, "");
    }

    #[test]
    fn test_split_fields() {
        let fields_str = "Front\u{001f}Back\u{001f}Extra";
        let result = split_fields(fields_str);
        assert_eq!(result, vec!["Front", "Back", "Extra"]);
    }

    #[test]
    fn test_split_fields_with_empty_trailing() {
        let fields_str = "Front\u{001f}Back\u{001f}";
        let result = split_fields(fields_str);
        assert_eq!(result, vec!["Front", "Back"]);
    }

    #[test]
    fn test_join_tags() {
        let tags = ["tag1", "tag with space", "tag3"];
        let result = join_tags(&tags);
        assert_eq!(result, "tag1 tag with space tag3");
    }

    #[test]
    fn test_join_tags_empty() {
        let tags: [&str; 0] = [];
        let result = join_tags(&tags);
        assert_eq!(result, "");
    }

    #[test]
    fn test_split_tags() {
        let tags_str = "tag1  tag2\ttag3\nTag4";
        let result = split_tags(tags_str);
        assert_eq!(result, vec!["tag1", "tag2", "tag3", "Tag4"]);
    }

    #[test]
    fn test_split_tags_with_leading_trailing_whitespace() {
        let tags_str = "  tag1 tag2  ";
        let result = split_tags(tags_str);
        assert_eq!(result, vec!["tag1", "tag2"]);
    }
    
    // Tests matching AnkiDroid's ApiUtilsTest.kt
    
    #[test]
    fn test_join_fields_kotlin_compatibility() {
        let field_list = ["A", "B", "C"];
        let expected = format!("A{}B{}C", FIELD_SEPARATOR, FIELD_SEPARATOR);
        assert_eq!(join_fields(&field_list), expected);
    }
    
    #[test]
    fn test_split_fields_kotlin_compatibility() {
        let field_list = format!("A{}B{}C", FIELD_SEPARATOR, FIELD_SEPARATOR);
        let output = split_fields(&field_list);
        assert_eq!(output[0], "A");
        assert_eq!(output[1], "B");
        assert_eq!(output[2], "C");
    }
    
    #[test]
    fn test_join_tags_kotlin_compatibility() {
        let tags = ["A", "B", "C"];
        assert_eq!(join_tags(&tags), "A B C");
    }
    
    #[test]
    fn test_split_tags_kotlin_compatibility() {
        let tags = "A B C";
        let output = split_tags(tags);
        assert_eq!(output[0], "A");
        assert_eq!(output[1], "B");
        assert_eq!(output[2], "C");
    }

    #[test]
    fn test_strip_html() {
        let html = "<b>Bold</b> <i>italic</i> text";
        let result = strip_html(html);
        assert_eq!(result, "Bold italic text");
    }

    #[test]
    fn test_strip_html_with_style_and_script() {
        let html = "<style>body { color: red; }</style><script>alert('hi');</script><b>Content</b>";
        let result = strip_html(html);
        assert_eq!(result, "Content");
    }

    #[test]
    fn test_strip_html_entities() {
        let text = "A &amp; B &lt; C &gt; D &quot;quoted&quot; &nbsp; space";
        let result = strip_html_entities(text);
        assert_eq!(result, "A & B < C > D \"quoted\"   space");
    }

    #[test]
    fn test_strip_html_entities_numeric() {
        let text = "&#65;&#x42;&#39;";  // A, B, '
        let result = strip_html_entities(text);
        assert_eq!(result, "AB'");
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_strip_html_media() {
        let html = r#"<b>Text</b> <img src="image.jpg"> more text"#;
        let result = strip_html_media(html);
        assert_eq!(result, "Text  image.jpg  more text");
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_strip_html_media_with_quotes() {
        let html = r#"<img src='image.png'/> and <img src="photo.gif" />"#;
        let result = strip_html_media(html);
        assert_eq!(result, " image.png  and  photo.gif ");
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_field_checksum() {
        // Test with simple text
        let data = "Hello World";
        let checksum = field_checksum(data);
        assert!(checksum > 0);
        
        // Test that same input gives same checksum
        let checksum2 = field_checksum(data);
        assert_eq!(checksum, checksum2);
        
        // Test with HTML
        let html_data = "<b>Hello</b> World";
        let html_checksum = field_checksum(html_data);
        // Should be same as plain text since HTML is stripped
        assert_eq!(checksum, html_checksum);
    }
    
    #[cfg(target_os = "android")]
    #[test]
    fn test_field_checksum_ankidroid_compatibility() {
        // Test with the exact value from AnkiDroid's ApiUtilsTest.kt
        // This ensures our checksum implementation matches AnkiDroid exactly
        let checksum = field_checksum("AnkiDroid");
        assert_eq!(checksum, 3533307532);
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_field_checksum_with_media() {
        let data = r#"<b>Text</b> <img src="image.jpg"> &amp; more"#;
        let checksum = field_checksum(data);
        assert!(checksum > 0);
        
        // The checksum should be calculated on "Text  image.jpg  & more"
        let expected_content = "Text  image.jpg  & more";
        let expected_checksum = field_checksum(expected_content);
        assert_eq!(checksum, expected_checksum);
    }
}
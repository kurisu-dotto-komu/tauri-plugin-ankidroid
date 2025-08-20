//! Integration tests for Android-specific utils functionality
//! 
//! These tests verify that our implementation matches AnkiDroid's behavior exactly.

#[cfg(target_os = "android")]
mod android_tests {
    use ankidroid_api_rust::utils::field_checksum;
    
    #[test]
    fn test_checksum_compatibility_ankidroid() {
        // This is the exact test case from AnkiDroid's ApiUtilsTest.kt
        // It ensures our implementation produces the same checksum as AnkiDroid
        assert_eq!(field_checksum("AnkiDroid"), 3533307532);
    }
    
    #[test]
    fn test_checksum_with_html_stripping() {
        // Test that HTML is properly stripped before checksum calculation
        let plain_text = "Hello World";
        let html_text = "<b>Hello</b> <i>World</i>";
        let complex_html = "<style>body{color:red;}</style><script>alert('hi');</script><b>Hello</b> <i>World</i>";
        
        let plain_checksum = field_checksum(plain_text);
        let html_checksum = field_checksum(html_text);
        let complex_checksum = field_checksum(complex_html);
        
        // All should produce the same checksum since HTML is stripped
        assert_eq!(plain_checksum, html_checksum);
        assert_eq!(plain_checksum, complex_checksum);
    }
    
    #[test]
    fn test_checksum_with_media_preservation() {
        // Test that media filenames are preserved in checksum calculation
        let text_with_img = r#"Front text <img src="image.jpg"> back text"#;
        let expected_content = "Front text  image.jpg  back text";
        
        let img_checksum = field_checksum(text_with_img);
        let expected_checksum = field_checksum(expected_content);
        
        assert_eq!(img_checksum, expected_checksum);
    }
    
    #[test]
    fn test_checksum_with_entities() {
        // Test that HTML entities are properly decoded before checksum
        let text_with_entities = "Text &amp; more &lt;test&gt;";
        let expected_content = "Text & more <test>";
        
        let entities_checksum = field_checksum(text_with_entities);
        let expected_checksum = field_checksum(expected_content);
        
        assert_eq!(entities_checksum, expected_checksum);
    }
}

#[cfg(not(target_os = "android"))]
mod non_android_tests {
    // On non-Android platforms, we can only test the non-checksum functions
    use ankidroid_api_rust::utils::{join_fields, split_fields, join_tags, split_tags};
    
    #[test]
    fn test_field_operations() {
        let fields = ["A", "B", "C"];
        let joined = join_fields(&fields);
        let split = split_fields(&joined);
        assert_eq!(split, vec!["A", "B", "C"]);
    }
    
    #[test]
    fn test_tag_operations() {
        let tags = ["tag1", "tag2", "tag3"];
        let joined = join_tags(&tags);
        let split = split_tags(&joined);
        assert_eq!(split, vec!["tag1", "tag2", "tag3"]);
    }
}
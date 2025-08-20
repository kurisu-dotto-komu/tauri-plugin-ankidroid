use crate::android::error::{AndroidError, AndroidResult};

/// Validation helpers for JNI parameters and Android-specific constraints
pub struct ValidationHelper;

impl ValidationHelper {
    /// Validate card content ensuring it meets AnkiDroid requirements
    pub fn validate_card_content(front: &str, back: &str) -> AndroidResult<()> {
        if front.trim().is_empty() {
            return Err(AndroidError::validation_error("Card front cannot be empty"));
        }
        
        if back.trim().is_empty() {
            return Err(AndroidError::validation_error("Card back cannot be empty"));
        }
        
        // Check for reasonable content length (AnkiDroid limit is ~131KB per field)
        const MAX_FIELD_LENGTH: usize = 131_072;
        if front.len() > MAX_FIELD_LENGTH {
            return Err(AndroidError::validation_error(
                format!("Card front exceeds maximum length of {} characters", MAX_FIELD_LENGTH)
            ));
        }
        
        if back.len() > MAX_FIELD_LENGTH {
            return Err(AndroidError::validation_error(
                format!("Card back exceeds maximum length of {} characters", MAX_FIELD_LENGTH)
            ));
        }
        
        Ok(())
    }
    
    /// Validate deck name according to AnkiDroid conventions
    pub fn validate_deck_name(deck_name: &str) -> AndroidResult<()> {
        let trimmed = deck_name.trim();
        
        if trimmed.is_empty() {
            return Err(AndroidError::validation_error("Deck name cannot be empty"));
        }
        
        if trimmed.len() > 100 {
            return Err(AndroidError::validation_error("Deck name cannot exceed 100 characters"));
        }
        
        // Check for invalid characters that could cause issues with AnkiDroid
        const INVALID_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for &invalid_char in INVALID_CHARS {
            if trimmed.contains(invalid_char) {
                return Err(AndroidError::validation_error(
                    format!("Deck name cannot contain '{}' character", invalid_char)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate model ID ensuring it's a valid AnkiDroid model identifier
    pub fn validate_model_id(model_id: i64) -> AndroidResult<()> {
        if model_id <= 0 {
            return Err(AndroidError::validation_error("Model ID must be positive"));
        }
        
        // AnkiDroid model IDs are typically Unix timestamps (13 digits)
        // But we'll be more permissive and just ensure it's reasonable
        if model_id > 9_999_999_999_999i64 {
            return Err(AndroidError::validation_error("Model ID appears to be invalid"));
        }
        
        Ok(())
    }
    
    /// Sanitize JNI string input to prevent issues with JNI string conversion
    pub fn sanitize_jni_string(input: &str) -> AndroidResult<String> {
        // Remove null bytes that could cause issues with JNI string conversion
        let sanitized = input.replace('\0', "");
        
        // Ensure the string is valid UTF-8 and doesn't contain problematic sequences
        if sanitized.len() != input.len() {
            return Err(AndroidError::validation_error(
                "Input string contains null bytes which are not allowed"
            ));
        }
        
        // Check for extremely long strings that could cause memory issues
        const MAX_JNI_STRING_LENGTH: usize = 1_048_576; // 1MB limit
        if sanitized.len() > MAX_JNI_STRING_LENGTH {
            return Err(AndroidError::validation_error(
                format!("String exceeds maximum JNI string length of {} characters", MAX_JNI_STRING_LENGTH)
            ));
        }
        
        Ok(sanitized)
    }
    
    /// Validate deck ID ensuring it's a valid AnkiDroid deck identifier
    pub fn validate_deck_id(deck_id: i64) -> AndroidResult<()> {
        if deck_id <= 0 {
            return Err(AndroidError::validation_error("Deck ID must be positive"));
        }
        
        // AnkiDroid deck IDs are typically Unix timestamps or 1 for default deck
        if deck_id != 1 && deck_id < 1_000_000_000i64 {
            return Err(AndroidError::validation_error("Deck ID appears to be invalid"));
        }
        
        Ok(())
    }
    
    /// Validate note ID ensuring it's a valid AnkiDroid note identifier  
    pub fn validate_note_id(note_id: i64) -> AndroidResult<()> {
        if note_id <= 0 {
            return Err(AndroidError::validation_error("Note ID must be positive"));
        }
        
        Ok(())
    }
    
    /// Validate card ID ensuring it's a valid AnkiDroid card identifier
    pub fn validate_card_id(card_id: i64) -> AndroidResult<()> {
        if card_id <= 0 {
            return Err(AndroidError::validation_error("Card ID must be positive"));
        }
        
        Ok(())
    }
    
    /// Validate selection clause for SQL queries to prevent injection
    pub fn validate_selection_clause(selection: &str) -> AndroidResult<()> {
        let trimmed = selection.trim();
        
        if trimmed.is_empty() {
            return Ok(());
        }
        
        // Basic SQL injection prevention - check for dangerous patterns
        let lowercase = trimmed.to_lowercase();
        const DANGEROUS_PATTERNS: &[&str] = &[
            "--", "/*", "*/", ";", "drop", "delete", "insert", "update", 
            "exec", "execute", "union", "alter", "create", "truncate"
        ];
        
        for &pattern in DANGEROUS_PATTERNS {
            if lowercase.contains(pattern) {
                return Err(AndroidError::validation_error(
                    format!("Selection clause contains potentially dangerous pattern: '{}'", pattern)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate projection columns for queries
    pub fn validate_projection(projection: &[String]) -> AndroidResult<()> {
        if projection.is_empty() {
            return Ok(());
        }
        
        for column in projection {
            let trimmed = column.trim();
            if trimmed.is_empty() {
                return Err(AndroidError::validation_error("Projection column cannot be empty"));
            }
            
            // Validate column name format (basic alphanumeric with underscores)
            if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(AndroidError::validation_error(
                    format!("Invalid column name: '{}'. Only alphanumeric characters and underscores allowed", trimmed)
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_card_content_valid() {
        let result = ValidationHelper::validate_card_content("Front text", "Back text");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_card_content_empty_front() {
        let result = ValidationHelper::validate_card_content("", "Back text");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("front cannot be empty"));
    }

    #[test]
    fn test_validate_card_content_empty_back() {
        let result = ValidationHelper::validate_card_content("Front text", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("back cannot be empty"));
    }

    #[test]
    fn test_validate_deck_name_valid() {
        let result = ValidationHelper::validate_deck_name("My Study Deck");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_deck_name_empty() {
        let result = ValidationHelper::validate_deck_name("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_deck_name_invalid_chars() {
        let result = ValidationHelper::validate_deck_name("My/Deck");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot contain"));
    }

    #[test]
    fn test_validate_model_id_valid() {
        let result = ValidationHelper::validate_model_id(1609459200000i64); // Valid timestamp
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_model_id_invalid() {
        let result = ValidationHelper::validate_model_id(-1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be positive"));
    }

    #[test]
    fn test_sanitize_jni_string_valid() {
        let result = ValidationHelper::sanitize_jni_string("Hello World");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");
    }

    #[test]
    fn test_sanitize_jni_string_with_null() {
        let result = ValidationHelper::sanitize_jni_string("Hello\0World");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null bytes"));
    }

    #[test]
    fn test_validate_selection_clause_valid() {
        let result = ValidationHelper::validate_selection_clause("name = ? AND type = ?");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_selection_clause_dangerous() {
        let result = ValidationHelper::validate_selection_clause("name = 'test'; DROP TABLE users;");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dangerous pattern"));
    }

    #[test]
    fn test_validate_projection_valid() {
        let projection = vec!["_id".to_string(), "name".to_string(), "type".to_string()];
        let result = ValidationHelper::validate_projection(&projection);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_projection_invalid_column() {
        let projection = vec!["_id".to_string(), "name;DROP".to_string()];
        let result = ValidationHelper::validate_projection(&projection);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid column name"));
    }
}
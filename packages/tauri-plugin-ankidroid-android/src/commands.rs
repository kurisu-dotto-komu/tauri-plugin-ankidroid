use crate::mobile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct HelloResponse {
    pub value: String,
}

// Using correct terminology - Notes, not Cards
#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub front: String,
    pub back: String,
    pub deck: Option<String>,
    pub tags: Option<String>,
}

// Legacy alias for backward compatibility
pub type CreateCardRequest = CreateNoteRequest;

// Response uses note_id as it correctly represents a Note ID
#[derive(Debug, Serialize)]
pub struct CreateNoteResponse {
    pub success: bool,
    pub note_id: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

// Legacy alias for backward compatibility
pub type CreateCardResponse = CreateNoteResponse;

#[tauri::command]
pub async fn hello(name: String) -> Result<String, String> {
    log::info!("Hello command called with name: {}", name);
    mobile::hello(name).await
}

// NEW: Correct terminology - list_notes
#[tauri::command]
pub async fn list_notes() -> Result<String, String> {
    log::info!("List notes command called");
    mobile::list_notes().await
}

// LEGACY: Backward compatibility wrapper - redirects to list_notes
#[tauri::command]
pub async fn list_cards() -> Result<String, String> {
    log::info!("List cards command called (legacy) - redirecting to list_notes");
    mobile::list_notes().await
}

// NEW: Correct terminology - create_note
#[tauri::command]
pub async fn create_note(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Create note command called - front: {}, back: {}",
        front,
        back
    );
    mobile::create_note(front, back, deck, tags).await
}

// LEGACY: Backward compatibility wrapper - redirects to create_note
#[tauri::command]
pub async fn create_card(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Create card command called (legacy) - redirecting to create_note"
    );
    mobile::create_note(front, back, deck, tags).await
}

#[tauri::command]
pub async fn get_decks() -> Result<String, String> {
    log::info!("Get decks command called");
    mobile::get_decks().await
}

// NEW: Correct terminology - update_note
#[tauri::command]
pub async fn update_note(
    note_id: i64,
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Update note command called - note_id: {}, front: {}, back: {}",
        note_id,
        front,
        back
    );
    mobile::update_note(note_id, front, back, deck, tags).await
}

// LEGACY: Backward compatibility wrapper - redirects to update_note
#[tauri::command]
pub async fn update_card(
    note_id: i64,
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Update card command called (legacy) - redirecting to update_note"
    );
    mobile::update_note(note_id, front, back, deck, tags).await
}

// NEW: Correct terminology - delete_note
#[tauri::command]
pub async fn delete_note(note_id: i64) -> Result<String, String> {
    log::info!("Delete note command called - note_id: {}", note_id);
    mobile::delete_note(note_id).await
}

// LEGACY: Backward compatibility wrapper - redirects to delete_note
#[tauri::command]
pub async fn delete_card(note_id: i64) -> Result<String, String> {
    log::info!("Delete card command called (legacy) - redirecting to delete_note");
    mobile::delete_note(note_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_request_deserialize() {
        let json = r#"{"name": "Test"}"#;
        let request: HelloRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Test");
    }

    #[test]
    fn test_hello_response_serialize() {
        let response = HelloResponse {
            value: "Hello, World!".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Hello, World!"));
    }

    #[test]
    fn test_create_note_request_deserialize() {
        let json =
            r#"{"front": "Question", "back": "Answer", "deck": "Test Deck", "tags": "test"}"#;
        let request: CreateNoteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.front, "Question");
        assert_eq!(request.back, "Answer");
        assert_eq!(request.deck, Some("Test Deck".to_string()));
        assert_eq!(request.tags, Some("test".to_string()));
    }

    #[test]
    fn test_legacy_create_card_request_deserialize() {
        // Test that legacy type alias still works
        let json =
            r#"{"front": "Question", "back": "Answer", "deck": "Test Deck", "tags": "test"}"#;
        let request: CreateCardRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.front, "Question");
        assert_eq!(request.back, "Answer");
        assert_eq!(request.deck, Some("Test Deck".to_string()));
        assert_eq!(request.tags, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_hello_command_integration() {
        let result = hello("Integration Test".to_string()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Integration Test"));
    }

    #[tokio::test]
    async fn test_list_notes_command_integration() {
        let result = list_notes().await;
        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "list_notes should return valid JSON");

        // Should be an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "list_notes should return a JSON array");
    }

    #[tokio::test]
    async fn test_list_cards_legacy_command_integration() {
        // Test that legacy command still works
        let result = list_cards().await;
        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "list_cards (legacy) should return valid JSON");

        // Should be an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "list_cards (legacy) should return a JSON array");
    }

    #[tokio::test]
    async fn test_create_note_command_integration() {
        let result = create_note(
            "Test Question".to_string(),
            "Test Answer".to_string(),
            Some("Test Deck".to_string()),
            Some("test".to_string()),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "create_note should return valid JSON");
    }

    #[tokio::test]
    async fn test_create_card_legacy_command_integration() {
        // Test that legacy command still works
        let result = create_card(
            "Test Question".to_string(),
            "Test Answer".to_string(),
            Some("Test Deck".to_string()),
            Some("test".to_string()),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "create_card (legacy) should return valid JSON");
    }

    #[tokio::test]
    async fn test_get_decks_command_integration() {
        let result = get_decks().await;
        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "get_decks should return valid JSON");

        // Should be an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "get_decks should return a JSON array");
    }

    #[tokio::test]
    async fn test_commands_dont_panic() {
        // Test that commands handle edge cases without panicking
        let test_cases = vec!["", "test", "special!@#$%^&*()chars"];
        let long_string = "very_long_string_".repeat(100);

        for test_case in test_cases {
            let result = hello(test_case.to_string()).await;
            assert!(
                result.is_ok(),
                "hello command should not panic with input: {}",
                test_case
            );
        }

        // Test with long string
        let result = hello(long_string.clone()).await;
        assert!(result.is_ok(), "hello command should handle long strings");

        // Test create_note with edge cases
        let result = create_note(long_string.clone(), long_string.clone(), None, None).await;
        assert!(
            result.is_ok(),
            "create_note command should handle long strings"
        );

        // Test legacy create_card with edge cases
        let result = create_card(long_string.clone(), long_string.clone(), None, None).await;
        assert!(
            result.is_ok(),
            "create_card (legacy) command should handle long strings"
        );

        // Test list_notes doesn't panic
        let result = list_notes().await;
        assert!(result.is_ok(), "list_notes command should not panic");

        // Test legacy list_cards doesn't panic
        let result = list_cards().await;
        assert!(result.is_ok(), "list_cards (legacy) command should not panic");

        // Test get_decks doesn't panic
        let result = get_decks().await;
        assert!(result.is_ok(), "get_decks command should not panic");
    }

    #[tokio::test]
    async fn test_update_note_command_integration() {
        let result = update_note(
            1,
            "Updated Front".to_string(),
            "Updated Back".to_string(),
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "update_note should return valid JSON");
    }

    #[tokio::test]
    async fn test_delete_note_command_integration() {
        let result = delete_note(1).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "delete_note should return valid JSON");
    }

    #[tokio::test]
    async fn test_backward_compatibility() {
        // Test that all legacy commands still work
        let result = create_card("Front".to_string(), "Back".to_string(), None, None).await;
        assert!(result.is_ok(), "create_card (legacy) should work");

        let result = list_cards().await;
        assert!(result.is_ok(), "list_cards (legacy) should work");

        let result = update_card(1, "Front".to_string(), "Back".to_string(), None, None).await;
        assert!(result.is_ok(), "update_card (legacy) should work");

        let result = delete_card(1).await;
        assert!(result.is_ok(), "delete_card (legacy) should work");
    }
}
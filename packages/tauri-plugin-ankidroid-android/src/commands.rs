use serde::{Deserialize, Serialize};

#[cfg(desktop)]
use crate::desktop;
#[cfg(mobile)]
use crate::mobile;

#[derive(Debug, Deserialize)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct HelloResponse {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCardRequest {
    pub front: String,
    pub back: String,
    pub deck: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateCardResponse {
    pub success: bool,
    pub note_id: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn hello(name: String) -> Result<String, String> {
    log::info!("Hello command called with name: {}", name);

    #[cfg(mobile)]
    {
        mobile::hello(name).await
    }
    #[cfg(desktop)]
    {
        desktop::hello(name).await
    }
}

#[tauri::command]
pub async fn list_cards() -> Result<String, String> {
    log::info!("List cards command called");

    #[cfg(mobile)]
    {
        mobile::list_cards().await
    }
    #[cfg(desktop)]
    {
        desktop::list_cards().await
    }
}

#[tauri::command]
pub async fn create_card(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::error!(
        "🟡 COMMAND: create_card invoked - front: {}, back: {}",
        front,
        back
    );

    #[cfg(desktop)]
    {
        desktop::create_card(front, back, deck, tags).await
    }
    #[cfg(mobile)]
    {
        log::error!("🟡 COMMAND: Calling mobile::create_card");
        let result = mobile::create_card(front, back, deck, tags).await;
        log::error!("🟡 COMMAND: mobile::create_card returned: {:?}", result);
        result
    }
}

#[tauri::command]
pub async fn get_decks() -> Result<String, String> {
    log::info!("Get decks command called");

    #[cfg(mobile)]
    {
        mobile::get_decks().await
    }
    #[cfg(desktop)]
    {
        desktop::get_decks().await
    }
}

#[tauri::command]
pub async fn update_card(
    note_id: i64,
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    log::info!(
        "Update card command called - note_id: {}, front: {}, back: {}",
        note_id,
        front,
        back
    );

    #[cfg(mobile)]
    {
        mobile::update_card(note_id, front, back, deck, tags).await
    }
    #[cfg(desktop)]
    {
        desktop::update_card(note_id, front, back, deck, tags).await
    }
}

#[tauri::command]
pub async fn delete_card(note_id: i64) -> Result<String, String> {
    log::info!("Delete card command called - note_id: {}", note_id);

    #[cfg(mobile)]
    {
        mobile::delete_card(note_id).await
    }
    #[cfg(desktop)]
    {
        desktop::delete_card(note_id).await
    }
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
    fn test_create_card_request_deserialize() {
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
    async fn test_list_cards_command_integration() {
        let result = list_cards().await;
        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "list_cards should return valid JSON");

        // Should be an array
        let json = parsed.unwrap();
        assert!(json.is_array(), "list_cards should return a JSON array");
    }

    #[tokio::test]
    async fn test_create_card_command_integration() {
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
        assert!(parsed.is_ok(), "create_card should return valid JSON");
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

        // Test create_card with edge cases
        let result = create_card(long_string.clone(), long_string.clone(), None, None).await;
        assert!(
            result.is_ok(),
            "create_card command should handle long strings"
        );

        // Test list_cards doesn't panic
        let result = list_cards().await;
        assert!(result.is_ok(), "list_cards command should not panic");

        // Test get_decks doesn't panic
        let result = get_decks().await;
        assert!(result.is_ok(), "get_decks command should not panic");
    }
}

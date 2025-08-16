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
    async fn test_commands_dont_panic() {
        // Test that commands handle edge cases without panicking
        let test_cases = vec!["", "test", "special!@#$%^&*()chars"];
        let long_string = "very_long_string_".repeat(100);
        
        for test_case in test_cases {
            let result = hello(test_case.to_string()).await;
            assert!(result.is_ok(), "hello command should not panic with input: {}", test_case);
        }
        
        // Test with long string
        let result = hello(long_string).await;
        assert!(result.is_ok(), "hello command should handle long strings");
        
        // Test list_cards doesn't panic
        let result = list_cards().await;
        assert!(result.is_ok(), "list_cards command should not panic");
    }
}

use serde::{Deserialize, Serialize};

/// Represents a flashcard in AnkiDroid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: i64,
    pub front: String,
    pub back: String,
    pub deck: String,
    pub tags: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deck_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<i64>,
}

impl Card {
    /// Create a new card
    pub fn new(id: i64, front: String, back: String, deck: String, tags: String) -> Self {
        Self {
            id,
            front,
            back,
            deck,
            tags,
            deck_id: None,
            model_id: None,
            note_id: None,
        }
    }

    /// Create a card with additional metadata
    pub fn with_metadata(
        id: i64,
        front: String,
        back: String,
        deck: String,
        tags: String,
        deck_id: Option<i64>,
        model_id: Option<i64>,
        note_id: Option<i64>,
    ) -> Self {
        Self {
            id,
            front,
            back,
            deck,
            tags,
            deck_id,
            model_id,
            note_id,
        }
    }

    /// Check if the card has valid content
    pub fn is_valid(&self) -> bool {
        !self.front.trim().is_empty() && !self.back.trim().is_empty()
    }

    /// Get the card's tags as a vector
    pub fn get_tags(&self) -> Vec<String> {
        self.tags
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Set the card's tags from a vector
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags.join(" ");
    }

    /// Add a tag to the card
    pub fn add_tag(&mut self, tag: String) {
        let mut tags = self.get_tags();
        if !tags.contains(&tag) {
            tags.push(tag);
            self.set_tags(tags);
        }
    }

    /// Remove a tag from the card
    pub fn remove_tag(&mut self, tag: &str) {
        let tags: Vec<String> = self.get_tags().into_iter().filter(|t| t != tag).collect();
        self.set_tags(tags);
    }
}

/// Represents a deck in AnkiDroid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_count: Option<i32>,
}

impl Deck {
    /// Create a new deck
    pub fn new(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            card_count: None,
            new_count: None,
            learning_count: None,
            review_count: None,
        }
    }

    /// Create a deck with statistics
    pub fn with_stats(
        id: i64,
        name: String,
        card_count: i32,
        new_count: i32,
        learning_count: i32,
        review_count: i32,
    ) -> Self {
        Self {
            id,
            name,
            description: None,
            card_count: Some(card_count),
            new_count: Some(new_count),
            learning_count: Some(learning_count),
            review_count: Some(review_count),
        }
    }

    /// Get total card count
    pub fn total_cards(&self) -> i32 {
        self.card_count.unwrap_or(0)
    }

    /// Check if the deck is empty
    pub fn is_empty(&self) -> bool {
        self.total_cards() == 0
    }
}

/// Represents a note model in AnkiDroid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub field_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_type: Option<i32>, // 0 = standard, 1 = cloze
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
}

impl Model {
    /// Create a new model
    pub fn new(id: i64, name: String, field_count: i32) -> Self {
        Self {
            id,
            name,
            field_count,
            model_type: None,
            css: None,
        }
    }

    /// Create a model with type information
    pub fn with_type(id: i64, name: String, field_count: i32, model_type: i32) -> Self {
        Self {
            id,
            name,
            field_count,
            model_type: Some(model_type),
            css: None,
        }
    }

    /// Check if this is a basic model (2 fields)
    pub fn is_basic(&self) -> bool {
        self.field_count == 2
    }

    /// Check if this is a cloze model
    pub fn is_cloze(&self) -> bool {
        self.model_type == Some(1)
    }

    /// Check if this is a standard model
    pub fn is_standard(&self) -> bool {
        self.model_type == Some(0) || self.model_type.is_none()
    }
}

/// Response structure for card creation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CreateCardResponse {
    /// Create a successful response
    pub fn success(note_id: i64, message: Option<String>) -> Self {
        Self {
            success: true,
            note_id: Some(note_id),
            message,
            error: None,
        }
    }

    /// Create an error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            note_id: None,
            message: None,
            error: Some(error),
        }
    }

    /// Create a simple success response
    pub fn simple_success(note_id: i64) -> Self {
        Self::success(note_id, Some("Card created successfully".to_string()))
    }
}

/// Request structure for card creation
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCardRequest {
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub deck: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
}

impl CreateCardRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), String> {
        if self.front.trim().is_empty() {
            return Err("Front field cannot be empty".to_string());
        }

        if self.back.trim().is_empty() {
            return Err("Back field cannot be empty".to_string());
        }

        if self.front.len() > 65536 {
            return Err("Front field too long (max 65536 characters)".to_string());
        }

        if self.back.len() > 65536 {
            return Err("Back field too long (max 65536 characters)".to_string());
        }

        Ok(())
    }

    /// Get the deck name or default
    pub fn deck_name(&self) -> &str {
        self.deck.as_deref().unwrap_or("Default")
    }

    /// Get the tags or empty string
    pub fn tags_string(&self) -> &str {
        self.tags.as_deref().unwrap_or("")
    }
}

/// Request structure for card updates
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCardRequest {
    pub note_id: i64,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub deck: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
}

impl UpdateCardRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), String> {
        if self.note_id <= 0 {
            return Err("Invalid note ID".to_string());
        }

        if self.front.trim().is_empty() {
            return Err("Front field cannot be empty".to_string());
        }

        if self.back.trim().is_empty() {
            return Err("Back field cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Response structure for general operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl OperationResponse {
    /// Create a successful response
    pub fn success(message: Option<String>) -> Self {
        Self {
            success: true,
            message,
            error: None,
            data: None,
        }
    }

    /// Create a successful response with data
    pub fn success_with_data(message: Option<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message,
            error: None,
            data: Some(data),
        }
    }

    /// Create an error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            message: None,
            error: Some(error),
            data: None,
        }
    }
}

/// Configuration for the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(default = "default_batch_size")]
    pub batch_size: i32,
    #[serde(default = "default_timeout")]
    pub timeout_ms: i32,
    #[serde(default = "default_max_field_length")]
    pub max_field_length: usize,
    #[serde(default)]
    pub enable_logging: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            batch_size: default_batch_size(),
            timeout_ms: default_timeout(),
            max_field_length: default_max_field_length(),
            enable_logging: true,
        }
    }
}

fn default_batch_size() -> i32 {
    100
}
fn default_timeout() -> i32 {
    30000
}
fn default_max_field_length() -> usize {
    65536
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_creation() {
        let card = Card::new(
            1,
            "Front".to_string(),
            "Back".to_string(),
            "Deck".to_string(),
            "tag1 tag2".to_string(),
        );

        assert_eq!(card.id, 1);
        assert_eq!(card.front, "Front");
        assert_eq!(card.back, "Back");
        assert_eq!(card.deck, "Deck");
        assert!(card.is_valid());
    }

    #[test]
    fn test_card_tags() {
        let mut card = Card::new(
            1,
            "Front".to_string(),
            "Back".to_string(),
            "Deck".to_string(),
            "tag1 tag2".to_string(),
        );

        let tags = card.get_tags();
        assert_eq!(tags, vec!["tag1".to_string(), "tag2".to_string()]);

        card.add_tag("tag3".to_string());
        assert!(card.tags.contains("tag3"));

        card.remove_tag("tag1");
        assert!(!card.tags.contains("tag1"));
    }

    #[test]
    fn test_create_card_request_validation() {
        let valid_request = CreateCardRequest {
            front: "Question".to_string(),
            back: "Answer".to_string(),
            deck: Some("Test".to_string()),
            tags: Some("test".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateCardRequest {
            front: "".to_string(),
            back: "Answer".to_string(),
            deck: None,
            tags: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_deck_creation() {
        let deck = Deck::new(1, "Test Deck".to_string());
        assert_eq!(deck.id, 1);
        assert_eq!(deck.name, "Test Deck");
        assert!(deck.is_empty());

        let deck_with_stats = Deck::with_stats(2, "Stats Deck".to_string(), 10, 5, 3, 2);
        assert_eq!(deck_with_stats.total_cards(), 10);
        assert!(!deck_with_stats.is_empty());
    }

    #[test]
    fn test_model_types() {
        let basic_model = Model::new(1, "Basic".to_string(), 2);
        assert!(basic_model.is_basic());
        assert!(basic_model.is_standard());

        let cloze_model = Model::with_type(2, "Cloze".to_string(), 3, 1);
        assert!(!cloze_model.is_basic());
        assert!(cloze_model.is_cloze());
        assert!(!cloze_model.is_standard());
    }

    #[test]
    fn test_response_creation() {
        let success = CreateCardResponse::simple_success(123);
        assert!(success.success);
        assert_eq!(success.note_id, Some(123));

        let error = CreateCardResponse::error("Test error".to_string());
        assert!(!error.success);
        assert_eq!(error.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_serialization() {
        let card = Card::new(
            1,
            "Front".to_string(),
            "Back".to_string(),
            "Deck".to_string(),
            "tags".to_string(),
        );

        let serialized = serde_json::to_string(&card).unwrap();
        let deserialized: Card = serde_json::from_str(&serialized).unwrap();

        assert_eq!(card.id, deserialized.id);
        assert_eq!(card.front, deserialized.front);
        assert_eq!(card.back, deserialized.back);
    }

    #[test]
    fn test_plugin_config() {
        let config = PluginConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_field_length, 65536);
        assert!(config.enable_logging);
    }
}

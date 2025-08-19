use tauri::{AppHandle, Runtime};

pub fn init<R: Runtime>(
    _app: &AppHandle<R>,
    _api: tauri::plugin::PluginApi<R, ()>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub async fn hello(name: String) -> Result<String, String> {
    Ok(format!("Hello, {} from AnkiDroid plugin!", name))
}

pub async fn create_card(
    front: String,
    back: String,
    deck: Option<String>,
    tags: Option<String>,
) -> Result<String, String> {
    // Mock implementation for desktop
    Ok(format!(
        r#"{{
    "success": true,
    "noteId": {},
    "message": "Card created (desktop mock): {} / {}",
    "error": null
}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        front.replace('"', r#"\""#),
        back.replace('"', r#"\""#)
    ))
}

pub async fn get_decks() -> Result<String, String> {
    // Mock deck data for desktop
    Ok(r#"[
    {"id": 1, "name": "Default"},
    {"id": 2, "name": "Geography"},
    {"id": 3, "name": "Math"},
    {"id": 4, "name": "Literature"},
    {"id": 5, "name": "Chemistry"},
    {"id": 6, "name": "Astronomy"}
]"#
    .to_string())
}

pub async fn list_cards() -> Result<String, String> {
    // Mock data for desktop testing
    Ok(r#"[
  {
    "id": 1,
    "front": "What is the capital of France?",
    "back": "Paris",
    "deck": "Geography",
    "tags": ""
  },
  {
    "id": 2,
    "front": "What is 2 + 2?",
    "back": "4",
    "deck": "Math",
    "tags": "basic"
  },
  {
    "id": 3,
    "front": "Who wrote Romeo and Juliet?",
    "back": "William Shakespeare",
    "deck": "Literature",
    "tags": "author"
  },
  {
    "id": 4,
    "front": "What is the chemical symbol for gold?",
    "back": "Au",
    "deck": "Chemistry",
    "tags": "element"
  },
  {
    "id": 5,
    "front": "What is the largest planet in our solar system?",
    "back": "Jupiter",
    "deck": "Astronomy",
    "tags": "planet"
  }
]"#
    .to_string())
}

pub async fn update_card(
    note_id: i64,
    front: String,
    back: String,
    _deck: Option<String>,
    _tags: Option<String>,
) -> Result<String, String> {
    Ok(format!(
        r#"{{
    "success": true,
    "noteId": {},
    "message": "Card updated successfully (desktop mock): {} / {}",
    "error": null
}}"#,
        note_id,
        front.replace('"', r#"\""#),
        back.replace('"', r#"\""#)
    ))
}

pub async fn delete_card(note_id: i64) -> Result<String, String> {
    Ok(format!(
        r#"{{
    "success": true,
    "noteId": {},
    "message": "Card deleted successfully (desktop mock)",
    "error": null
}}"#,
        note_id
    ))
}

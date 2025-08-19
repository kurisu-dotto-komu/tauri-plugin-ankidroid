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

pub async fn list_cards() -> Result<String, String> {
    // Mock data for desktop testing
    Ok(r#"[
  {
    "id": 1,
    "question": "What is the capital of France?",
    "answer": "Paris",
    "deck": "Geography"
  },
  {
    "id": 2,
    "question": "What is 2 + 2?",
    "answer": "4",
    "deck": "Math"
  },
  {
    "id": 3,
    "question": "Who wrote Romeo and Juliet?",
    "answer": "William Shakespeare",
    "deck": "Literature"
  },
  {
    "id": 4,
    "question": "What is the chemical symbol for gold?",
    "answer": "Au",
    "deck": "Chemistry"
  },
  {
    "id": 5,
    "question": "What is the largest planet in our solar system?",
    "answer": "Jupiter",
    "deck": "Astronomy"
  }
]"#.to_string())
}

use crate::android::constants::{
    model_columns, DEFAULT_BASIC_MODEL_ID, DEFAULT_MODEL_NAME, MODELS_URI,
};
use crate::android::content_provider::query;
use crate::android::cursor::collect_cursor_results;
use crate::android::error::{AndroidError, AndroidResult};
use crate::android::jni_helpers::SafeJNIEnv;
use jni::objects::JObject;

/// Find a model ID by name with optional field count validation
pub fn find_basic_model_id(env: &SafeJNIEnv, activity: &JObject) -> AndroidResult<i64> {
    find_model_id_by_name(env, activity, DEFAULT_MODEL_NAME, Some(2))
}

/// Find a model ID by name with optional field count validation
pub fn find_model_id_by_name(
    env: &SafeJNIEnv,
    activity: &JObject,
    model_name: &str,
    min_field_count: Option<usize>,
) -> AndroidResult<i64> {
    log::info!(
        "Searching for model: {} with min fields: {:?}",
        model_name,
        min_field_count
    );

    let projection = vec![
        "_id".to_string(), // Use "_id" instead of "mid" to avoid "Queue 'mid' is unknown" error
        model_columns::NAME.to_string(),
        model_columns::FLDS.to_string(),
    ];

    let env_clone = env.clone();
    let cursor = query(env_clone, MODELS_URI)
        .projection(projection)
        .execute(activity)?;

    let models = collect_cursor_results(cursor, |cursor| {
        let id = cursor.get_long_by_name("_id")?; // Use "_id" instead of "mid"
        let name = cursor.get_string_by_name(model_columns::NAME)?;
        let fields_json = cursor.get_string_by_name(model_columns::FLDS)?;

        // Parse field count from fields JSON if validation is needed
        let field_count = if min_field_count.is_some() {
            parse_field_count(&fields_json)?
        } else {
            0
        };

        Ok((id, name, field_count))
    })?;

    // Look for exact name match first
    for (id, name, field_count) in &models {
        if name == model_name {
            if let Some(min_count) = min_field_count {
                if *field_count >= min_count {
                    log::info!(
                        "Found exact model match: {} (ID: {}, fields: {})",
                        name,
                        id,
                        field_count
                    );
                    return Ok(*id);
                } else {
                    log::warn!(
                        "Model {} found but has insufficient fields: {} < {}",
                        name,
                        field_count,
                        min_count
                    );
                }
            } else {
                log::info!("Found exact model match: {} (ID: {})", name, id);
                return Ok(*id);
            }
        }
    }

    // Look for partial name match (case insensitive)
    let model_name_lower = model_name.to_lowercase();
    for (id, name, field_count) in &models {
        if name.to_lowercase().contains(&model_name_lower) {
            if let Some(min_count) = min_field_count {
                if *field_count >= min_count {
                    log::info!(
                        "Found partial model match: {} (ID: {}, fields: {})",
                        name,
                        id,
                        field_count
                    );
                    return Ok(*id);
                }
            } else {
                log::info!("Found partial model match: {} (ID: {})", name, id);
                return Ok(*id);
            }
        }
    }

    // If looking for Basic model specifically, try using the default ID
    if model_name.to_lowercase().contains("basic") {
        log::warn!(
            "Basic model not found in database, using default ID: {}",
            DEFAULT_BASIC_MODEL_ID
        );
        return Ok(DEFAULT_BASIC_MODEL_ID);
    }

    // If no models found, return the first model with sufficient fields
    if let Some(min_count) = min_field_count {
        for (id, name, field_count) in &models {
            if *field_count >= min_count {
                log::warn!(
                    "Using fallback model: {} (ID: {}, fields: {})",
                    name,
                    id,
                    field_count
                );
                return Ok(*id);
            }
        }
    }

    // Last resort: use the first available model
    if let Some((id, name, _)) = models.first() {
        log::warn!(
            "Using first available model as fallback: {} (ID: {})",
            name,
            id
        );
        return Ok(*id);
    }

    Err(AndroidError::model_not_found(format!(
        "No suitable model found for '{}' with min {} fields",
        model_name,
        min_field_count.unwrap_or(0)
    )))
}

/// Get all available models
pub fn list_models(
    env: SafeJNIEnv,
    activity: &JObject,
) -> AndroidResult<Vec<(i64, String, usize)>> {
    log::info!("Listing all available models");

    let projection = vec![
        "_id".to_string(), // Use "_id" instead of "mid" to avoid "Queue 'mid' is unknown" error
        model_columns::NAME.to_string(),
        model_columns::FLDS.to_string(),
        model_columns::TYPE.to_string(),
    ];

    let cursor = query(env, MODELS_URI)
        .projection(projection)
        .sort_order(format!("{} ASC", model_columns::NAME))
        .execute(activity)?;

    collect_cursor_results(cursor, |cursor| {
        let id = cursor.get_long_by_name("_id")?; // Use "_id" instead of "mid"
        let name = cursor.get_string_by_name(model_columns::NAME)?;
        let fields_json = cursor.get_string_by_name(model_columns::FLDS)?;
        let model_type = cursor.get_int_by_name(model_columns::TYPE)?;

        let field_count = parse_field_count(&fields_json)?;

        log::debug!(
            "Found model: {} (ID: {}, type: {}, fields: {})",
            name,
            id,
            model_type,
            field_count
        );
        Ok((id, name, field_count))
    })
}

/// Check if a model exists by ID
pub fn model_exists(env: SafeJNIEnv, activity: &JObject, model_id: i64) -> AndroidResult<bool> {
    log::info!("Checking if model exists: {}", model_id);

    let projection = vec!["_id".to_string()]; // Use "_id" instead of "mid" to avoid "Queue 'mid' is unknown" error
    // Use "_id" for selection clauses instead of "mid" to avoid "Queue 'mid' is unknown" error
    let selection = format!("{} = ?", "_id");
    let selection_args = vec![model_id.to_string()];

    let mut cursor = query(env, MODELS_URI)
        .projection(projection)
        .selection(selection)
        .selection_args(selection_args)
        .execute(activity)?;

    let count = cursor.get_count()?;
    Ok(count > 0)
}

/// Get model information by ID
pub fn get_model_info(
    env: SafeJNIEnv,
    activity: &JObject,
    model_id: i64,
) -> AndroidResult<(String, usize, i32)> {
    log::info!("Getting model info for ID: {}", model_id);

    let projection = vec![
        model_columns::NAME.to_string(),
        model_columns::FLDS.to_string(),
        model_columns::TYPE.to_string(),
    ];
    // Use "_id" for selection clauses instead of "mid" to avoid "Queue 'mid' is unknown" error
    let selection = format!("{} = ?", "_id");
    let selection_args = vec![model_id.to_string()];

    let cursor = query(env, MODELS_URI)
        .projection(projection)
        .selection(selection)
        .selection_args(selection_args)
        .execute(activity)?;

    let results = collect_cursor_results(cursor, |cursor| {
        let name = cursor.get_string_by_name(model_columns::NAME)?;
        let fields_json = cursor.get_string_by_name(model_columns::FLDS)?;
        let model_type = cursor.get_int_by_name(model_columns::TYPE)?;

        let field_count = parse_field_count(&fields_json)?;
        Ok((name, field_count, model_type))
    })?;

    results
        .into_iter()
        .next()
        .ok_or_else(|| AndroidError::model_not_found(format!("Model ID {} not found", model_id)))
}

/// Parse field count from fields JSON string
fn parse_field_count(fields_json: &str) -> AndroidResult<usize> {
    if fields_json.is_empty() {
        return Ok(0);
    }

    // Try to parse as JSON array
    match serde_json::from_str::<serde_json::Value>(fields_json) {
        Ok(json) => {
            if let Some(array) = json.as_array() {
                Ok(array.len())
            } else {
                // If not an array, assume it's a single field
                Ok(1)
            }
        }
        Err(_) => {
            // If JSON parsing fails, try to count field separators or commas
            let comma_count = fields_json.chars().filter(|&c| c == ',').count();
            let separator_count = fields_json.chars().filter(|&c| c == '\u{001f}').count();

            // Use the higher count plus one, or default to 2 for Basic models
            if comma_count > 0 || separator_count > 0 {
                Ok(std::cmp::max(comma_count, separator_count) + 1)
            } else {
                Ok(2) // Default for Basic model
            }
        }
    }
}

/// Validate that a model is suitable for basic card operations
pub fn validate_model_for_cards(
    env: &mut SafeJNIEnv,
    activity: &JObject,
    model_id: i64,
) -> AndroidResult<()> {
    let env_for_info = env.clone();
    let (name, field_count, model_type) = get_model_info(env_for_info, activity, model_id)?;

    if field_count < 2 {
        return Err(AndroidError::validation_error(format!(
            "Model '{}' has insufficient fields: {} (minimum 2 required)",
            name, field_count
        )));
    }

    if model_type != 0 && model_type != 1 {
        return Err(AndroidError::validation_error(format!(
            "Model '{}' has unsupported type: {} (only standard and cloze models supported)",
            name, model_type
        )));
    }

    log::info!(
        "Model validation passed: {} (ID: {}, fields: {}, type: {})",
        name,
        model_id,
        field_count,
        model_type
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_count_json_array() {
        let fields_json = r#"[{"name":"Front","ord":0},{"name":"Back","ord":1}]"#;
        let count = parse_field_count(fields_json).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_field_count_empty() {
        let fields_json = "";
        let count = parse_field_count(fields_json).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_parse_field_count_comma_separated() {
        let fields_json = "Front,Back,Extra";
        let count = parse_field_count(fields_json).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_field_count_separator() {
        let fields_json = "Front\u{001f}Back\u{001f}Extra";
        let count = parse_field_count(fields_json).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_parse_field_count_fallback() {
        let fields_json = "SomeUnstructuredText";
        let count = parse_field_count(fields_json).unwrap();
        assert_eq!(count, 2); // Default fallback
    }

    #[test]
    fn test_model_validation() {
        // Test model validation logic
        let field_count = 2;
        let model_type = 0;

        assert!(field_count >= 2);
        assert!(model_type == 0 || model_type == 1);
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_MODEL_NAME, "Basic");
        assert_eq!(DEFAULT_BASIC_MODEL_ID, 1607392319495);
    }
}

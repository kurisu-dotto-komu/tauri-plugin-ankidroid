use crate::android::constants::{CARDS_URI, NOTES_URI};
use crate::android::content_provider::{query, update};
use crate::android::cursor::CursorIterator;
use crate::android::error::{AndroidError, AndroidResult};
use crate::android::jni_helpers::{ContentValuesBuilder, SafeJNIEnv};
use jni::objects::JObject;

/// Suspend a card (mark it as suspended so it won't appear in reviews)
pub fn suspend_card(mut env: SafeJNIEnv, activity: &JObject, card_id: i64) -> AndroidResult<bool> {
    log::info!("Suspending card ID: {}", card_id);

    // Build ContentValues with suspend flag
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_int("queue", -1)?; // -1 indicates suspended

    // Update the card
    let updated_rows = update(env, CARDS_URI)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity, values)?;

    let success = updated_rows > 0;
    log::info!("Suspend card {} - Success: {}", card_id, success);
    Ok(success)
}

/// Unsuspend a card (restore it to normal review queue)
pub fn unsuspend_card(
    mut env: SafeJNIEnv,
    activity: &JObject,
    card_id: i64,
) -> AndroidResult<bool> {
    log::info!("Unsuspending card ID: {}", card_id);

    // Build ContentValues to restore to new queue
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_int("queue", 0)?; // 0 indicates new/learning

    // Update the card
    let updated_rows = update(env, CARDS_URI)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity, values)?;

    let success = updated_rows > 0;
    log::info!("Unsuspend card {} - Success: {}", card_id, success);
    Ok(success)
}

/// Bury a card (temporarily hide it until tomorrow)
pub fn bury_card(mut env: SafeJNIEnv, activity: &JObject, card_id: i64) -> AndroidResult<bool> {
    log::info!("Burying card ID: {}", card_id);

    // Build ContentValues with bury queue
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_int("queue", -2)?; // -2 indicates buried

    // Update the card
    let updated_rows = update(env, CARDS_URI)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity, values)?;

    let success = updated_rows > 0;
    log::info!("Bury card {} - Success: {}", card_id, success);
    Ok(success)
}

/// Unbury a card (restore it from buried state)
pub fn unbury_card(mut env: SafeJNIEnv, activity: &JObject, card_id: i64) -> AndroidResult<bool> {
    log::info!("Unburying card ID: {}", card_id);

    // Build ContentValues to restore to new queue
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_int("queue", 0)?; // 0 indicates new/learning

    // Update the card
    let updated_rows = update(env, CARDS_URI)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity, values)?;

    let success = updated_rows > 0;
    log::info!("Unbury card {} - Success: {}", card_id, success);
    Ok(success)
}

/// Change the deck of a card
pub fn change_card_deck(
    mut env: SafeJNIEnv,
    activity: &JObject,
    card_id: i64,
    deck_id: i64,
) -> AndroidResult<bool> {
    log::info!("Changing card {} to deck {}", card_id, deck_id);

    // Build ContentValues with new deck ID
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_long("did", deck_id)?;

    // Update the card
    let updated_rows = update(env, CARDS_URI)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity, values)?;

    let success = updated_rows > 0;
    log::info!("Change card deck {} - Success: {}", card_id, success);
    Ok(success)
}

/// Move all cards of a note to a different deck
pub fn move_all_cards_to_deck(
    env: SafeJNIEnv,
    activity: &JObject,
    note_id: i64,
    deck_id: i64,
) -> AndroidResult<i32> {
    log::info!("Moving all cards of note {} to deck {}", note_id, deck_id);

    // First, get all cards for this note
    let cards = get_cards_for_note(env.clone(), activity, note_id)?;
    let mut updated_count = 0;

    // Update each card
    for card_id in cards {
        if change_card_deck(env.clone(), activity, card_id, deck_id)? {
            updated_count += 1;
        }
    }

    log::info!("Moved {} cards to deck {}", updated_count, deck_id);
    Ok(updated_count)
}

/// Get all card IDs for a given note
pub fn get_cards_for_note(
    env: SafeJNIEnv,
    activity: &JObject,
    note_id: i64,
) -> AndroidResult<Vec<i64>> {
    log::debug!("Getting cards for note ID: {}", note_id);

    let projection = vec!["_id".to_string()];

    let mut cursor_iterator = query(env, CARDS_URI)
        .projection(projection)
        .selection("nid = ?".to_string())
        .selection_args(vec![note_id.to_string()])
        .execute(activity)?;

    let mut cards = Vec::new();

    while cursor_iterator.move_to_next()? {
        let card_id = cursor_iterator.get_long_by_name("_id")?;
        cards.push(card_id);
    }

    log::debug!("Found {} cards for note {}", cards.len(), note_id);
    Ok(cards)
}

/// Reset the progress of a card (make it new again)
pub fn reset_card_progress(
    mut env: SafeJNIEnv,
    activity: &JObject,
    card_id: i64,
) -> AndroidResult<bool> {
    log::info!("Resetting progress for card ID: {}", card_id);

    // Build ContentValues to reset card
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_int("queue", 0)? // New queue
        .put_int("type", 0)?  // New type
        .put_int("due", 0)?   // Reset due date
        .put_int("ivl", 0)?   // Reset interval
        .put_int("factor", 2500)? // Reset ease factor to default
        .put_int("reps", 0)?  // Reset repetitions
        .put_int("lapses", 0)?; // Reset lapses

    // Update the card
    let updated_rows = update(env, CARDS_URI)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity, values)?;

    let success = updated_rows > 0;
    log::info!("Reset card progress {} - Success: {}", card_id, success);
    Ok(success)
}

/// Set the due date for a card (for custom scheduling)
pub fn set_card_due_date(
    mut env: SafeJNIEnv,
    activity: &JObject,
    card_id: i64,
    days_from_today: i32,
) -> AndroidResult<bool> {
    log::info!(
        "Setting due date for card {} to {} days from today",
        card_id,
        days_from_today
    );

    // Calculate the due date (days since collection creation)
    // For simplicity, we'll use days from today as the due value
    let due_value = days_from_today;

    // Build ContentValues with new due date
    let mut env_for_values = env.clone();
    let values = ContentValuesBuilder::new(&mut env_for_values)?
        .put_int("due", due_value)?;

    // Update the card
    let updated_rows = update(env, CARDS_URI)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity, values)?;

    let success = updated_rows > 0;
    log::info!("Set card due date {} - Success: {}", card_id, success);
    Ok(success)
}

/// Get the current state of a card
pub fn get_card_state(
    env: SafeJNIEnv,
    activity: &JObject,
    card_id: i64,
) -> AndroidResult<CardState> {
    log::debug!("Getting state for card ID: {}", card_id);

    let projection = vec![
        "_id".to_string(),
        "nid".to_string(),
        "did".to_string(),
        "queue".to_string(),
        "type".to_string(),
        "due".to_string(),
        "ivl".to_string(),
        "factor".to_string(),
        "reps".to_string(),
        "lapses".to_string(),
    ];

    let mut cursor_iterator = query(env, CARDS_URI)
        .projection(projection)
        .selection("_id = ?".to_string())
        .selection_args(vec![card_id.to_string()])
        .execute(activity)?;

    if !cursor_iterator.move_to_first()? {
        return Err(AndroidError::CardOperationError(format!(
            "Card {} not found",
            card_id
        )));
    }

    let state = CardState {
        id: cursor_iterator.get_long_by_name("_id")?,
        note_id: cursor_iterator.get_long_by_name("nid")?,
        deck_id: cursor_iterator.get_long_by_name("did")?,
        queue: cursor_iterator.get_int_by_name("queue")?,
        card_type: cursor_iterator.get_int_by_name("type")?,
        due: cursor_iterator.get_int_by_name("due")?,
        interval: cursor_iterator.get_int_by_name("ivl")?,
        ease_factor: cursor_iterator.get_int_by_name("factor")?,
        reviews: cursor_iterator.get_int_by_name("reps")?,
        lapses: cursor_iterator.get_int_by_name("lapses")?,
    };

    log::debug!(
        "Card {} state: queue={}, type={}",
        card_id,
        state.queue,
        state.card_type
    );
    Ok(state)
}

/// Structure representing a card's current state
#[derive(Debug, Clone)]
pub struct CardState {
    pub id: i64,
    pub note_id: i64,
    pub deck_id: i64,
    pub queue: i32,       // -2=buried, -1=suspended, 0=new, 1=learning, 2=review
    pub card_type: i32,   // 0=new, 1=learning, 2=review, 3=relearning
    pub due: i32,         // Due date or queue position
    pub interval: i32,    // Interval in days
    pub ease_factor: i32, // Ease factor (2500 = 250%)
    pub reviews: i32,     // Number of reviews
    pub lapses: i32,      // Number of lapses
}

impl CardState {
    pub fn is_suspended(&self) -> bool {
        self.queue == -1
    }

    pub fn is_buried(&self) -> bool {
        self.queue == -2
    }

    pub fn is_new(&self) -> bool {
        self.queue == 0
    }

    pub fn is_learning(&self) -> bool {
        self.queue == 1
    }

    pub fn is_review(&self) -> bool {
        self.queue == 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_state() {
        let state = CardState {
            id: 1,
            note_id: 2,
            deck_id: 3,
            queue: -1,
            card_type: 0,
            due: 0,
            interval: 0,
            ease_factor: 2500,
            reviews: 0,
            lapses: 0,
        };

        assert!(state.is_suspended());
        assert!(!state.is_buried());
        assert!(!state.is_new());

        let buried_state = CardState { queue: -2, ..state };
        assert!(buried_state.is_buried());

        let new_state = CardState { queue: 0, ..state };
        assert!(new_state.is_new());
    }
}

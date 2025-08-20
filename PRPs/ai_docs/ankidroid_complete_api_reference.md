# Complete AnkiDroid API Reference for JNI Implementation

## Critical Information

This document provides the COMPLETE and ACCURATE AnkiDroid API reference with exact JNI method signatures, ContentProvider URIs, and common exception patterns. This is the authoritative reference to fix the exceptions in the tauri-plugin-ankidroid implementation.

## 1. Content Provider URIs (FlashCardsContract)

### Authority Constants
```java
// Primary authority (modern versions)
public static final String AUTHORITY = "com.ichi2.anki.flashcards";

// Legacy authority (older versions)
public static final String LEGACY_AUTHORITY = "com.ichi2.anki.provider";

// Permission required
public static final String READ_WRITE_PERMISSION = "com.ichi2.anki.permission.READ_WRITE_DATABASE";
```

### Content URIs
```java
// Base URIs
content://com.ichi2.anki.flashcards/notes       // Note operations
content://com.ichi2.anki.flashcards/notes_v2    // Note operations v2
content://com.ichi2.anki.flashcards/cards       // Card operations
content://com.ichi2.anki.flashcards/decks       // Deck operations
content://com.ichi2.anki.flashcards/models      // Model operations
content://com.ichi2.anki.flashcards/schedule    // Scheduling operations
content://com.ichi2.anki.flashcards/review_info // Review information
content://com.ichi2.anki.flashcards/media       // Media operations
content://com.ichi2.anki.flashcards/selected_deck // Selected deck

// Special URIs for card operations
content://com.ichi2.anki.flashcards/notes/{noteId}/cards          // All cards for a note
content://com.ichi2.anki.flashcards/notes/{noteId}/cards/{cardOrd} // Specific card
content://com.ichi2.anki.flashcards/models/{modelId}/empty_cards   // Empty cards for deletion
```

## 2. Column Names for ContentValues and Queries

### Note Columns (FlashCardsContract.Note)
```java
public static final String _ID = "_id";           // Note ID (long)
public static final String GUID = "guid";         // Globally unique ID (string)
public static final String MID = "mid";           // Model ID (long)
public static final String MOD = "mod";           // Modification time (long)
public static final String USQN = "usqn";         // Update sequence number (int)
public static final String TAGS = "tags";         // Space-separated tags (string)
public static final String FLDS = "flds";         // Fields separated by \u001f (string)
public static final String FLAGS = "flags";       // Flags (int)
public static final String DATA = "data";         // Extra data (string)
public static final String SFLD = "sfld";         // Sort field (string)
public static final String CSUM = "csum";         // Checksum (long)
```

### Card Columns (FlashCardsContract.Card)
```java
public static final String _ID = "_id";           // Card ID (long)
public static final String NID = "nid";           // Note ID (long)
public static final String DID = "did";           // Deck ID (long)
public static final String ORD = "ord";           // Card ordinal/template index (int)
public static final String MOD = "mod";           // Modification time (long)
public static final String TYPE = "type";         // Card type (int: 0=new, 1=learning, 2=review, 3=relearning)
public static final String QUEUE = "queue";       // Queue type (int: -3=user buried, -2=sched buried, -1=suspended, 0=new, 1=learning, 2=review, 3=in learning, 4=preview)
public static final String DUE = "due";           // Due date/time (long)
public static final String IVL = "ivl";           // Interval in days (int)
public static final String FACTOR = "factor";     // Ease factor x1000 (int)
public static final String REPS = "reps";         // Number of reviews (int)
public static final String LAPSES = "lapses";     // Number of lapses (int)
public static final String LEFT = "left";         // Reviews left today (int)
public static final String ODUE = "odue";         // Original due (long)
public static final String ODID = "odid";         // Original deck ID (long)
public static final String FLAGS = "flags";       // Card flags (int)
public static final String DATA = "data";         // Extra data (string)

// Additional query-only columns
public static final String QUESTION = "question";             // Card question HTML (string)
public static final String ANSWER = "answer";                 // Card answer HTML (string)
public static final String QUESTION_SIMPLE = "question_simple"; // Plain text question (string)
public static final String ANSWER_SIMPLE = "answer_simple";     // Plain text answer (string)
public static final String ANSWER_PURE = "answer_pure";         // Answer without question (string)
```

### Deck Columns (FlashCardsContract.Deck)
```java
public static final String DECK_ID = "deck_id";        // Deck ID (long)
public static final String DECK_NAME = "deck_name";    // Deck name (string)
public static final String DECK_DESC = "deck_desc";    // Deck description (string)
public static final String DECK_DYN = "deck_dyn";      // Dynamic deck flag (int: 0 or 1)
public static final String DECK_COUNTS = "deck_counts"; // JSON array: [new, learning, review] (string)
public static final String OPTIONS = "deck_options";   // Deck configuration JSON (string)
public static final String DECK_CONF = "deck_conf";    // Deck configuration ID (long)
```

### Model Columns (FlashCardsContract.Model)
```java
public static final String MID = "mid";           // Model ID (long)
public static final String NAME = "name";         // Model name (string)
public static final String FLDS = "flds";         // Fields JSON array (string)
public static final String TMPLS = "tmpls";       // Templates JSON array (string)
public static final String CSS = "css";           // CSS styling (string)
public static final String TYPE = "type";         // Model type (int: 0=standard, 1=cloze)
public static final String LATEX_PRE = "latex_pre";   // LaTeX preamble (string)
public static final String LATEX_POST = "latex_post"; // LaTeX postamble (string)
public static final String SORT_FIELD = "sort_field"; // Sort field index (int)
public static final String REQ = "req";           // Required fields array (string)
public static final String DID = "did";           // Default deck ID (long)
```

### ReviewInfo Columns (FlashCardsContract.ReviewInfo)
```java
public static final String NOTE_ID = "note_id";    // Note ID (long)
public static final String CARD_ORD = "card_ord";  // Card ordinal (int)
public static final String BUTTON = "button";      // Button pressed (int: 1=again, 2=hard, 3=good, 4=easy)
public static final String TIME = "time";          // Review time in ms (long)
public static final String EASE = "ease";          // New ease factor (int)
public static final String CARD_ID = "card_id";    // Card ID (long)
public static final String NEXT_REVIEW = "next_review"; // Next review time (long)
public static final String SUSPEND = "suspend";    // Suspend flag (int: 0 or 1)
```

### Media Columns (FlashCardsContract.AnkiMedia)
```java
public static final String FILE_URI = "file_uri";           // File URI (string)
public static final String PREFERRED_NAME = "preferred_name"; // Preferred filename (string)
public static final String MIME_TYPE = "mime_type";         // MIME type (string)
```

## 3. Content Provider Operations

### Insert Note (CRITICAL - EXACT FIELDS)
```java
ContentValues values = new ContentValues();
values.put("mid", modelId);                    // Model ID (required)
values.put("flds", fields);                    // Fields with \u001f separator (required)
values.put("tags", tags);                      // Space-separated tags (optional)
// DO NOT include "did" here - it causes "Queue 'X' is unknown" errors
// Deck assignment happens through the deck's default model setting

Uri uri = contentResolver.insert(Uri.parse("content://com.ichi2.anki.flashcards/notes"), values);
// Extract note ID from returned URI: content://com.ichi2.anki.flashcards/notes/{noteId}
```

### Query Notes
```java
String[] projection = {"_id", "mid", "flds", "tags", "mod"};
String selection = "mid = ?";
String[] selectionArgs = {String.valueOf(modelId)};
String sortOrder = "mod DESC";

Cursor cursor = contentResolver.query(
    Uri.parse("content://com.ichi2.anki.flashcards/notes"),
    projection,
    selection,
    selectionArgs,
    sortOrder
);
```

### Update Note
```java
ContentValues values = new ContentValues();
values.put("flds", updatedFields);  // New fields with \u001f separator
values.put("tags", updatedTags);    // New tags

int rowsUpdated = contentResolver.update(
    Uri.parse("content://com.ichi2.anki.flashcards/notes/" + noteId),
    values,
    null,
    null
);
```

### Delete Note
```java
int rowsDeleted = contentResolver.delete(
    Uri.parse("content://com.ichi2.anki.flashcards/notes/" + noteId),
    null,
    null
);
```

### Suspend Card
```java
ContentValues values = new ContentValues();
values.put("note_id", noteId);
values.put("card_ord", cardOrd);  // 0-based card template index
values.put("suspend", 1);         // 1 = suspend, 0 = unsuspend

int updated = contentResolver.update(
    Uri.parse("content://com.ichi2.anki.flashcards/review_info"),
    values,
    null,
    null
);
```

### Change Card Deck
```java
Uri cardUri = Uri.parse("content://com.ichi2.anki.flashcards/notes/" + noteId + "/cards/" + cardOrd);

ContentValues values = new ContentValues();
values.put("did", deckId);

int updated = contentResolver.update(cardUri, values, null, null);
```

### Add Media
```java
ContentValues values = new ContentValues();
values.put("file_uri", fileUri.toString());    // File URI from FileProvider
values.put("preferred_name", fileName);        // Desired filename
values.put("mime_type", mimeType);            // e.g., "image/jpeg"

Uri insertedUri = contentResolver.insert(
    Uri.parse("content://com.ichi2.anki.flashcards/media"),
    values
);
// Returns filename actually used (may differ if conflicts)
String actualFilename = insertedUri.getLastPathSegment();
```

## 4. AddContentApi Method Signatures (Java)

### Constructor
```java
public AddContentApi(Context context)
```

### Package Check
```java
public static String getAnkiDroidPackageName(Context context)
// Returns: Package name if installed, null otherwise
```

### Note Operations
```java
public Long addNote(long modelId, long deckId, String[] fields, Set<String> tags)
// Returns: Note ID or null on failure

public int addNotes(long modelId, long deckId, List<String[]> fieldsList, List<Set<String>> tagsList)
// Returns: Number added (negative on error)
```

### Deck Operations
```java
public long addNewDeck(String deckName)
// Returns: Deck ID

public String getDeckName(long did)
// Returns: Deck name or null

public Map<Long, String> getDeckList()
// Returns: Map of deck ID to name
```

### Model Operations
```java
public long addNewBasicModel(String name)
// Returns: Model ID

public long addNewBasic2Model(String name)
// Returns: Model ID

public long addNewCustomModel(String name, String[] fields, String[] cards, 
                              String[] qfmt, String[] afmt, String css, 
                              Long did, Long sortf)
// Returns: Model ID

public String getModelName(long mid)
// Returns: Model name or null

public String[] getFieldList(Long modelId)
// Returns: Array of field names

public Map<Long, String> getModelList(int minNumFields)
// Returns: Map of model ID to name
```

### Utility Methods
```java
public int getApiHostSpecVersion()
// Returns: API version number

public SparseArray<List<NoteInfo>> findDuplicateNotes(long modelId, List<String> keys)
// Returns: Duplicate notes by index
```

## 5. JNI Method Signatures (Critical for Rust Implementation)

### Getting JNI Environment
```rust
// Attach current thread
let vm = jni::JavaVM::from_raw(java_vm_ptr)?;
let env = vm.attach_current_thread()?;

// Or get from existing attachment
let env = vm.get_env()?;
```

### ContentResolver Operations
```rust
// Get ContentResolver
let content_resolver = env.call_method(
    activity,
    "getContentResolver",
    "()Landroid/content/ContentResolver;",
    &[]
)?;

// Create ContentValues
let content_values = env.new_object(
    "android/content/ContentValues",
    "()V",
    &[]
)?;

// Put values
env.call_method(
    &content_values,
    "put",
    "(Ljava/lang/String;Ljava/lang/String;)V",
    &[
        JValue::Object(&env.new_string("flds")?),
        JValue::Object(&env.new_string(fields)?)
    ]
)?;

// Insert
let uri = env.call_method(
    content_resolver,
    "insert",
    "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
    &[
        JValue::Object(&uri_object),
        JValue::Object(&content_values)
    ]
)?;
```

### Cursor Operations
```rust
// Query
let cursor = env.call_method(
    content_resolver,
    "query",
    "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
    &[
        JValue::Object(&uri),
        JValue::Object(&projection),
        JValue::Object(&selection),
        JValue::Object(&selection_args),
        JValue::Object(&sort_order)
    ]
)?;

// Navigate cursor
let has_data = env.call_method(&cursor, "moveToFirst", "()Z", &[])?;
let has_next = env.call_method(&cursor, "moveToNext", "()Z", &[])?;

// Get column index
let column_index = env.call_method(
    &cursor,
    "getColumnIndex",
    "(Ljava/lang/String;)I",
    &[JValue::Object(&column_name)]
)?;

// Get values
let string_value = env.call_method(
    &cursor,
    "getString",
    "(I)Ljava/lang/String;",
    &[JValue::Int(column_index)]
)?;

let long_value = env.call_method(
    &cursor,
    "getLong",
    "(I)J",
    &[JValue::Long(column_index)]
)?;

// Close cursor (CRITICAL)
env.call_method(&cursor, "close", "()V", &[])?;
```

### Exception Handling Pattern
```rust
// After EVERY JNI call
if env.exception_check()? {
    let exception = env.exception_occurred()?;
    env.exception_clear()?;
    
    // Get exception message
    let message = env.call_method(
        exception,
        "getMessage",
        "()Ljava/lang/String;",
        &[]
    )?;
    
    return Err(AndroidError::JavaException(message));
}
```

## 6. Common Exceptions and Their Fixes

### Exception 1: NullPointerException on Collection.getConf()
```
java.lang.NullPointerException: Attempt to invoke virtual method 
'org.json.JSONObject com.ichi2.libanki.Collection.getConf()' on a null object reference
```

**Cause**: AnkiDroid's collection is not initialized (app not running or database not loaded)

**Fix**:
1. Check if AnkiDroid is running
2. Use AddContentApi.getAnkiDroidPackageName() to verify availability
3. Handle null returns gracefully
4. Consider triggering AnkiDroid to start first

### Exception 2: Queue 'X' is unknown
```
BackendInvalidInputException: Queue '2' is unknown
```

**Cause**: Including 'did' (deck ID) in ContentValues when inserting notes

**Fix**:
```rust
// WRONG - causes queue error
values.put("mid", model_id);
values.put("did", deck_id);  // DO NOT INCLUDE
values.put("flds", fields);

// CORRECT
values.put("mid", model_id);
values.put("flds", fields);
values.put("tags", tags);
// Deck is determined by the model's default deck setting
```

### Exception 3: NullPointerException on JSONObject.getLong()
```
java.lang.NullPointerException: Attempt to invoke virtual method 
'long com.ichi2.utils.JSONObject.getLong(java.lang.String)' on a null object reference
```

**Cause**: Trying to access model or deck information that doesn't exist

**Fix**:
1. Always check if model/deck exists before using
2. Create default model/deck if needed
3. Validate IDs before operations

### Exception 4: BackendInvalidInputException: card was modified
```
BackendInvalidInputException: card was modified
```

**Cause**: Attempting to modify a card that has been changed since it was queried

**Fix**:
1. Re-query the card before modification
2. Use atomic operations when possible
3. Handle modification conflicts gracefully

### Exception 5: SecurityException on ContentProvider
```
java.lang.SecurityException: Permission Denial: reading com.ichi2.anki.provider
```

**Cause**: Missing or not granted READ_WRITE_DATABASE permission

**Fix**:
```rust
// Check permission
let permission = "com.ichi2.anki.permission.READ_WRITE_DATABASE";
let check_result = env.call_method(
    context,
    "checkSelfPermission",
    "(Ljava/lang/String;)I",
    &[JValue::Object(&env.new_string(permission)?)]
)?;

// 0 = PERMISSION_GRANTED, -1 = PERMISSION_DENIED
if check_result.i()? != 0 {
    // Request permission or use intent fallback
}
```

## 7. Field Separator and Formatting

### CRITICAL: Field Separator
```rust
// Fields MUST be separated by ASCII Unit Separator
const FIELD_SEPARATOR: &str = "\u{001f}";

// Creating fields string
let fields = format!("{}{}{}", front, FIELD_SEPARATOR, back);

// Parsing fields string
let parts: Vec<&str> = fields_str.split(FIELD_SEPARATOR).collect();
```

### Tags Format
```rust
// Tags are space-separated
let tags = "tag1 tag2 tag3";

// Parse tags
let tag_list: Vec<&str> = tags.split(' ').filter(|t| !t.is_empty()).collect();
```

## 8. Model Detection Pattern

### Finding Basic Model
```rust
fn find_basic_model_id(env: &JNIEnv, content_resolver: &JObject) -> Result<i64> {
    let models_uri = "content://com.ichi2.anki.flashcards/models";
    let projection = ["mid", "name", "flds"];
    
    let cursor = query(env, content_resolver, models_uri, projection)?;
    
    while cursor.move_to_next() {
        let name = cursor.get_string("name")?;
        let fields_json = cursor.get_string("flds")?;
        
        // Parse fields JSON to check count
        let fields = parse_json_array(fields_json)?;
        
        // Basic model has exactly 2 fields
        if name == "Basic" && fields.len() == 2 {
            return Ok(cursor.get_long("mid")?);
        }
    }
    
    // Fallback: create new Basic model
    create_basic_model(env, content_resolver)
}
```

## 9. Deck Selection Strategy

### Default Deck Handling
```rust
fn get_or_create_deck(env: &JNIEnv, deck_name: Option<String>) -> Result<i64> {
    match deck_name {
        Some(name) => {
            // Try to find existing deck
            if let Some(deck_id) = find_deck_by_name(&name)? {
                deck_id
            } else {
                // Create new deck
                create_deck(&name)?
            }
        }
        None => {
            // Use selected deck
            get_selected_deck_id()?
        }
    }
}

fn get_selected_deck_id(env: &JNIEnv) -> Result<i64> {
    let uri = "content://com.ichi2.anki.flashcards/selected_deck";
    let cursor = query(env, uri)?;
    
    if cursor.move_to_first() {
        cursor.get_long("deck_id")
    } else {
        1 // Default deck ID
    }
}
```

## 10. Complete Working Example

```rust
pub fn create_card_complete(
    env: &mut JNIEnv,
    activity: &JObject,
    front: String,
    back: String,
    deck_name: Option<String>,
    tags: Option<String>,
) -> Result<i64> {
    // 1. Get ContentResolver
    let content_resolver = env.call_method(
        activity,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[]
    )?.l()?;
    
    // 2. Find or create model
    let model_id = find_basic_model_id(env, &content_resolver)?;
    
    // 3. Prepare fields with separator
    let fields = format!("{}\u{001f}{}", front, back);
    
    // 4. Create ContentValues
    let content_values = env.new_object(
        "android/content/ContentValues",
        "()V",
        &[]
    )?;
    
    // 5. Put values (NO deck_id!)
    env.call_method(
        &content_values,
        "put",
        "(Ljava/lang/String;Ljava/lang/Long;)V",
        &[
            JValue::Object(&env.new_string("mid")?),
            JValue::Long(model_id)
        ]
    )?;
    
    env.call_method(
        &content_values,
        "put",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[
            JValue::Object(&env.new_string("flds")?),
            JValue::Object(&env.new_string(&fields)?)
        ]
    )?;
    
    if let Some(tags_str) = tags {
        env.call_method(
            &content_values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[
                JValue::Object(&env.new_string("tags")?),
                JValue::Object(&env.new_string(&tags_str)?)
            ]
        )?;
    }
    
    // 6. Insert note
    let notes_uri = env.call_static_method(
        "android/net/Uri",
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&env.new_string("content://com.ichi2.anki.flashcards/notes")?)]
    )?.l()?;
    
    let result_uri = env.call_method(
        &content_resolver,
        "insert",
        "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
        &[
            JValue::Object(&notes_uri),
            JValue::Object(&content_values)
        ]
    )?.l()?;
    
    // 7. Check for exceptions
    if env.exception_check()? {
        env.exception_clear()?;
        return Err(AndroidError::InsertFailed);
    }
    
    // 8. Extract note ID from URI
    let uri_string = env.call_method(
        &result_uri,
        "toString",
        "()Ljava/lang/String;",
        &[]
    )?.l()?;
    
    let uri_str = env.get_string(&uri_string.into())?;
    let note_id = uri_str
        .split('/')
        .last()
        .and_then(|id| id.parse::<i64>().ok())
        .ok_or(AndroidError::InvalidUri)?;
    
    // 9. Handle deck assignment if specified
    if let Some(deck) = deck_name {
        let deck_id = get_or_create_deck(env, Some(deck))?;
        // Move cards to specified deck
        move_cards_to_deck(env, &content_resolver, note_id, deck_id)?;
    }
    
    Ok(note_id)
}
```

## 11. RAII Pattern for Safe Resource Management

```rust
pub struct CursorGuard<'a> {
    env: &'a JNIEnv<'a>,
    cursor: JObject<'a>,
}

impl<'a> CursorGuard<'a> {
    pub fn new(env: &'a JNIEnv<'a>, cursor: JObject<'a>) -> Self {
        Self { env, cursor }
    }
    
    pub fn move_to_first(&self) -> Result<bool> {
        Ok(self.env.call_method(&self.cursor, "moveToFirst", "()Z", &[])?.z()?)
    }
    
    pub fn move_to_next(&self) -> Result<bool> {
        Ok(self.env.call_method(&self.cursor, "moveToNext", "()Z", &[])?.z()?)
    }
    
    pub fn get_string(&self, column: &str) -> Result<String> {
        let index = self.env.call_method(
            &self.cursor,
            "getColumnIndex",
            "(Ljava/lang/String;)I",
            &[JValue::Object(&self.env.new_string(column)?)]
        )?.i()?;
        
        let jstring = self.env.call_method(
            &self.cursor,
            "getString",
            "(I)Ljava/lang/String;",
            &[JValue::Int(index)]
        )?.l()?;
        
        Ok(self.env.get_string(&jstring.into())?.into())
    }
}

impl<'a> Drop for CursorGuard<'a> {
    fn drop(&mut self) {
        // Always close cursor, ignore errors in drop
        let _ = self.env.call_method(&self.cursor, "close", "()V", &[]);
    }
}
```

## Key Takeaways for Implementation

1. **NEVER include 'did' in ContentValues** when inserting notes - this causes "Queue unknown" errors
2. **ALWAYS use \u{001f} as field separator** - this is non-negotiable
3. **ALWAYS check for exceptions after JNI calls** - Java exceptions don't auto-propagate
4. **ALWAYS close cursors** - use RAII pattern to ensure cleanup
5. **Handle null returns** - AnkiDroid API returns null when not available
6. **Check permissions** - Required for Android M+ (API 23+)
7. **Validate model existence** - Users can delete/rename models
8. **Use proper URI construction** - Different operations need different URI patterns
9. **Local reference management** - Use push_local_frame/pop_local_frame for operations creating many objects
10. **Thread attachment** - Ensure thread is attached before JNI operations

This comprehensive reference should resolve all the JNI implementation issues and exceptions being encountered.
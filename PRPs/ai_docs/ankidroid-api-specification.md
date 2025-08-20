# AnkiDroid API Complete Specification

This document provides a comprehensive specification of the AnkiDroid API based on the Kotlin/Java implementation in `PRPs/research/AnkiDroidAPI/`.

## Overview

The AnkiDroid API provides access to flashcard data through Android's ContentProvider mechanism. It consists of:
1. **FlashCardsContract**: Defines URIs, constants, and data schemas
2. **AddContentApi**: High-level convenience API for common operations
3. **Data Models**: Note, Card, Deck, Model (NoteType) representations
4. **Utilities**: Helper functions for field/tag manipulation

## Authority and Permissions

```kotlin
const val AUTHORITY = "com.ichi2.anki.flashcards"
const val READ_WRITE_PERMISSION = "com.ichi2.anki.permission.READ_WRITE_PERMISSION"
val AUTHORITY_URI = Uri.parse("content://$AUTHORITY")
```

## Content URIs and Operations

### URI Structure and Supported Operations

| URI Pattern | Description | Supported Operations |
|------------|-------------|---------------------|
| `notes` | All notes | insert(mid), query() |
| `notes/<note_id>` | Specific note | query(), update(), delete() |
| `notes/<note_id>/cards` | Cards of a note | query() |
| `notes/<note_id>/cards/<ord>` | Specific card | query(), update() |
| `notes_v2` | Notes with direct SQL | query() |
| `models` | All note types | query() |
| `models/<model_id>` | Specific note type | query() |
| `models/current` | Current note type | query() |
| `models/<model_id>/templates` | Card templates | query() |
| `models/<model_id>/templates/<ord>` | Specific template | update() |
| `decks` | All decks | query(), insert() |
| `decks/<deck_id>` | Specific deck | query() |
| `selected_deck` | Currently selected deck | query(), update() |
| `schedule` | Review schedule | query(), update() |
| `media` | Media files | insert() |

## Data Models

### Note

Represents a flashcard note (the data from which cards are generated).

**Columns:**
- `_id` (long, read-only): Note ID
- `guid` (String, read-only): Globally unique identifier
- `mid` (long, read-only): Model/NoteType ID
- `mod` (long, read-only): Modification timestamp
- `usn` (int, read-only): Update sequence number
- `tags` (String, read-write): Space-separated tags
- `flds` (String, read-write): Fields separated by 0x1F character
- `sfld` (String, read-only): Sort field
- `csum` (long, read-only): Checksum of first field
- `flags` (int, read-only): Flags
- `data` (String, read-only): Additional data

**Projection:**
```kotlin
DEFAULT_PROJECTION = arrayOf(_ID, GUID, MID, MOD, USN, TAGS, FLDS, SFLD, CSUM, FLAGS, DATA)
```

### Card

Represents a single card instance of a note.

**Columns:**
- `note_id` (long, read-only): Parent note ID
- `ord` (int, read-only): Card ordinal (0-based)
- `card_name` (String, read-only): Card name
- `deck_id` (long, read-write): Deck ID
- `question` (String, read-only): Question HTML
- `answer` (String, read-only): Answer HTML
- `question_simple` (String, read-only): Question without CSS
- `answer_simple` (String, read-only): Answer without CSS
- `answer_pure` (String, read-only): Pure answer text

### Deck

Represents a deck containing cards.

**Columns:**
- `deck_id` (long, read-only): Deck ID
- `deck_name` (String): Deck name
- `deck_desc` (String): Deck description
- `deck_count` (JSONArray, read-only): [learn, review, new] counts
- `options` (JSONObject, read-only): Deck options
- `deck_dyn` (int, read-only): 1 if filtered deck

### Model (Note Type)

Defines the structure and rendering of notes.

**Columns:**
- `_id` (long, read-only): Model ID
- `name` (String): Model name
- `field_names` (String, read-only): 0x1F-separated field names
- `num_cards` (int, read-only): Number of card templates
- `css` (String): Shared CSS
- `deck_id` (long, read-only): Default deck ID
- `sort_field_index` (int, read-only): Sort field index
- `type` (int, read-only): 0=normal, 1=cloze
- `latex_post` (String, read-only): LaTeX postamble
- `latex_pre` (String, read-only): LaTeX preamble

### CardTemplate

Template for generating cards from notes.

**Columns:**
- `_id` (long): Virtual ID
- `model_id` (long): Parent model ID
- `ord` (int): Template ordinal
- `card_template_name` (String): Template name
- `question_format` (String): Question template
- `answer_format` (String): Answer template
- `browser_question_format` (String, optional): Browser question
- `browser_answer_format` (String, optional): Browser answer

### ReviewInfo

Information for reviewing cards.

**Columns:**
- `note_id` (long, read-write): Note ID
- `ord` (int, read-write): Card ordinal
- `button_count` (int, read-only): Number of answer buttons
- `next_review_times` (JSONArray, read-only): Review intervals
- `media_files` (JSONArray, read-only): Media in card
- `answer_ease` (int, write-only): Answer ease (1-4)
- `time_taken` (long, write-only): Time to answer (ms)
- `buried` (int, write-only): 1 to bury
- `suspended` (int, write-only): 1 to suspend

## AddContentApi Methods

### Note Operations

#### addNote
```kotlin
fun addNote(modelId: Long, deckId: Long, fields: Array<String>, tags: Set<String>?): Long?
```
Creates a new note with specified fields and tags.

**Parameters:**
- `modelId`: ID of the note type to use
- `deckId`: ID of the deck for cards (use 1 for default)
- `fields`: Array of field values (must match model field count)
- `tags`: Optional set of tags

**Returns:** Note ID on success, null on failure

#### addNotes
```kotlin
fun addNotes(modelId: Long, deckId: Long, fieldsList: List<Array<String>>, tagsList: List<Set<String>?>?): Int
```
Bulk creates multiple notes.

**Returns:** Number of successfully added notes

#### updateNoteTags
```kotlin
fun updateNoteTags(noteId: Long, tags: Set<String>): Boolean
```
Updates tags for an existing note.

#### updateNoteFields
```kotlin
fun updateNoteFields(noteId: Long, fields: Array<String>): Boolean
```
Updates fields for an existing note.

#### getNote
```kotlin
fun getNote(noteId: Long): NoteInfo?
```
Retrieves a note by ID.

#### findDuplicateNotes
```kotlin
fun findDuplicateNotes(mid: Long, key: String): List<NoteInfo?>
fun findDuplicateNotes(mid: Long, keys: List<String>): SparseArray<MutableList<NoteInfo?>>?
```
Finds notes with duplicate first fields.

#### getNoteCount
```kotlin
fun getNoteCount(mid: Long): Int
```
Gets count of notes for a model.

### Model Operations

#### addNewBasicModel
```kotlin
fun addNewBasicModel(name: String): Long?
```
Creates a basic front/back note type.

#### addNewBasic2Model
```kotlin
fun addNewBasic2Model(name: String): Long?
```
Creates a basic note type with reverse card.

#### addNewCustomModel
```kotlin
fun addNewCustomModel(
    name: String,
    fields: Array<String>,
    cards: Array<String>,
    qfmt: Array<String>,
    afmt: Array<String>,
    css: String?,
    did: Long?,
    sortf: Int?
): Long?
```
Creates a custom note type with full control.

#### getCurrentModelId
```kotlin
val currentModelId: Long
```
Gets the currently selected model ID.

#### getFieldList
```kotlin
fun getFieldList(modelId: Long): Array<String>?
```
Gets field names for a model.

#### getModelList
```kotlin
fun getModelList(minNumFields: Int = 1): Map<Long, String>?
```
Gets all models with at least minNumFields fields.

#### getModelName
```kotlin
fun getModelName(mid: Long): String?
```
Gets the name of a model by ID.

### Deck Operations

#### addNewDeck
```kotlin
fun addNewDeck(deckName: String): Long?
```
Creates a new deck.

#### getDeckList
```kotlin
val deckList: Map<Long, String>?
```
Gets all decks as ID->name map.

#### getSelectedDeckName
```kotlin
val selectedDeckName: String?
```
Gets the name of the currently selected deck.

#### getDeckName
```kotlin
fun getDeckName(did: Long): String?
```
Gets deck name by ID.

### Media Operations

#### addMediaFromUri
```kotlin
fun addMediaFromUri(fileUri: Uri, preferredName: String, mimeType: String): String?
```
Adds a media file to the collection.

**Parameters:**
- `fileUri`: URI to the media file
- `preferredName`: Desired filename (without extension)
- `mimeType`: "audio" or "image"

**Returns:** Formatted media reference string for card fields

### Preview Operations

#### previewNewNote
```kotlin
fun previewNewNote(mid: Long, flds: Array<String>): Map<String, Map<String, String>>?
```
Generates HTML preview for a note without saving.

**Returns:** Map of card name to {"q": question_html, "a": answer_html}

### API Information

#### getApiHostSpecVersion
```kotlin
val apiHostSpecVersion: Int
```
Gets the API specification version:
- 1: AnkiDroid 2.5 (slower bulk operations)
- 2: AnkiDroid 2.6+ (optimized bulk operations)

## Utility Functions

### Field Operations

#### joinFields
```kotlin
fun joinFields(fields: Array<String>): String
```
Joins fields with 0x1F separator.

#### splitFields
```kotlin
fun splitFields(fields: String): Array<String>
```
Splits fields by 0x1F separator.

### Tag Operations

#### joinTags
```kotlin
fun joinTags(tags: Set<String>): String
```
Joins tags with spaces, replacing spaces in tags with underscores.

#### splitTags
```kotlin
fun splitTags(tags: String): Array<String>
```
Splits tags by whitespace.

### Checksum Operations

#### fieldChecksum
```kotlin
fun fieldChecksum(data: String): Long
```
Calculates checksum for duplicate detection:
1. Strip HTML tags and entities
2. Keep media filenames
3. Calculate SHA1 hash
4. Return first 8 hex characters as long

## Built-in Models

### BasicModel
Simple front/back card:
- Fields: ["Front", "Back"]
- Cards: ["Card 1"]
- Question: `{{Front}}`
- Answer: `{{FrontSide}}<hr id="answer">{{Back}}`

### Basic2Model
Front/back with reverse:
- Fields: ["Front", "Back"]
- Cards: ["Card 1", "Card 2"]
- Card 1: Front->Back
- Card 2: Back->Front

## Error Handling

Common error scenarios:
- `AnkiDroidNotAvailable`: AnkiDroid not installed or API not accessible
- `PermissionDenied`: Missing READ_WRITE_PERMISSION
- `InvalidModelId`: Model doesn't exist
- `InvalidDeckId`: Deck doesn't exist
- `DuplicateNote`: Note with same first field exists
- `FieldCountMismatch`: Fields array doesn't match model

## Constants

### Special Values
- `DEFAULT_DECK_ID = 1`: Default deck ID
- `FIELD_SEPARATOR = '\u001f'`: Field separator character
- `TEST_TAG = "PREVIEW_NOTE"`: Tag used for preview notes

### MIME Types
- Note: `vnd.android.cursor.item/vnd.com.ichi2.anki.note`
- Notes: `vnd.android.cursor.dir/vnd.com.ichi2.anki.note`
- Model: `vnd.android.cursor.item/vnd.com.ichi2.anki.model`
- Models: `vnd.android.cursor.dir/vnd.com.ichi2.anki.model`
- Card: `vnd.android.cursor.item/vnd.com.ichi2.anki.card`
- Cards: `vnd.android.cursor.dir/vnd.com.ichi2.anki.card`
- Deck: `vnd.android.cursor.dir/vnd.com.ichi2.anki.deck`
- ReviewInfo: `vnd.android.cursor.dir/vnd.com.ichi2.anki.review_info`

## Version Compatibility

- API v1 (AnkiDroid 2.5):
  - Slower bulk operations
  - Browser query syntax for note queries
  - Limited model persistence
  
- API v2 (AnkiDroid 2.6+):
  - Optimized bulk insert
  - Direct SQL queries via notes_v2
  - Improved model handling
  - Batch operations support

## Usage Examples

### Adding a Note
```kotlin
val api = AddContentApi(context)
val modelId = api.currentModelId
val deckId = 1L // Default deck
val fields = arrayOf("Front text", "Back text")
val tags = setOf("tag1", "tag2")
val noteId = api.addNote(modelId, deckId, fields, tags)
```

### Querying Notes
```kotlin
val uri = Uri.parse("content://com.ichi2.anki.flashcards/notes")
val cursor = contentResolver.query(
    uri,
    null, // projection
    "tag:important", // selection (browser syntax)
    null, // selectionArgs
    null  // sortOrder
)
```

### Updating a Card's Deck
```kotlin
val noteUri = Uri.parse("content://com.ichi2.anki.flashcards/notes/$noteId")
val cardUri = Uri.withAppendedPath(noteUri, "cards/0")
val values = ContentValues().apply {
    put("deck_id", newDeckId)
}
contentResolver.update(cardUri, values, null, null)
```

## Implementation Notes

1. All text fields use UTF-8 encoding
2. Timestamps are Unix epoch in seconds
3. IDs are SQLite rowids (64-bit integers)
4. HTML in cards uses AnkiDroid's rendering engine
5. Media files are stored in collection.media directory
6. Field separator (0x1F) must be preserved exactly
7. Tag spaces are replaced with underscores
8. Checksum strips HTML but preserves media references
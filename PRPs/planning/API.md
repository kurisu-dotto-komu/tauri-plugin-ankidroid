# AnkiDroid API Documentation

## Overview
The AnkiDroid API provides methods for interacting with the AnkiDroid application, allowing external apps to add flashcards, manage decks, and perform synchronization operations.

## TypeScript API Interface

```typescript
// Permission constants
export const ANKIDROID_PERMISSIONS = {
  READ_WRITE_DATABASE: "com.ichi2.anki.permission.READ_WRITE_DATABASE"
} as const;

// Intent actions
export const ANKIDROID_ACTIONS = {
  ADD_CARD: "org.openintents.action.CREATE_FLASHCARD",
  SYNC: "com.ichi2.anki.DO_SYNC",
  SEND: "android.intent.action.SEND"
} as const;

// Package names
export const ANKIDROID_PACKAGE = {
  MAIN: "com.ichi2.anki",
  DEBUG: "com.ichi2.anki.debug"
} as const;

// Content provider URIs
export const CONTENT_URI = {
  AUTHORITY: "com.ichi2.anki.flashcards",
  NOTES: "content://com.ichi2.anki.flashcards/notes",
  NOTES_V2: "content://com.ichi2.anki.flashcards/notes_v2",
  CARDS: "content://com.ichi2.anki.flashcards/cards",
  DECKS: "content://com.ichi2.anki.flashcards/decks",
  MODELS: "content://com.ichi2.anki.flashcards/models",
  SELECTED_DECK: "content://com.ichi2.anki.flashcards/selected_deck",
  SCHEDULE: "content://com.ichi2.anki.flashcards/schedule",
  REVIEW_INFO: "content://com.ichi2.anki.flashcards/review_info",
  MEDIA: "content://com.ichi2.anki.flashcards/media"
} as const;

// MIME Types
export const MIME_TYPE = {
  NOTE: "vnd.android.cursor.item/vnd.com.ichi2.anki.note",
  NOTES: "vnd.android.cursor.dir/vnd.com.ichi2.anki.note",
  CARD: "vnd.android.cursor.item/vnd.com.ichi2.anki.card",
  CARDS: "vnd.android.cursor.dir/vnd.com.ichi2.anki.card",
  MODEL: "vnd.android.cursor.item/vnd.com.ichi2.anki.model",
  MODELS: "vnd.android.cursor.dir/vnd.com.ichi2.anki.model",
  DECK: "vnd.android.cursor.item/vnd.com.ichi2.anki.deck",
  DECKS: "vnd.android.cursor.dir/vnd.com.ichi2.anki.deck"
} as const;
```

## Core API Methods

### 1. Instant-Add API (Recommended)

```typescript
interface AnkiDroidAPI {
  /**
   * Add a single note to AnkiDroid
   * @param modelId - The ID of the note model to use
   * @param deckId - The ID of the deck to add the note to
   * @param fields - Array of field values for the note
   * @param tags - Optional tags for the note
   * @returns Promise<number> - The ID of the added note, or null if failed
   */
  addNote(
    modelId: number,
    deckId: number,
    fields: string[],
    tags?: string[]
  ): Promise<number | null>;

  /**
   * Add multiple notes efficiently
   * @param notes - Array of note objects to add
   * @returns Promise<number[]> - Array of added note IDs
   */
  addNotes(notes: Note[]): Promise<number[]>;

  /**
   * Create a new deck
   * @param deckName - Name of the deck to create
   * @returns Promise<number> - The ID of the created deck
   */
  addNewDeck(deckName: string): Promise<number>;

  /**
   * Create a new basic note model
   * @param modelName - Name for the new model
   * @param fields - Array of field names
   * @param cardTemplates - Array of card templates
   * @param css - Optional CSS for card styling
   * @returns Promise<number> - The ID of the created model
   */
  addNewBasicModel(
    modelName: string,
    fields: string[],
    cardTemplates: CardTemplate[],
    css?: string
  ): Promise<number>;

  /**
   * Get AnkiDroid package name
   * @returns string | null - Package name if installed, null otherwise
   */
  getAnkiDroidPackageName(): string | null;

  /**
   * Check if AnkiDroid is installed
   * @returns boolean
   */
  isAnkiDroidInstalled(): boolean;

  /**
   * Request permissions for API access
   * @returns Promise<boolean> - True if permissions granted
   */
  requestPermissions(): Promise<boolean>;

  /**
   * Check if API permissions are granted
   * @returns boolean
   */
  hasPermissions(): boolean;
}
```

### 2. Intent-based API (Fallback)

```typescript
interface IntentAPI {
  /**
   * Add a card using ACTION_SEND intent
   * @param front - Front side of the card
   * @param back - Back side of the card
   * @param deck - Optional deck name
   */
  addCardViaIntent(
    front: string,
    back: string,
    deck?: string
  ): Promise<void>;

  /**
   * Add a card using CREATE_FLASHCARD intent (deprecated)
   * @param sourceText - Front side text
   * @param targetText - Back side text
   */
  addCardLegacy(
    sourceText: string,
    targetText: string
  ): Promise<void>;
}
```

### 3. Sync Operations

```typescript
interface SyncAPI {
  /**
   * Trigger background sync
   * Note: Minimum 5-minute interval between syncs
   * @returns Promise<void>
   */
  triggerSync(): Promise<void>;

  /**
   * Check last sync time
   * @returns Promise<Date | null>
   */
  getLastSyncTime(): Promise<Date | null>;
}
```

### 4. Database Operations (Low-Level API)

```typescript
interface DatabaseAPI {
  /**
   * Query notes from AnkiDroid database
   * @param selection - SQL WHERE clause
   * @param selectionArgs - Arguments for selection
   * @param projection - Columns to return
   * @returns Promise<Note[]>
   */
  queryNotes(
    selection?: string,
    selectionArgs?: string[],
    projection?: string[]
  ): Promise<Note[]>;

  /**
   * Query decks from AnkiDroid database
   * @returns Promise<Deck[]>
   */
  queryDecks(): Promise<Deck[]>;

  /**
   * Query note models
   * @returns Promise<Model[]>
   */
  queryModels(): Promise<Model[]>;

  /**
   * Get currently selected deck
   * @returns Promise<number> - Deck ID
   */
  getSelectedDeckId(): Promise<number>;

  /**
   * Update a note
   * @param noteId - ID of the note to update
   * @param fields - New field values
   * @returns Promise<boolean> - Success status
   */
  updateNote(
    noteId: number,
    fields: string[]
  ): Promise<boolean>;

  /**
   * Delete a note
   * @param noteId - ID of the note to delete
   * @returns Promise<boolean> - Success status
   */
  deleteNote(noteId: number): Promise<boolean>;
}

interface MediaAPI {
  /**
   * Add media file to AnkiDroid's media collection
   * @param filename - Name of the media file
   * @param data - Base64 encoded data or URL to fetch from
   * @returns Promise<string> - The actual filename used (may be modified to avoid conflicts)
   */
  addMedia(filename: string, data: string): Promise<string | null>;

  /**
   * Check if media file exists
   * @param filename - Name of the media file
   * @returns Promise<boolean>
   */
  mediaExists(filename: string): Promise<boolean>;

  /**
   * Get media file path
   * @param filename - Name of the media file
   * @returns Promise<string | null> - Full path to the media file
   */
  getMediaPath(filename: string): Promise<string | null>;
}

interface CardManagementAPI {
  /**
   * Suspend a specific card
   * @param noteId - ID of the note
   * @param cardOrd - Card ordinal (0-based index of the card template)
   * @returns Promise<boolean> - Success status
   */
  suspendCard(noteId: number, cardOrd: number): Promise<boolean>;

  /**
   * Unsuspend a specific card
   * @param noteId - ID of the note
   * @param cardOrd - Card ordinal (0-based index of the card template)
   * @returns Promise<boolean> - Success status
   */
  unsuspendCard(noteId: number, cardOrd: number): Promise<boolean>;

  /**
   * Move a card to a different deck
   * @param deckId - Target deck ID
   * @param noteId - ID of the note
   * @param cardOrd - Card ordinal (0-based index of the card template)
   * @returns Promise<boolean> - Success status
   */
  changeDeck(deckId: number, noteId: number, cardOrd: number): Promise<boolean>;

  /**
   * Bury a card until tomorrow
   * @param noteId - ID of the note
   * @param cardOrd - Card ordinal
   * @returns Promise<boolean> - Success status
   */
  buryCard(noteId: number, cardOrd: number): Promise<boolean>;
}
```

## Data Types

```typescript
interface Note {
  id?: number;
  modelId: number;
  deckId: number;
  fields: string[];
  tags?: string[];
  guid?: string;
  created?: number;
  modified?: number;
}

interface Deck {
  id: number;
  name: string;
  description?: string;
  created: number;
  modified: number;
  cardCount?: number;
  newCount?: number;
  learningCount?: number;
  reviewCount?: number;
}

interface Model {
  id: number;
  name: string;
  fields: Field[];
  templates: CardTemplate[];
  css?: string;
  type: ModelType;
}

interface Field {
  name: string;
  ord: number;
  sticky?: boolean;
  required?: boolean;
  font?: string;
  size?: number;
}

interface CardTemplate {
  name: string;
  ord: number;
  qfmt: string; // Question format
  afmt: string; // Answer format
}

enum ModelType {
  STANDARD = 0,
  CLOZE = 1
}

interface Card {
  id: number;
  noteId: number;
  deckId: number;
  ord: number; // Template ordinal
  queue: CardQueue;
  type: CardType;
  due: number;
  interval: number;
  factor: number; // Ease factor (x1000)
  reps: number;
  lapses: number;
  left?: number;
  odue?: number; // Original due
  odid?: number; // Original deck ID
  flags?: number;
  data?: string;
}

enum CardQueue {
  USER_BURIED = -3,
  SCHED_BURIED = -2,
  SUSPENDED = -1,
  NEW = 0,
  LEARNING = 1,
  REVIEW = 2,
  DAY_LEARN_RELEARN = 3,
  PREVIEW = 4
}

enum CardType {
  NEW = 0,
  LEARNING = 1,
  REVIEW = 2,
  RELEARNING = 3
}

interface DeckStats {
  newCount: number;
  learningCount: number;
  reviewCount: number;
  totalCards: number;
}

interface ReviewResult {
  cardId: number;
  noteId: number;
  ease: number;
  interval: number;
  nextReview: Date;
}

interface NoteWithCards {
  note: Note;
  cards: Card[];
}

interface ContentProviderOperation {
  type: 'insert' | 'update' | 'delete';
  uri: string;
  values?: ContentValues;
  selection?: string;
  selectionArgs?: string[];
}
```

## FlashCardsContract Constants

```typescript
export const FlashCardsContract = {
  Note: {
    _ID: "_id",
    GUID: "guid",
    MID: "mid", // Model ID
    MOD: "mod", // Modification time
    USQN: "usqn", // Update sequence number
    TAGS: "tags",
    FLDS: "flds", // Fields (separated by 0x1f)
    FLAGS: "flags",
    DATA: "data",
    SFLD: "sfld", // Sort field
    CSUM: "csum" // Checksum
  },
  
  Deck: {
    DECK_ID: "deck_id",
    DECK_NAME: "deck_name",
    DECK_DESC: "deck_desc",
    DECK_DYN: "deck_dyn", // Dynamic deck (0 or 1)
    DECK_COUNTS: "deck_counts", // New, learning, review counts
    OPTIONS: "deck_options", // Deck configuration JSON
    DECK_CONF: "deck_conf" // Deck configuration ID
  },
  
  Model: {
    MID: "mid", // Model ID
    NAME: "name",
    FLDS: "flds", // Fields JSON array
    TMPLS: "tmpls", // Templates JSON array
    CSS: "css",
    TYPE: "type", // 0 = standard, 1 = cloze
    LATEX_PRE: "latex_pre", // LaTeX preamble
    LATEX_POST: "latex_post", // LaTeX postamble
    SORT_FIELD: "sort_field", // Sort field index
    REQ: "req", // Required fields array
    DID: "did" // Default deck ID
  },
  
  Card: {
    _ID: "_id",
    NID: "nid", // Note ID
    DID: "did", // Deck ID
    ORD: "ord", // Card ordinal (template index)
    MOD: "mod", // Modification time
    TYPE: "type", // 0=new, 1=learning, 2=review, 3=relearning
    QUEUE: "queue", // -3=user buried, -2=sched buried, -1=suspended, 0=new, 1=learning, 2=review, 3=in learning, 4=preview
    DUE: "due", // Due date/time
    IVL: "ivl", // Interval in days
    FACTOR: "factor", // Ease factor (x1000)
    REPS: "reps", // Number of reviews
    LAPSES: "lapses", // Number of lapses
    LEFT: "left", // Reviews left today
    ODUE: "odue", // Original due
    ODID: "odid", // Original deck ID
    FLAGS: "flags", // Card flags
    DATA: "data", // Extra data
    QUESTION: "question", // Card question HTML
    ANSWER: "answer", // Card answer HTML
    QUESTION_SIMPLE: "question_simple", // Plain text question
    ANSWER_SIMPLE: "answer_simple", // Plain text answer
    ANSWER_PURE: "answer_pure", // Answer without question
    NEXT_REVIEW1: "next_review1", // Next review for button 1
    NEXT_REVIEW2: "next_review2", // Next review for button 2
    NEXT_REVIEW3: "next_review3", // Next review for button 3
    NEXT_REVIEW4: "next_review4" // Next review for button 4
  },
  
  ReviewInfo: {
    CARD_ORD: "card_ord",
    BUTTON: "button", // 1=again, 2=hard, 3=good, 4=easy
    TIME: "time", // Review time in milliseconds
    EASE: "ease", // New ease factor
    NOTE_ID: "note_id",
    CARD_ID: "card_id",
    NEXT_REVIEW: "next_review" // Next review time
  },
  
  Schedule: {
    CARD_ID: "card_id",
    NOTE_ID: "note_id",
    CARD_ORD: "card_ord",
    IVL: "ivl",
    EASE: "ease",
    TIME_TAKEN: "time_taken",
    ANSWER: "answer" // Answer button pressed
  }
} as const;
```

## ContentProvider Operations (Low-Level API)

```typescript
interface ContentProviderAPI {
  /**
   * Insert a note using ContentProvider
   * @param values - ContentValues with note data
   * @returns Promise<Uri> - URI of the inserted note
   */
  insertNote(values: ContentValues): Promise<string>;

  /**
   * Bulk insert notes
   * @param values - Array of ContentValues
   * @returns Promise<number> - Number of inserted notes
   */
  bulkInsertNotes(values: ContentValues[]): Promise<number>;

  /**
   * Update notes matching selection
   * @param values - New values to set
   * @param selection - WHERE clause
   * @param selectionArgs - Arguments for selection
   * @returns Promise<number> - Number of updated rows
   */
  updateNotes(
    values: ContentValues,
    selection?: string,
    selectionArgs?: string[]
  ): Promise<number>;

  /**
   * Delete notes matching selection
   * @param selection - WHERE clause
   * @param selectionArgs - Arguments for selection
   * @returns Promise<number> - Number of deleted rows
   */
  deleteNotes(
    selection?: string,
    selectionArgs?: string[]
  ): Promise<number>;

  /**
   * Query with raw SQL
   * @param uri - Content URI to query
   * @param projection - Columns to return
   * @param selection - WHERE clause
   * @param selectionArgs - Arguments for selection
   * @param sortOrder - ORDER BY clause
   * @returns Promise<Cursor> - Query results
   */
  query(
    uri: string,
    projection?: string[],
    selection?: string,
    selectionArgs?: string[],
    sortOrder?: string
  ): Promise<Cursor>;

  /**
   * Answer a card (schedule review)
   * @param cardId - ID of the card
   * @param ease - Answer button (1-4)
   * @param timeTaken - Time taken to answer in milliseconds
   * @returns Promise<boolean> - Success status
   */
  answerCard(
    cardId: number,
    ease: number,
    timeTaken: number
  ): Promise<boolean>;

  /**
   * Get next card for review
   * @param deckId - Optional deck to limit to
   * @returns Promise<Card | null> - Next card or null if none
   */
  getNextReviewCard(deckId?: number): Promise<Card | null>;

  /**
   * Reschedule a card
   * @param cardId - ID of the card
   * @param interval - New interval in days
   * @returns Promise<boolean> - Success status
   */
  rescheduleCard(
    cardId: number,
    interval: number
  ): Promise<boolean>;

  /**
   * Reset card progress
   * @param cardId - ID of the card
   * @returns Promise<boolean> - Success status
   */
  resetCard(cardId: number): Promise<boolean>;
}

interface ContentValues {
  [key: string]: string | number | boolean | null;
}

interface Cursor {
  count: number;
  position: number;
  columns: string[];
  
  moveToFirst(): boolean;
  moveToNext(): boolean;
  moveToPosition(position: number): boolean;
  getString(columnIndex: number): string | null;
  getInt(columnIndex: number): number;
  getLong(columnIndex: number): number;
  getColumnIndex(columnName: string): number;
  close(): void;
}
```

## Advanced ContentProvider Usage

### Direct Database Operations

```typescript
async function directDatabaseQuery(): Promise<Note[]> {
  const contentResolver = getContentResolver();
  const uri = CONTENT_URI.NOTES;
  
  const projection = [
    FlashCardsContract.Note._ID,
    FlashCardsContract.Note.FLDS,
    FlashCardsContract.Note.TAGS,
    FlashCardsContract.Note.MID
  ];
  
  const selection = `${FlashCardsContract.Note.TAGS} LIKE ?`;
  const selectionArgs = ["%vocabulary%"];
  const sortOrder = `${FlashCardsContract.Note.MOD} DESC`;
  
  const cursor = await contentResolver.query(
    uri,
    projection,
    selection,
    selectionArgs,
    sortOrder
  );
  
  const notes: Note[] = [];
  if (cursor.moveToFirst()) {
    do {
      const fields = cursor.getString(
        cursor.getColumnIndex(FlashCardsContract.Note.FLDS)
      ).split('\x1f'); // Fields are separated by 0x1f
      
      notes.push({
        id: cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Note._ID)),
        modelId: cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Note.MID)),
        fields: fields,
        tags: cursor.getString(cursor.getColumnIndex(FlashCardsContract.Note.TAGS))
          .split(' ')
          .filter(tag => tag.length > 0)
      });
    } while (cursor.moveToNext());
  }
  cursor.close();
  
  return notes;
}
```

### Batch Operations with Transactions

```typescript
async function batchInsertWithTransaction(notes: Note[]): Promise<void> {
  const contentResolver = getContentResolver();
  const operations: ContentProviderOperation[] = [];
  
  for (const note of notes) {
    const values = new ContentValues();
    values.put(FlashCardsContract.Note.MID, note.modelId);
    values.put(FlashCardsContract.Note.FLDS, note.fields.join('\x1f'));
    values.put(FlashCardsContract.Note.TAGS, note.tags?.join(' ') || '');
    
    operations.push(
      ContentProviderOperation
        .newInsert(CONTENT_URI.NOTES)
        .withValues(values)
        .build()
    );
  }
  
  // Apply all operations in a single transaction
  const results = await contentResolver.applyBatch(
    CONTENT_URI.AUTHORITY,
    operations
  );
  
  console.log(`Inserted ${results.length} notes in batch`);
}
```

### Review Scheduling

```typescript
async function scheduleReview(
  cardId: number,
  ease: number,
  timeTaken: number
): Promise<void> {
  const contentResolver = getContentResolver();
  const values = new ContentValues();
  
  values.put(FlashCardsContract.Schedule.CARD_ID, cardId);
  values.put(FlashCardsContract.Schedule.EASE, ease);
  values.put(FlashCardsContract.Schedule.TIME_TAKEN, timeTaken);
  values.put(FlashCardsContract.Schedule.ANSWER, ease);
  
  const uri = await contentResolver.insert(
    CONTENT_URI.SCHEDULE,
    values
  );
  
  if (!uri) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.DATABASE_ERROR,
      "Failed to schedule review"
    );
  }
}
```

### Model and Deck Management via ContentProvider

```typescript
async function createModelViaContentProvider(
  name: string,
  fields: string[],
  templates: CardTemplate[]
): Promise<number> {
  const contentResolver = getContentResolver();
  const values = new ContentValues();
  
  values.put(FlashCardsContract.Model.NAME, name);
  values.put(FlashCardsContract.Model.FLDS, JSON.stringify(
    fields.map((field, index) => ({
      name: field,
      ord: index,
      sticky: false,
      rtl: false,
      font: "Arial",
      size: 20
    }))
  ));
  values.put(FlashCardsContract.Model.TMPLS, JSON.stringify(templates));
  values.put(FlashCardsContract.Model.CSS, ".card { font-family: arial; }");
  values.put(FlashCardsContract.Model.TYPE, 0); // Standard model
  
  const uri = await contentResolver.insert(
    CONTENT_URI.MODELS,
    values
  );
  
  // Extract model ID from URI
  const modelId = parseInt(uri.getLastPathSegment());
  return modelId;
}
```

### Special URI Operations

```typescript
// Query parameters for special operations
export const URI_PARAMS = {
  // For note URIs
  WITH_CARDS: "with_cards", // Include cards when querying notes
  CARDS_ONLY: "cards_only", // Get only cards for a note
  
  // For deck URIs
  INCLUDE_STATS: "include_stats", // Include review statistics
  SELECTED_ONLY: "selected_only", // Get only selected deck
  
  // For model URIs
  WITH_DEFAULTS: "with_defaults", // Include default values
  
  // For schedule URIs
  LIMIT: "limit", // Limit number of results
  OFFSET: "offset" // Offset for pagination
} as const;

// Example: Query note with its cards
async function getNoteWithCards(noteId: number): Promise<NoteWithCards> {
  const uri = `${CONTENT_URI.NOTES}/${noteId}?${URI_PARAMS.WITH_CARDS}=true`;
  const cursor = await contentResolver.query(uri);
  // Process results...
}

// Example: Get specific card from a note
async function getCardFromNote(noteId: number, cardOrd: number): Promise<Card> {
  const uri = `${CONTENT_URI.NOTES}/${noteId}/cards/${cardOrd}`;
  const cursor = await contentResolver.query(uri);
  // Process results...
}

// Example: Query with pagination
async function getNotesWithPagination(limit: number, offset: number): Promise<Note[]> {
  const uri = `${CONTENT_URI.NOTES}?${URI_PARAMS.LIMIT}=${limit}&${URI_PARAMS.OFFSET}=${offset}`;
  const cursor = await contentResolver.query(uri);
  // Process results...
}
```

### Helper Methods for ContentProvider

```typescript
class AnkiDroidContentHelper {
  /**
   * Find model ID by name
   * @param modelName - Name of the model
   * @param numFields - Optional number of fields to match
   * @returns Model ID or null if not found
   */
  static async findModelIdByName(
    modelName: string,
    numFields?: number
  ): Promise<number | null> {
    const projection = [
      FlashCardsContract.Model.MID,
      FlashCardsContract.Model.NAME,
      FlashCardsContract.Model.FLDS
    ];
    
    const selection = `${FlashCardsContract.Model.NAME} = ?`;
    const selectionArgs = [modelName];
    
    const cursor = await contentResolver.query(
      CONTENT_URI.MODELS,
      projection,
      selection,
      selectionArgs
    );
    
    if (cursor.moveToFirst()) {
      do {
        if (numFields) {
          const fields = JSON.parse(
            cursor.getString(cursor.getColumnIndex(FlashCardsContract.Model.FLDS))
          );
          if (fields.length === numFields) {
            return cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Model.MID));
          }
        } else {
          return cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Model.MID));
        }
      } while (cursor.moveToNext());
    }
    cursor.close();
    return null;
  }
  
  /**
   * Find deck ID by name
   * @param deckName - Name of the deck
   * @returns Deck ID or null if not found
   */
  static async findDeckIdByName(deckName: string): Promise<number | null> {
    const projection = [
      FlashCardsContract.Deck.DECK_ID,
      FlashCardsContract.Deck.DECK_NAME
    ];
    
    const selection = `${FlashCardsContract.Deck.DECK_NAME} = ?`;
    const selectionArgs = [deckName];
    
    const cursor = await contentResolver.query(
      CONTENT_URI.DECKS,
      projection,
      selection,
      selectionArgs
    );
    
    if (cursor.moveToFirst()) {
      return cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Deck.DECK_ID));
    }
    cursor.close();
    return null;
  }
  
  /**
   * Get review statistics for a deck
   * @param deckId - ID of the deck
   * @returns Review statistics
   */
  static async getDeckStats(deckId: number): Promise<DeckStats> {
    const uri = `${CONTENT_URI.DECKS}/${deckId}?${URI_PARAMS.INCLUDE_STATS}=true`;
    const cursor = await contentResolver.query(uri);
    
    if (cursor.moveToFirst()) {
      const countsJson = cursor.getString(
        cursor.getColumnIndex(FlashCardsContract.Deck.DECK_COUNTS)
      );
      const counts = JSON.parse(countsJson);
      
      return {
        newCount: counts[0],
        learningCount: counts[1],
        reviewCount: counts[2],
        totalCards: counts[0] + counts[1] + counts[2]
      };
    }
    cursor.close();
    return null;
  }
  
  /**
   * Check if a note exists
   * @param noteId - ID of the note
   * @returns True if note exists
   */
  static async noteExists(noteId: number): Promise<boolean> {
    const uri = `${CONTENT_URI.NOTES}/${noteId}`;
    const cursor = await contentResolver.query(uri, ["_id"]);
    const exists = cursor.getCount() > 0;
    cursor.close();
    return exists;
  }
  
  /**
   * Get all cards for a note
   * @param noteId - ID of the note
   * @returns Array of cards
   */
  static async getCardsForNote(noteId: number): Promise<Card[]> {
    const uri = `${CONTENT_URI.NOTES}/${noteId}/cards`;
    const cursor = await contentResolver.query(uri);
    
    const cards: Card[] = [];
    if (cursor.moveToFirst()) {
      do {
        cards.push({
          id: cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Card._ID)),
          noteId: cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Card.NID)),
          deckId: cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Card.DID)),
          ord: cursor.getInt(cursor.getColumnIndex(FlashCardsContract.Card.ORD)),
          queue: cursor.getInt(cursor.getColumnIndex(FlashCardsContract.Card.QUEUE)),
          type: cursor.getInt(cursor.getColumnIndex(FlashCardsContract.Card.TYPE)),
          due: cursor.getLong(cursor.getColumnIndex(FlashCardsContract.Card.DUE)),
          interval: cursor.getInt(cursor.getColumnIndex(FlashCardsContract.Card.IVL)),
          factor: cursor.getInt(cursor.getColumnIndex(FlashCardsContract.Card.FACTOR)),
          reps: cursor.getInt(cursor.getColumnIndex(FlashCardsContract.Card.REPS)),
          lapses: cursor.getInt(cursor.getColumnIndex(FlashCardsContract.Card.LAPSES))
        });
      } while (cursor.moveToNext());
    }
    cursor.close();
    return cards;
  }
}
```

## Permission Handling

```typescript
interface PermissionManager {
  /**
   * Check if AnkiDroid API permission is granted
   * @returns boolean
   */
  checkPermission(): boolean;

  /**
   * Request AnkiDroid API permission
   * @returns Promise<PermissionResult>
   */
  requestPermission(): Promise<PermissionResult>;

  /**
   * Should show permission rationale
   * @returns boolean
   */
  shouldShowRationale(): boolean;
}

interface PermissionResult {
  granted: boolean;
  shouldRetry: boolean;
  permanentlyDenied: boolean;
}
```

## Error Handling

```typescript
enum AnkiDroidError {
  NOT_INSTALLED = "ANKIDROID_NOT_INSTALLED",
  PERMISSION_DENIED = "PERMISSION_DENIED",
  INVALID_MODEL = "INVALID_MODEL",
  INVALID_DECK = "INVALID_DECK",
  DATABASE_ERROR = "DATABASE_ERROR",
  SYNC_IN_PROGRESS = "SYNC_IN_PROGRESS",
  SYNC_COOLDOWN = "SYNC_COOLDOWN",
  UNKNOWN_ERROR = "UNKNOWN_ERROR"
}

class AnkiDroidAPIError extends Error {
  constructor(
    public code: AnkiDroidError,
    public message: string,
    public details?: any
  ) {
    super(message);
    this.name = "AnkiDroidAPIError";
  }
}
```

## Usage Examples

### Basic Card Addition

```typescript
async function addBasicCard(front: string, back: string): Promise<void> {
  const api = new AnkiDroidAPI();
  
  // Check if AnkiDroid is installed
  if (!api.isAnkiDroidInstalled()) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.NOT_INSTALLED,
      "AnkiDroid is not installed"
    );
  }
  
  // Request permissions if needed
  if (!api.hasPermissions()) {
    const granted = await api.requestPermissions();
    if (!granted) {
      // Fallback to intent method
      return api.addCardViaIntent(front, back);
    }
  }
  
  // Get default deck and model
  const deckId = await api.getSelectedDeckId();
  const models = await api.queryModels();
  const basicModel = models.find(m => m.name === "Basic");
  
  if (!basicModel) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.INVALID_MODEL,
      "Basic model not found"
    );
  }
  
  // Add the note
  const noteId = await api.addNote(
    basicModel.id,
    deckId,
    [front, back],
    []
  );
  
  if (!noteId) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.DATABASE_ERROR,
      "Failed to add note"
    );
  }
}
```

### Batch Import

```typescript
async function batchImport(cards: Array<{front: string, back: string, tags?: string[]}>): Promise<void> {
  const api = new AnkiDroidAPI();
  
  if (!api.hasPermissions()) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.PERMISSION_DENIED,
      "API permissions required for batch import"
    );
  }
  
  const deckId = await api.getSelectedDeckId();
  const models = await api.queryModels();
  const basicModel = models.find(m => m.name === "Basic");
  
  if (!basicModel) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.INVALID_MODEL,
      "Basic model not found"
    );
  }
  
  const notes: Note[] = cards.map(card => ({
    modelId: basicModel.id,
    deckId: deckId,
    fields: [card.front, card.back],
    tags: card.tags || []
  }));
  
  const addedIds = await api.addNotes(notes);
  console.log(`Added ${addedIds.length} notes successfully`);
}
```

### Custom Model Creation

```typescript
async function createLanguageModel(): Promise<number> {
  const api = new AnkiDroidAPI();
  
  const fields = ["Word", "Translation", "Example", "Audio"];
  
  const cardTemplates: CardTemplate[] = [
    {
      name: "Word → Translation",
      ord: 0,
      qfmt: "{{Word}}<br>{{Audio}}",
      afmt: "{{FrontSide}}<hr>{{Translation}}<br><br>{{Example}}"
    },
    {
      name: "Translation → Word",
      ord: 1,
      qfmt: "{{Translation}}",
      afmt: "{{FrontSide}}<hr>{{Word}}<br>{{Audio}}<br><br>{{Example}}"
    }
  ];
  
  const css = `
    .card {
      font-family: Arial, sans-serif;
      font-size: 20px;
      text-align: center;
      color: #333;
    }
    .word {
      font-size: 24px;
      font-weight: bold;
      color: #007acc;
    }
    .example {
      font-style: italic;
      color: #666;
      margin-top: 20px;
    }
  `;
  
  return await api.addNewBasicModel(
    "Language Learning",
    fields,
    cardTemplates,
    css
  );
}
```

### Media Handling

```typescript
async function addNoteWithMedia(
  front: string,
  back: string,
  imagePath: string,
  imageData: string // Base64 encoded image data
): Promise<void> {
  const api = new AnkiDroidAPI();
  const mediaApi = new MediaAPI();
  
  // Add the image to AnkiDroid's media collection
  const insertedImageName = await mediaApi.addMedia(imagePath, imageData);
  if (!insertedImageName) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.DATABASE_ERROR,
      "Failed to add media file"
    );
  }
  
  // Create HTML with the embedded image
  const frontWithImage = `${front}<br><img src="${insertedImageName}">`;
  
  // Get default deck and model
  const deckId = await api.getSelectedDeckId();
  const models = await api.queryModels();
  const basicModel = models.find(m => m.name === "Basic");
  
  // Add the note with media reference
  const noteId = await api.addNote(
    basicModel.id,
    deckId,
    [frontWithImage, back],
    []
  );
  
  console.log(`Note added with media: ${noteId}`);
}
```

### Adding Custom Fonts

```typescript
async function createModelWithCustomFont(
  fontUrl: string,
  fontName: string
): Promise<number> {
  const api = new AnkiDroidAPI();
  const mediaApi = new MediaAPI();
  
  // Add the font file to media collection
  const insertedFontName = await mediaApi.addMedia(fontName, fontUrl);
  if (!insertedFontName) {
    throw new AnkiDroidAPIError(
      AnkiDroidError.DATABASE_ERROR,
      "Failed to add font file"
    );
  }
  
  // CSS with custom font
  const css = `
    @font-face {
      font-family: 'CustomFont';
      src: url('${insertedFontName}');
    }
    .card {
      font-family: 'CustomFont', sans-serif;
      font-size: 20px;
    }
  `;
  
  const fields = ["Front", "Back"];
  const cardTemplates: CardTemplate[] = [
    {
      name: "Card 1",
      ord: 0,
      qfmt: "{{Front}}",
      afmt: "{{FrontSide}}<hr>{{Back}}"
    }
  ];
  
  return await api.addNewBasicModel(
    "Model with Custom Font",
    fields,
    cardTemplates,
    css
  );
}
```

### Card Management

```typescript
async function createSeparateReadingWritingCards(
  word: string,
  translation: string,
  example: string,
  deckBaseName: string
): Promise<void> {
  const api = new AnkiDroidAPI();
  const cardApi = new CardManagementAPI();
  
  // Create separate decks for reading and writing
  const readingDeck = `${deckBaseName}::Reading`;
  const writingDeck = `${deckBaseName}::Writing`;
  
  let readingDeckId = await api.findDeckIdByName(readingDeck);
  let writingDeckId = await api.findDeckIdByName(writingDeck);
  
  // Create decks if they don't exist
  if (!readingDeckId) {
    readingDeckId = await api.addNewDeck(readingDeck);
  }
  if (!writingDeckId) {
    writingDeckId = await api.addNewDeck(writingDeck);
  }
  
  // Add note to reading deck (creates both cards initially)
  const noteId = await api.addNote(
    modelId,
    readingDeckId,
    [word, translation, example],
    []
  );
  
  // Suspend the writing card (card ordinal 1)
  await cardApi.suspendCard(noteId, 1);
  
  // Move the writing card to the writing deck
  await cardApi.changeDeck(writingDeckId, noteId, 1);
  
  console.log(`Created separate reading/writing cards for: ${word}`);
}
```

### Sync Integration

```typescript
async function syncWithRetry(): Promise<void> {
  const api = new AnkiDroidAPI();
  
  try {
    await api.triggerSync();
    console.log("Sync triggered successfully");
  } catch (error) {
    if (error.code === AnkiDroidError.SYNC_COOLDOWN) {
      const lastSync = await api.getLastSyncTime();
      const nextSyncTime = new Date(lastSync.getTime() + 5 * 60 * 1000);
      console.log(`Sync on cooldown. Next sync available at: ${nextSyncTime}`);
    } else {
      throw error;
    }
  }
}
```

## Platform-Specific Considerations

### Android Manifest Requirements

```xml
<uses-permission android:name="com.ichi2.anki.permission.READ_WRITE_DATABASE" />

<queries>
    <package android:name="com.ichi2.anki" />
    <package android:name="com.ichi2.anki.debug" />
</queries>
```

### Gradle Dependencies

```gradle
repositories {
    maven { url "https://jitpack.io" }
}

dependencies {
    implementation 'com.github.ankidroid:Anki-Android:api-v1.1.0'
}
```

## Version Compatibility

| AnkiDroid Version | API Version | Min SDK | Features |
|-------------------|-------------|---------|----------|
| 2.5+ | v1.0.0 | 15 | Basic intent API |
| 2.8+ | v1.1.0 | 19 | Instant-Add API |
| 2.9+ | v1.1.0 | 21 | Full ContentProvider API |
| 2.14+ | v1.1.0 | 21 | Sync API |
| 2.15+ | v1.1.0 | 21 | Advanced model management |

## Best Practices

1. **Permission Handling**: Always check and request permissions before using the API
2. **Fallback Strategy**: Implement intent-based fallback for when permissions are denied
3. **Error Handling**: Implement comprehensive error handling for all API calls
4. **Batch Operations**: Use batch methods when adding multiple notes for better performance
5. **Sync Timing**: Respect the 5-minute cooldown between sync operations
6. **Model Caching**: Cache model and deck information to reduce database queries
7. **Threading**: Perform all API operations on background threads
8. **Testing**: Test on both AnkiDroid stable and debug versions

## Limitations

1. Cannot modify existing card templates programmatically
2. Cannot access review statistics directly
3. Sync operations have a 5-minute cooldown
4. Some advanced Anki features not exposed through API
5. Cannot programmatically change user preferences
6. Media files can be added but not deleted or modified through the API

## Additional Resources

- [AnkiDroid GitHub Repository](https://github.com/ankidroid/Anki-Android)
- [API Source Code](https://github.com/ankidroid/Anki-Android/tree/master/api)
- [FlashCardsContract Documentation](https://github.com/ankidroid/Anki-Android/wiki/Database-Structure)
- [React Native Wrapper](https://github.com/ankidroid/Anki-Android/tree/master/docs/marketing/localized_description/react-native)
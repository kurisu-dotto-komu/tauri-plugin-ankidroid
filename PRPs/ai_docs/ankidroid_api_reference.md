# AnkiDroid API Implementation Reference

## Critical Implementation Details

This document contains essential AnkiDroid API patterns that must be referenced during implementation. These patterns have been extracted from the official AnkiDroid API documentation and real-world implementations.

## ContentProvider URIs

```kotlin
const val AUTHORITY = "com.ichi2.anki.flashcards"
const val NOTES_URI = "content://$AUTHORITY/notes"
const val MODELS_URI = "content://$AUTHORITY/models"
const val DECKS_URI = "content://$AUTHORITY/decks"
```

## Required Manifest Configuration

```xml
<manifest>
    <!-- Required for Android 11+ to resolve content provider -->
    <queries>
        <package android:name="com.ichi2.anki" />
    </queries>
    
    <uses-permission android:name="com.ichi2.anki.permission.READ_WRITE_DATABASE" />
</manifest>
```

## Core API Operations

### 1. Check AnkiDroid Installation
```kotlin
fun isAnkiDroidInstalled(context: Context): Boolean {
    return AddContentApi.getAnkiDroidPackageName(context) != null
}
```

### 2. Add Note Pattern
```kotlin
fun addNoteToAnkiDroid(
    context: Context, 
    front: String, 
    back: String, 
    deckName: String = "Default"
): Result<Long> {
    val api = AddContentApi(context)
    
    // Get or create deck
    val deckId = api.findDeckIdByName(deckName) 
        ?: api.addNewDeck(deckName)
    
    // Get or create model  
    val modelId = api.findModelIdByName("Basic")
        ?: api.addNewBasicModel("Basic")
    
    // Add note
    val noteId = api.addNote(
        modelId,
        deckId,
        arrayOf(front, back),
        null // tags
    )
    
    return if (noteId != null) {
        Result.success(noteId)
    } else {
        Result.failure(RuntimeException("Failed to add note"))
    }
}
```

### 3. Permission Handling
```kotlin
companion object {
    private const val READ_WRITE_PERMISSION = "com.ichi2.anki.permission.READ_WRITE_DATABASE"
    private const val PERMISSION_REQUEST_CODE = 100
}

fun shouldRequestPermission(context: Context): Boolean {
    return if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
        false
    } else {
        ContextCompat.checkSelfPermission(context, READ_WRITE_PERMISSION) != 
            PackageManager.PERMISSION_GRANTED
    }
}
```

### 4. Query Decks
```kotlin
fun getAvailableDecks(context: Context): List<Deck> {
    val decks = mutableListOf<Deck>()
    val uri = Uri.parse("content://com.ichi2.anki.flashcards/decks")
    
    context.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
        while (cursor.moveToNext()) {
            val id = cursor.getLong(cursor.getColumnIndex("_id"))
            val name = cursor.getString(cursor.getColumnIndex("name"))
            decks.add(Deck(id, name))
        }
    }
    
    return decks
}
```

## Field Data Format

Notes fields are separated by the ASCII unit separator character (`\x1f`):

```kotlin
val fields = arrayOf(front, back)
val fieldsString = fields.joinToString("\u001f")
```

## Error Codes and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `SecurityException` | Permission not granted | Request `READ_WRITE_DATABASE` permission |
| `IllegalStateException` | AnkiDroid not installed | Check installation first |
| `NullPointerException` | Invalid deck/model ID | Create fallback deck/model |
| `RemoteException` | ContentProvider error | Retry with exponential backoff |

## Gradle Dependencies

```gradle
repositories {
    maven { url "https://jitpack.io" }
}

dependencies {
    implementation 'com.github.ankidroid:Anki-Android:api-v1.1.0'
}
```

## Testing Considerations

1. **Mock ContentProvider** for unit tests
2. **Robolectric** for JVM-based testing
3. **Instrumented tests** for actual AnkiDroid integration
4. **Permission scenarios** must be tested explicitly

## Critical Implementation Notes

1. **Always check AnkiDroid installation** before any API call
2. **Handle permissions** for Android 6.0+ (API 23+)
3. **Use batch operations** for multiple notes (performance)
4. **Validate all inputs** before sending to AnkiDroid
5. **Implement retry logic** for transient failures
6. **Never assume deck/model exists** - always check or create

## API Version Compatibility

- Minimum AnkiDroid version: 2.15
- API version: v1.1.0
- Target Android SDK: 34
- Minimum Android SDK: 24
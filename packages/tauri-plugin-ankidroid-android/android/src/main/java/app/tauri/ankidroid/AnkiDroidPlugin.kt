package app.tauri.ankidroid

import android.app.Activity
import android.content.ContentValues
import android.database.Cursor
import android.net.Uri
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.JSArray
import org.json.JSONObject
import org.json.JSONArray

@TauriPlugin
class AnkiDroidPlugin(private val activity: Activity): Plugin(activity) {
    
    companion object {
        const val TAG = "AnkiDroidPlugin"
        const val ANKIDROID_AUTHORITY = "com.ichi2.anki.provider"
        val NOTES_URI = Uri.parse("content://$ANKIDROID_AUTHORITY/notes")
        val CARDS_URI = Uri.parse("content://$ANKIDROID_AUTHORITY/cards")
        val DECKS_URI = Uri.parse("content://$ANKIDROID_AUTHORITY/decks")
        val MODELS_URI = Uri.parse("content://$ANKIDROID_AUTHORITY/models")
    }
    
    @Command
    fun hello(invoke: Invoke) {
        val args = invoke.parseArgs(HelloArgs::class.java)
        val ret = JSObject()
        ret.put("value", "Hello, ${args.name} from AnkiDroid plugin on Android!")
        invoke.resolve(ret)
    }
    
    @Command
    fun createCard(invoke: Invoke) {
        Log.d(TAG, "createCard called with args")
        try {
            val args = invoke.parseArgs(CreateCardArgs::class.java)
            Log.d(TAG, "Creating card - Front: ${args.front}, Back: ${args.back}, Deck: ${args.deck}")
            
            // First, get or create a deck
            val deckId = getOrCreateDeck(args.deck ?: "Default")
            Log.d(TAG, "Got deck ID: $deckId")
            if (deckId == -1L) {
                val errorObj = JSObject()
                errorObj.put("success", false)
                errorObj.put("error", "Failed to get or create deck")
                Log.e(TAG, "Failed to get or create deck")
                invoke.resolveObject(errorObj.toString())
                return
            }
            
            // Get the basic model (Note type)
            val modelId = getBasicModel()
            if (modelId == -1L) {
                val errorObj = JSObject()
                errorObj.put("success", false)
                errorObj.put("error", "Failed to get basic model")
                Log.e(TAG, "Failed to get basic model")
                invoke.resolveObject(errorObj.toString())
                return
            }
            
            // Create the note
            val noteValues = ContentValues().apply {
                put("mid", modelId) // Model ID
                put("did", deckId) // Deck ID
                put("flds", "${args.front}\u001f${args.back}") // Front and back separated by unit separator
                put("tags", args.tags ?: "")
            }
            
            Log.d(TAG, "Attempting to insert note with values: $noteValues")
            val noteUri = activity.contentResolver.insert(NOTES_URI, noteValues)
            Log.d(TAG, "Insert result URI: $noteUri")
            
            if (noteUri != null) {
                val noteId = noteUri.lastPathSegment?.toLongOrNull() ?: -1L
                Log.d(TAG, "Successfully created note with ID: $noteId")
                
                val ret = JSObject()
                ret.put("success", true)
                ret.put("noteId", noteId)
                ret.put("message", "Card created successfully")
                Log.d(TAG, "Returning success response: ${ret.toString()}")
                invoke.resolveObject(ret.toString())
            } else {
                val errorObj = JSObject()
                errorObj.put("success", false)
                errorObj.put("error", "Failed to create note - noteUri was null")
                Log.e(TAG, "Failed to create note - insert returned null")
                invoke.resolveObject(errorObj.toString())
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error creating card", e)
            val errorObj = JSObject()
            errorObj.put("success", false)
            errorObj.put("error", "Error creating card: ${e.message}")
            invoke.resolveObject(errorObj.toString())
        }
    }
    
    @Command
    fun listCards(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ListCardsArgs::class.java)
            val limit = args.limit ?: 100
            
            val cards = JSONArray()
            
            // Query notes instead of cards for better information
            val projection = arrayOf("_id", "flds", "tags", "did")
            val selection = null
            val selectionArgs = null
            val sortOrder = "_id DESC"
            
            val cursor: Cursor? = activity.contentResolver.query(
                NOTES_URI,
                projection,
                selection,
                selectionArgs,
                sortOrder
            )
            
            cursor?.use {
                var count = 0
                while (it.moveToNext() && count < limit) {
                    val cardObj = JSONObject()
                    
                    val noteId = it.getLong(it.getColumnIndexOrThrow("_id"))
                    val fields = it.getString(it.getColumnIndexOrThrow("flds"))
                    val tags = it.getString(it.getColumnIndexOrThrow("tags"))
                    val deckId = it.getLong(it.getColumnIndexOrThrow("did"))
                    
                    // Split fields (front and back)
                    val fieldParts = fields.split("\u001f")
                    val front = if (fieldParts.isNotEmpty()) fieldParts[0] else ""
                    val back = if (fieldParts.size > 1) fieldParts[1] else ""
                    
                    cardObj.put("id", noteId)
                    cardObj.put("front", front)
                    cardObj.put("back", back)
                    cardObj.put("tags", tags)
                    cardObj.put("deckId", deckId)
                    cardObj.put("deck", getDeckName(deckId))
                    
                    cards.put(cardObj)
                    count++
                }
            }
            
            if (cards.length() == 0) {
                // Return a message card if no cards found
                val messageCard = JSONObject()
                messageCard.put("id", 0)
                messageCard.put("front", "No cards found")
                messageCard.put("back", "Create some cards first using the Create Card button")
                messageCard.put("deck", "Info")
                messageCard.put("tags", "")
                cards.put(messageCard)
            }
            
            // Convert JSONArray to string and resolve
            invoke.resolveObject(cards.toString())
        } catch (e: Exception) {
            Log.e(TAG, "Error listing cards", e)
            
            // Return error as a card for better UI display
            val errorCards = JSONArray()
            val errorCard = JSONObject()
            errorCard.put("id", -1)
            errorCard.put("front", "Error reading cards")
            errorCard.put("back", "Error: ${e.message}")
            errorCard.put("deck", "Error")
            errorCard.put("tags", "error")
            errorCards.put(errorCard)
            
            // Return error cards as string
            invoke.resolveObject(errorCards.toString())
        }
    }
    
    @Command
    fun getDecks(invoke: Invoke) {
        try {
            val decks = JSONArray()
            
            val projection = arrayOf("did", "name")
            val cursor: Cursor? = activity.contentResolver.query(
                DECKS_URI,
                projection,
                null,
                null,
                "name ASC"
            )
            
            cursor?.use {
                while (it.moveToNext()) {
                    val deckObj = JSONObject()
                    deckObj.put("id", it.getLong(it.getColumnIndexOrThrow("did")))
                    deckObj.put("name", it.getString(it.getColumnIndexOrThrow("name")))
                    decks.put(deckObj)
                }
            }
            
            // Return decks array as string
            invoke.resolveObject(decks.toString())
        } catch (e: Exception) {
            Log.e(TAG, "Error getting decks", e)
            val errorArray = JSONArray()
            // Return empty array on error
            invoke.resolveObject(errorArray.toString())
        }
    }
    
    private fun getOrCreateDeck(deckName: String): Long {
        Log.d(TAG, "getOrCreateDeck called for: $deckName")
        try {
            // First try to find existing deck
            val projection = arrayOf("did")
            val selection = "name = ?"
            val selectionArgs = arrayOf(deckName)
            
            val cursor: Cursor? = activity.contentResolver.query(
                DECKS_URI,
                projection,
                selection,
                selectionArgs,
                null
            )
            
            cursor?.use {
                if (it.moveToFirst()) {
                    return it.getLong(it.getColumnIndexOrThrow("did"))
                }
            }
            
            // If not found, create new deck
            val values = ContentValues().apply {
                put("name", deckName)
            }
            
            val uri = activity.contentResolver.insert(DECKS_URI, values)
            return uri?.lastPathSegment?.toLongOrNull() ?: -1L
            
        } catch (e: Exception) {
            Log.e(TAG, "Error getting or creating deck", e)
            // Return default deck ID
            return 1L
        }
    }
    
    private fun getDeckName(deckId: Long): String {
        try {
            val projection = arrayOf("name")
            val selection = "did = ?"
            val selectionArgs = arrayOf(deckId.toString())
            
            val cursor: Cursor? = activity.contentResolver.query(
                DECKS_URI,
                projection,
                selection,
                selectionArgs,
                null
            )
            
            cursor?.use {
                if (it.moveToFirst()) {
                    return it.getString(it.getColumnIndexOrThrow("name"))
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting deck name", e)
        }
        return "Default"
    }
    
    private fun getBasicModel(): Long {
        Log.d(TAG, "getBasicModel called")
        try {
            // Query for Basic model
            val projection = arrayOf("mid", "name")
            val cursor: Cursor? = activity.contentResolver.query(
                MODELS_URI,
                projection,
                null,
                null,
                null
            )
            Log.d(TAG, "Models query returned cursor: ${cursor != null}")
            
            cursor?.use {
                while (it.moveToNext()) {
                    val name = it.getString(it.getColumnIndexOrThrow("name"))
                    if (name.contains("Basic", ignoreCase = true)) {
                        return it.getLong(it.getColumnIndexOrThrow("mid"))
                    }
                }
                // If no Basic model found, return first model
                if (it.moveToFirst()) {
                    return it.getLong(it.getColumnIndexOrThrow("mid"))
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error getting model", e)
        }
        // Return a default model ID
        return 1L
    }
    
    @Command
    fun updateCard(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(UpdateCardArgs::class.java)
            
            // Update the note
            val values = ContentValues().apply {
                // AnkiDroid stores front and back in a single field separated by \u001f
                put("flds", "${args.front}\u001f${args.back}")
                
                // Update tags if provided
                args.tags?.let { put("tags", it) }
                
                // Update deck if provided
                args.deck?.let { 
                    val deckId = getOrCreateDeck(it)
                    if (deckId > 0) {
                        put("did", deckId)
                    }
                }
            }
            
            val uri = Uri.withAppendedPath(NOTES_URI, args.noteId.toString())
            val updatedRows = activity.contentResolver.update(uri, values, null, null)
            
            val result = JSObject()
            if (updatedRows > 0) {
                result.put("success", true)
                result.put("noteId", args.noteId)
                result.put("message", "Card updated successfully")
            } else {
                result.put("success", false)
                result.put("error", "Failed to update card - card not found")
            }
            invoke.resolve(result)
            
        } catch (e: Exception) {
            Log.e(TAG, "Error updating card", e)
            val errorObj = JSObject()
            errorObj.put("success", false)
            errorObj.put("error", "Error updating card: ${e.message}")
            invoke.resolve(errorObj)
        }
    }
    
    @Command
    fun deleteCard(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(DeleteCardArgs::class.java)
            
            // Delete the note
            val uri = Uri.withAppendedPath(NOTES_URI, args.noteId.toString())
            val deletedRows = activity.contentResolver.delete(uri, null, null)
            
            val result = JSObject()
            if (deletedRows > 0) {
                result.put("success", true)
                result.put("noteId", args.noteId)
                result.put("message", "Card deleted successfully")
            } else {
                result.put("success", false)
                result.put("error", "Failed to delete card - card not found")
            }
            invoke.resolve(result)
            
        } catch (e: Exception) {
            Log.e(TAG, "Error deleting card", e)
            val errorObj = JSObject()
            errorObj.put("success", false)
            errorObj.put("error", "Error deleting card: ${e.message}")
            invoke.resolve(errorObj)
        }
    }
}

data class HelloArgs(val name: String)
data class CreateCardArgs(
    val front: String,
    val back: String,
    val deck: String? = "Default",
    val tags: String? = ""
)
data class ListCardsArgs(val limit: Int? = 100)
data class UpdateCardArgs(
    val noteId: Long,
    val front: String,
    val back: String,
    val deck: String? = null,
    val tags: String? = null
)
data class DeleteCardArgs(val noteId: Long)
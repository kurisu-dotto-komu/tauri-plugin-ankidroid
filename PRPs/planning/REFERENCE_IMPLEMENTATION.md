This file contains two files from an old version of this plugin project that works with Capcaitor.

This is just for a reference to help integrate with the anki.api -- we are not using Capacitor anymore.

```java
// apps/capacitor/android/app/src/main/java/com/majikai/app/AnkiDroidPlugin.java
package com.majikai.app;

import com.getcapacitor.JSObject;
import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;

import java.util.*;

import org.json.JSONException;
import com.getcapacitor.JSArray;

import com.ichi2.anki.api.AddContentApi;

@CapacitorPlugin(name = "AnkiDroid")
public class AnkiDroidPlugin extends Plugin {

    private String[] toStringArray(JSArray array) throws JSONException {
        String[] stringArray = new String[array.length()];
        for (int i = 0; i < array.length(); i++) {
            stringArray[i] = array.getString(i);
        }
        return stringArray;
    }

    @PluginMethod()
    public void checkModelExists(PluginCall call) {
        String modelName = call.getString("modelName");
        AnkiDroidHelper mAnkiDroid = new AnkiDroidHelper(getContext());
        Long modelId = mAnkiDroid.findModelIdByName(modelName, 0);
        Boolean exists = modelId != null;
        JSObject ret = new JSObject();
        ret.put("exists", exists);
        call.resolve(ret);
        return;
    }

    @PluginMethod()
    public void createModel(PluginCall call) {
        // TODO this will not throw if it already exists, we should check for that
        // TODO only call this from readyCheck, so we don't accidentally call it?

        AnkiDroidHelper mAnkiDroid = new AnkiDroidHelper(getContext());
        AddContentApi api = mAnkiDroid.getApi();
        String fontUrl = call.getString("fontUrl");
        String fontName = call.getString("fontName");
        String insertedFont = mAnkiDroid.addMedia(fontName, fontUrl);
        if (insertedFont == null) {
            call.reject("Error inserting font");
            return;
        }
        // replace the css font filename with the inserted font name
        String css = call.getString("css").replace(fontName, insertedFont);
        String modelName = call.getString("name");
        try {
            Long modelId = api.addNewCustomModel(
                    call.getString("name"),
                    // field names
                    toStringArray(call.getArray("fields")),
                    // array of names for the card templates
                    toStringArray(call.getArray("cards")),
                    // array of formatting strings for the question side of
                    toStringArray(call.getArray("qfmt")),
                    // afmt array of formatting strings for the answer side of
                    toStringArray(call.getArray("afmt")),
                    // le css
                    css,
                    null,
                    null);

            JSObject ret = new JSObject();
            ret.put("success", true);
            ret.put("modelId", modelId);
            call.resolve(ret);
        } catch (JSONException e) {
            // Handle the exception, e.g., return an error response
            call.reject("JSONException occurred");
        }
    }

    @PluginMethod()
    public void createNote(PluginCall call) {
        AnkiDroidHelper mAnkiDroid = new AnkiDroidHelper(getContext());
        AddContentApi api = mAnkiDroid.getApi();

        String modelName = call.getString("modelName");
        Long modelId = mAnkiDroid.findModelIdByName(modelName, 0);
        if (modelId == null) {
            call.reject("Model not found");
            return;
        }

        String deckName = call.getString("deckName");
        // throw if not set
        if (deckName == null) {
            call.reject("Deck name not provided");
            return;
        }

        String readingDeck = deckName + "::Reading";
        String writingDeck = deckName + "::Writing";

        Long readingDeckId = mAnkiDroid.findDeckIdByName(readingDeck);
        Long writingDeckId = mAnkiDroid.findDeckIdByName(writingDeck);

        // create the decks if they don't exist
        if (readingDeckId == null) {
            readingDeckId = api.addNewDeck(readingDeck);
        }
        if (writingDeckId == null) {
            writingDeckId = api.addNewDeck(writingDeck);
        }

        try {
            // if there's an image, insert it first and replace it in the image field
            String[] fields = toStringArray(call.getArray("fields"));
            List<String> fieldsList = new ArrayList<>(Arrays.asList(fields));

            String image = call.getString("image");
            String imageData = call.getString("imageData");
            if (image != null && imageData != null) {
                String insertedImage = mAnkiDroid.addMedia(image, imageData);
                if (insertedImage == null) {
                    call.reject("Error inserting image");
                    return;
                }
                fieldsList.add(insertedImage);
            } else {
                // add an empty string to the fields array
                fieldsList.add("");
            }
            fields = fieldsList.toArray(new String[0]);

            Long noteId = api.addNote(modelId, readingDeckId, fields, null);
            // suspend the writing card
            mAnkiDroid.suspendCard(noteId, 1);
            // move the writing card to the writing deck
            mAnkiDroid.changeDeck(writingDeckId, noteId, 1);

            JSObject ret = new JSObject();
            ret.put("success", true);
            ret.put("noteId", noteId);
            call.resolve(ret);
        } catch (JSONException e) {
            // Handle the exception, e.g., return an error response
            call.reject("JSONException occurred");
        }
    }

    @PluginMethod()
    public void checkConnection(PluginCall call) {
        // TODO figure out a good way of checking this properly

        if (!AnkiDroidHelper.isApiAvailable(getContext())) {
            call.reject("AnkiDroid API not available");
            return;
        }
        JSObject ret = new JSObject();
        ret.put("success", true);
        call.resolve(ret);
    }

}
```

```java
// apps/capacitor/android/app/src/main/java/com/majikai/app/AnkiDroidHelper.java
package com.majikai.app;

import android.content.Intent;

import android.app.Activity;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Context;
import android.content.pm.PackageManager;
import android.content.SharedPreferences;
import android.net.Uri;
import android.os.Build;
import android.util.Base64;
import android.util.SparseArray;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;
import androidx.core.content.FileProvider;
import com.ichi2.anki.api.AddContentApi;
import com.ichi2.anki.api.NoteInfo;
import com.ichi2.anki.FlashCardsContract;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URI;
import java.net.URL;
import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;
import java.util.ListIterator;
import java.util.Map;
import java.util.Set;

import static com.ichi2.anki.api.AddContentApi.READ_WRITE_PERMISSION;

public class AnkiDroidHelper {
  private static final String DECK_REF_DB = "com.ichi2.anki.api.decks";
  private static final String MODEL_REF_DB = "com.ichi2.anki.api.models";

  private AddContentApi mApi;
  private Context mContext;

  public AnkiDroidHelper(Context context) {
    mContext = context.getApplicationContext();
    mApi = new AddContentApi(mContext);
  }

  public AddContentApi getApi() {
    return mApi;
  }

  /**
   * Whether or not the API is available to use.
   * The API could be unavailable if AnkiDroid is not installed or the user
   * explicitly disabled the API
   *
   * @return true if the API is available to use
   */
  public static boolean isApiAvailable(Context context) {
    return AddContentApi.getAnkiDroidPackageName(context) != null;
  }

  /**
   * Whether or not we should request full access to the AnkiDroid API
   */
  public boolean shouldRequestPermission() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
      return false;
    }
    return ContextCompat.checkSelfPermission(mContext, READ_WRITE_PERMISSION) != PackageManager.PERMISSION_GRANTED;
  }

  /**
   * Request permission from the user to access the AnkiDroid API (for SDK 23+)
   *
   * @param callbackActivity An Activity which implements
   *                         onRequestPermissionsResult()
   * @param callbackCode     The callback code to be used in
   *                         onRequestPermissionsResult()
   */
  public void requestPermission(Activity callbackActivity, int callbackCode) {
    ActivityCompat.requestPermissions(callbackActivity, new String[] { READ_WRITE_PERMISSION }, callbackCode);
  }

  /**
   * Save a mapping from deckName to getDeckId in the SharedPreferences
   */
  public void storeDeckReference(String deckName, long deckId) {
    final SharedPreferences decksDb = mContext.getSharedPreferences(DECK_REF_DB, Context.MODE_PRIVATE);
    decksDb.edit().putLong(deckName, deckId).apply();
  }

  /**
   * Save a mapping from modelName to modelId in the SharedPreferences
   */
  public void storeModelReference(String modelName, long modelId) {
    final SharedPreferences modelsDb = mContext.getSharedPreferences(MODEL_REF_DB, Context.MODE_PRIVATE);
    modelsDb.edit().putLong(modelName, modelId).apply();
  }

  /**
   * Remove the duplicates from a list of note fields and tags
   *
   * @param fields  List of fields to remove duplicates from
   * @param tags    List of tags to remove duplicates from
   * @param modelId ID of model to search for duplicates on
   */
  public void removeDuplicates(LinkedList<String[]> fields, LinkedList<Set<String>> tags, long modelId) {
    // Build a list of the duplicate keys (first fields) and find all notes that
    // have a match with each key
    List<String> keys = new ArrayList<>(fields.size());
    for (String[] f : fields) {
      keys.add(f[0]);
    }
    SparseArray<List<NoteInfo>> duplicateNotes = getApi().findDuplicateNotes(modelId, keys);
    // Do some sanity checks
    if (tags.size() != fields.size()) {
      throw new IllegalStateException("List of tags must be the same length as the list of fields");
    }
    if (duplicateNotes == null || duplicateNotes.size() == 0 || fields.size() == 0 || tags.size() == 0) {
      return;
    }
    if (duplicateNotes.keyAt(duplicateNotes.size() - 1) >= fields.size()) {
      throw new IllegalStateException("The array of duplicates goes outside the bounds of the original lists");
    }
    // Iterate through the fields and tags LinkedLists, removing those that had a
    // duplicate
    ListIterator<String[]> fieldIterator = fields.listIterator();
    ListIterator<Set<String>> tagIterator = tags.listIterator();
    int listIndex = -1;
    for (int i = 0; i < duplicateNotes.size(); i++) {
      int duplicateIndex = duplicateNotes.keyAt(i);
      while (listIndex < duplicateIndex) {
        fieldIterator.next();
        tagIterator.next();
        listIndex++;
      }
      fieldIterator.remove();
      tagIterator.remove();
    }
  }

  /**
   * Try to find the given model by name, accounting for renaming of the model:
   * If there's a model with this modelName that is known to have previously been
   * created (by this app)
   * and the corresponding model ID exists and has the required number of fields
   * then return that ID (even though it may have since been renamed)
   * If there's a model from #getModelList with modelName and required number of
   * fields then return its ID
   * Otherwise return null
   *
   * @param modelName the name of the model to find
   * @param numFields the minimum number of fields the model is required to have
   * @return the model ID or null if something went wrong
   */
  public Long findModelIdByName(String modelName, int numFields) {
    SharedPreferences modelsDb = mContext.getSharedPreferences(MODEL_REF_DB, Context.MODE_PRIVATE);
    long prefsModelId = modelsDb.getLong(modelName, -1L);
    // if we have a reference saved to modelName and it exists and has at least
    // numFields then return it
    if ((prefsModelId != -1L)
        && (mApi.getModelName(prefsModelId) != null)
        && (mApi.getFieldList(prefsModelId) != null)
        && (mApi.getFieldList(prefsModelId).length >= numFields)) { // could potentially have been renamed
      return prefsModelId;
    }
    Map<Long, String> modelList = mApi.getModelList(numFields);
    if (modelList != null) {
      for (Map.Entry<Long, String> entry : modelList.entrySet()) {
        if (entry.getValue().equals(modelName)) {
          return entry.getKey(); // first model wins
        }
      }
    }
    // model no longer exists (by name nor old id), the number of fields was
    // reduced, or API error
    return null;
  }

  /**
   * Try to find the given deck by name, accounting for potential renaming of the
   * deck by the user as follows:
   * If there's a deck with deckName then return it's ID
   * If there's no deck with deckName, but a ref to deckName is stored in
   * SharedPreferences, and that deck exist in
   * AnkiDroid (i.e. it was renamed), then use that deck.Note: this deck will not
   * be found if your app is re-installed
   * If there's no reference to deckName anywhere then return null
   *
   * @param deckName the name of the deck to find
   * @return the did of the deck in Anki
   */
  public Long findDeckIdByName(String deckName) {
    SharedPreferences decksDb = mContext.getSharedPreferences(DECK_REF_DB, Context.MODE_PRIVATE);
    // Look for deckName in the deck list
    Long did = getDeckId(deckName);
    if (did != null) {
      // If the deck was found then return it's id
      return did;
    } else {
      // Otherwise try to check if we have a reference to a deck that was renamed and
      // return that
      did = decksDb.getLong(deckName, -1);
      if (did != -1 && mApi.getDeckName(did) != null) {
        return did;
      } else {
        // If the deck really doesn't exist then return null
        return null;
      }
    }
  }

  /**
   * Get the ID of the deck which matches the name
   *
   * @param deckName Exact name of deck (note: deck names are unique in Anki)
   * @return the ID of the deck that has given name, or null if no deck was found
   *         or API error
   */
  private Long getDeckId(String deckName) {
    Map<Long, String> deckList = mApi.getDeckList();
    if (deckList != null) {
      for (Map.Entry<Long, String> entry : deckList.entrySet()) {
        if (entry.getValue().equalsIgnoreCase(deckName)) {
          return entry.getKey();
        }
      }
    }
    return null;
  }

  /**
   * Suspend a card in AnkiDroid.
   *
   * @param noteId  The ID of the note.
   * @param cardOrd The ordinal of the card within the note.
   * @return true if the operation was successful, false otherwise.
   */
  public boolean suspendCard(long noteId, int cardOrd) {
    try {
      ContentResolver cr = mContext.getContentResolver();
      Uri reviewInfoUri = FlashCardsContract.ReviewInfo.CONTENT_URI;
      ContentValues values = new ContentValues();

      values.put(FlashCardsContract.ReviewInfo.NOTE_ID, noteId);
      values.put(FlashCardsContract.ReviewInfo.CARD_ORD, cardOrd);
      values.put(FlashCardsContract.ReviewInfo.SUSPEND, 1);

      int updateCount = cr.update(reviewInfoUri, values, null, null);
      return updateCount > 0;
    } catch (Exception e) {
      e.printStackTrace();
      return false;
    }
  }

  /**
   * Move a card to a different deck in AnkiDroid.
   *
   * @param deckId  The ID of the deck to move the card to.
   * @param noteId  The ID of the note.
   * @param cardOrd The ordinal of the card within the note.
   * @return true if the operation was successful, false otherwise.
   */
  public boolean changeDeck(long deckId, long noteId, int cardOrd) {
    try {
      ContentResolver cr = mContext.getContentResolver();
      Uri cardsUri = Uri.withAppendedPath(FlashCardsContract.Note.CONTENT_URI, Long.toString(noteId));
      cardsUri = Uri.withAppendedPath(cardsUri, "cards");
      Uri specificCardUri = Uri.withAppendedPath(cardsUri, Integer.toString(cardOrd));

      ContentValues values = new ContentValues();
      values.put(FlashCardsContract.Card.DECK_ID, deckId);

      int updateCount = cr.update(specificCardUri, values, null, null);
      return updateCount > 0;
    } catch (Exception e) {
      e.printStackTrace();
      return false;
    }
  }

  /**
   * Add media to AnkiDroid's media collection.
   *
   * @param fileName The name of the file to add.
   * @param path     The path of the file, which can be a URL or a data URI.
   * @return true if the operation was successful, false otherwise.
   */
  public String addMedia(String fileName, String path) {
    try {
      byte[] mediaData;
      if (path.startsWith("http://") || path.startsWith("https://")) {
        // Handle URL, perhaps there's some easier way than this?
        URL url = new URL(path);
        HttpURLConnection connection = (HttpURLConnection) url.openConnection();
        InputStream inputStream = connection.getInputStream();
        ByteArrayOutputStream outputStream = new ByteArrayOutputStream();
        byte[] buffer = new byte[1024];
        int bytesRead;
        while ((bytesRead = inputStream.read(buffer)) != -1) {
          outputStream.write(buffer, 0, bytesRead);
        }
        mediaData = outputStream.toByteArray();
        inputStream.close();
        outputStream.close();
      } else if (path.startsWith("data:")) {
        // Handle data URI
        String base64Data = path.split(",")[1];
        mediaData = Base64.decode(base64Data, Base64.DEFAULT);
      } else {
        // Invalid path
        return null;
      }

      // Save the mediaData to a file
      File file = new File(mContext.getCacheDir(), fileName);
      FileOutputStream fos = new FileOutputStream(file);
      fos.write(mediaData);
      fos.close();

      // Get URI for file from FileProvider
      Uri fileUri = FileProvider.getUriForFile(mContext, "com.majikai.app.fileprovider", file);

      // Grant permission to AnkiDroid to access the file
      mContext.grantUriPermission("com.ichi2.anki", fileUri, Intent.FLAG_GRANT_READ_URI_PERMISSION);

      // Add media to AnkiDroid using ContentResolver
      ContentResolver cr = mContext.getContentResolver();
      ContentValues values = new ContentValues();
      values.put(FlashCardsContract.AnkiMedia.FILE_URI, fileUri.toString());
      values.put(FlashCardsContract.AnkiMedia.PREFERRED_NAME, fileName);
      Uri insertedFile = cr.insert(FlashCardsContract.AnkiMedia.CONTENT_URI, values);

      if (insertedFile == null) {
        return null;
      }

      return insertedFile.toString().substring(8);

    } catch (Exception e) {
      e.printStackTrace();
      return null;
    }
  }

}
```

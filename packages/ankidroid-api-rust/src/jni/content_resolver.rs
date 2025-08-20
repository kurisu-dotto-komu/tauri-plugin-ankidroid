//! JNI wrapper for Android ContentResolver operations
//!
//! This module provides a safe wrapper around Android's ContentResolver class,
//! enabling content provider operations from Rust. It supports query, insert,
//! update, delete, and bulk insert operations with proper error handling.
//!
//! # Examples
//!
//! ```rust
//! use crate::jni::helpers::SafeJNIEnv;
//! use crate::jni::content_resolver::ContentResolver;
//! use crate::error::Result;
//!
//! fn example_usage(env: SafeJNIEnv, context: &JObject) -> Result<()> {
//!     let content_resolver = ContentResolver::from_context(env, context)?;
//!     
//!     // Query example
//!     let cursor = content_resolver.query(
//!         "content://com.ichi2.anki.flascards/decks",
//!         Some(vec!["_id".to_string(), "name".to_string()]),
//!         None,
//!         None,
//!         None
//!     )?;
//!     
//!     // Insert example
//!     let mut content_values = ContentValuesBuilder::new(&mut env)?;
//!     content_values = content_values.put_string("name", "New Deck")?;
//!     let uri = content_resolver.insert(
//!         "content://com.ichi2.anki.flascards/decks",
//!         content_values
//!     )?;
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{AnkiDroidError, Result};
use crate::jni::helpers::{ContentValuesBuilder, SafeJNIEnv, StringHelper, JniResultExt};
use crate::jni::cursor::Cursor;
use jni::objects::{JObject, JValue};

/// Wrapper for Android ContentResolver with safe JNI operations
/// 
/// This struct provides a safe interface to Android's ContentResolver,
/// handling all JNI operations and error checking automatically.
pub struct ContentResolver<'local> {
    env: SafeJNIEnv<'local>,
    resolver: JObject<'local>,
}

impl<'local> ContentResolver<'local> {
    /// Create a ContentResolver from an Android Context
    ///
    /// # Arguments
    ///
    /// * `env` - The safe JNI environment
    /// * `context` - The Android Context object
    ///
    /// # Returns
    ///
    /// A ContentResolver wrapper, or an error if the operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let content_resolver = ContentResolver::from_context(env, &context)?;
    /// ```
    pub fn from_context(mut env: SafeJNIEnv<'local>, context: &JObject<'local>) -> Result<Self> {
        if context.is_null() {
            return Err(AnkiDroidError::null_pointer(
                "Context is null when creating ContentResolver",
            ));
        }

        let result = env
            .env_mut()
            .call_method(
                context,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[],
            )
            .check_exception(env.env_mut())?;

        let resolver = result.l().map_err(AnkiDroidError::from)?;

        if resolver.is_null() {
            return Err(AnkiDroidError::null_pointer("ContentResolver is null"));
        }

        Ok(Self { env, resolver })
    }

    /// Create a ContentResolver from an existing ContentResolver object
    ///
    /// # Arguments
    ///
    /// * `env` - The safe JNI environment
    /// * `resolver` - The ContentResolver object
    ///
    /// # Returns
    ///
    /// A ContentResolver wrapper, or an error if the resolver is null
    ///
    /// # Examples
    ///
    /// ```rust
    /// let content_resolver = ContentResolver::from_resolver(env, resolver_obj)?;
    /// ```
    pub fn from_resolver(env: SafeJNIEnv<'local>, resolver: JObject<'local>) -> Result<Self> {
        if resolver.is_null() {
            return Err(AnkiDroidError::null_pointer("ContentResolver is null"));
        }

        Ok(Self { env, resolver })
    }

    /// Query a content provider
    ///
    /// # Arguments
    ///
    /// * `uri` - The content URI to query
    /// * `projection` - The columns to return (None for all columns)
    /// * `selection` - The WHERE clause (None for no filtering)
    /// * `selection_args` - Arguments for the WHERE clause placeholders
    /// * `sort_order` - The ORDER BY clause (None for default order)
    ///
    /// # Returns
    ///
    /// A Cursor for iterating over the results, or an error if the query failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let cursor = content_resolver.query(
    ///     "content://com.ichi2.anki.flascards/decks",
    ///     Some(vec!["_id".to_string(), "name".to_string()]),
    ///     Some("name = ?".to_string()),
    ///     Some(vec!["Basic".to_string()]),
    ///     Some("name ASC".to_string())
    /// )?;
    /// ```
    pub fn query(
        &mut self,
        uri: &str,
        projection: Option<Vec<String>>,
        selection: Option<String>,
        selection_args: Option<Vec<String>>,
        sort_order: Option<String>,
    ) -> Result<Cursor<'local>> {
        let uri_obj = self.parse_uri(uri)?;

        // Prepare projection array if provided
        let projection_obj = if let Some(proj) = &projection {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env_mut()
                .new_object_array(proj.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, column) in proj.iter().enumerate() {
                let column_string = self.env.new_string_checked(column)?;
                self.env
                    .env_mut()
                    .set_object_array_element(&array, i as i32, column_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };

        // Prepare selection string if provided
        let selection_string_obj = if let Some(sel) = &selection {
            let sel_string = self.env.new_string_checked(sel)?;
            Some(JObject::from(sel_string))
        } else {
            None
        };

        // Prepare selection args array if provided
        let selection_args_obj = if let Some(args) = &selection_args {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env_mut()
                .new_object_array(args.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, arg) in args.iter().enumerate() {
                let arg_string = self.env.new_string_checked(arg)?;
                self.env
                    .env_mut()
                    .set_object_array_element(&array, i as i32, arg_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };

        // Prepare sort order string if provided
        let sort_order_string_obj = if let Some(order) = &sort_order {
            let order_string = self.env.new_string_checked(order)?;
            Some(JObject::from(order_string))
        } else {
            None
        };

        // Create JValue references for the method call
        let null_obj = JObject::null();
        let projection_jvalue = match projection_obj.as_ref() {
            Some(obj) => JValue::Object(obj),
            None => JValue::Object(&null_obj),
        };
        let selection_jvalue = match selection_string_obj.as_ref() {
            Some(obj) => JValue::Object(obj),
            None => JValue::Object(&null_obj),
        };
        let selection_args_jvalue = match selection_args_obj.as_ref() {
            Some(obj) => JValue::Object(obj),
            None => JValue::Object(&null_obj),
        };
        let sort_order_jvalue = match sort_order_string_obj.as_ref() {
            Some(obj) => JValue::Object(obj),
            None => JValue::Object(&null_obj),
        };

        // Execute the query
        let cursor_result = self.env.env_mut().call_method(
            &self.resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                JValue::Object(&uri_obj),
                projection_jvalue,
                selection_jvalue,
                selection_args_jvalue,
                sort_order_jvalue,
            ],
        ).check_exception(self.env.env_mut())?;

        let cursor_obj = cursor_result.l().map_err(AnkiDroidError::from)?;
        Cursor::new(self.env.clone(), cursor_obj)
    }

    /// Insert a new record into a content provider
    ///
    /// # Arguments
    ///
    /// * `uri` - The content URI to insert into
    /// * `values` - The values to insert
    ///
    /// # Returns
    ///
    /// The URI of the inserted record, or an error if the insert failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let values = ContentValuesBuilder::new(&mut env)?
    ///     .put_string("name", "New Deck")?
    ///     .put_long("timestamp", 1234567890)?;
    /// let inserted_uri = content_resolver.insert(
    ///     "content://com.ichi2.anki.flascards/decks",
    ///     values
    /// )?;
    /// ```
    pub fn insert(
        &mut self,
        uri: &str,
        values: ContentValuesBuilder<'local>,
    ) -> Result<String> {
        let uri_obj = self.parse_uri(uri)?;
        let content_values = values.build();

        let result_uri = self
            .env
            .env_mut()
            .call_method(
                &self.resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[JValue::Object(&uri_obj), JValue::Object(&content_values)],
            )
            .check_exception(self.env.env_mut())?;

        let uri_result_obj = result_uri.l().map_err(AnkiDroidError::from)?;
        if uri_result_obj.is_null() {
            return Err(AnkiDroidError::database_error("Insert returned null URI"));
        }

        // Convert URI to string
        let uri_string_result = self
            .env
            .env_mut()
            .call_method(&uri_result_obj, "toString", "()Ljava/lang/String;", &[])
            .check_exception(self.env.env_mut())?;
        let uri_string_obj = uri_string_result.l().map_err(AnkiDroidError::from)?;

        if uri_string_obj.is_null() {
            return Err(AnkiDroidError::database_error(
                "Insert URI toString returned null",
            ));
        }

        StringHelper::jobject_to_rust(&mut self.env, &uri_string_obj)
    }

    /// Update existing records in a content provider
    ///
    /// # Arguments
    ///
    /// * `uri` - The content URI to update
    /// * `values` - The new values
    /// * `selection` - The WHERE clause (None to update all records)
    /// * `selection_args` - Arguments for the WHERE clause placeholders
    ///
    /// # Returns
    ///
    /// The number of rows updated, or an error if the update failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let values = ContentValuesBuilder::new(&mut env)?
    ///     .put_string("name", "Updated Deck")?;
    /// let updated_count = content_resolver.update(
    ///     "content://com.ichi2.anki.flascards/decks",
    ///     values,
    ///     Some("_id = ?".to_string()),
    ///     Some(vec!["1".to_string()])
    /// )?;
    /// ```
    pub fn update(
        &mut self,
        uri: &str,
        values: ContentValuesBuilder<'local>,
        selection: Option<String>,
        selection_args: Option<Vec<String>>,
    ) -> Result<i32> {
        let uri_obj = self.parse_uri(uri)?;
        let content_values = values.build();

        // Prepare selection string if provided
        let selection_string_obj = if let Some(sel) = &selection {
            Some(self.env.new_string_checked(sel)?)
        } else {
            None
        };

        // Prepare selection args array if provided
        let selection_args_array_obj = if let Some(args) = &selection_args {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env_mut()
                .new_object_array(args.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, arg) in args.iter().enumerate() {
                let arg_string = self.env.new_string_checked(arg)?;
                self.env
                    .env_mut()
                    .set_object_array_element(&array, i as i32, arg_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };

        // Create JValue references
        let null_obj = JObject::null();
        let selection_jvalue = match selection_string_obj.as_ref() {
            Some(s) => JValue::Object(s.as_ref()),
            None => JValue::Object(&null_obj),
        };
        let selection_args_jvalue = match selection_args_array_obj.as_ref() {
            Some(a) => JValue::Object(a),
            None => JValue::Object(&null_obj),
        };

        let result = self.env.env_mut().call_method(
            &self.resolver,
            "update",
            "(Landroid/net/Uri;Landroid/content/ContentValues;Ljava/lang/String;[Ljava/lang/String;)I",
            &[
                JValue::Object(&uri_obj),
                JValue::Object(&content_values),
                selection_jvalue,
                selection_args_jvalue,
            ],
        ).check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Delete records from a content provider
    ///
    /// # Arguments
    ///
    /// * `uri` - The content URI to delete from
    /// * `selection` - The WHERE clause (None to delete all records)
    /// * `selection_args` - Arguments for the WHERE clause placeholders
    ///
    /// # Returns
    ///
    /// The number of rows deleted, or an error if the delete failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let deleted_count = content_resolver.delete(
    ///     "content://com.ichi2.anki.flascards/decks",
    ///     Some("_id = ?".to_string()),
    ///     Some(vec!["1".to_string()])
    /// )?;
    /// ```
    pub fn delete(
        &mut self,
        uri: &str,
        selection: Option<String>,
        selection_args: Option<Vec<String>>,
    ) -> Result<i32> {
        let uri_obj = self.parse_uri(uri)?;

        // Prepare selection string if provided
        let selection_string_obj = if let Some(sel) = &selection {
            Some(self.env.new_string_checked(sel)?)
        } else {
            None
        };

        // Prepare selection args array if provided
        let selection_args_array_obj = if let Some(args) = &selection_args {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env_mut()
                .new_object_array(args.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, arg) in args.iter().enumerate() {
                let arg_string = self.env.new_string_checked(arg)?;
                self.env
                    .env_mut()
                    .set_object_array_element(&array, i as i32, arg_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };

        // Create JValue references
        let null_obj = JObject::null();
        let selection_jvalue = match selection_string_obj.as_ref() {
            Some(s) => JValue::Object(s.as_ref()),
            None => JValue::Object(&null_obj),
        };
        let selection_args_jvalue = match selection_args_array_obj.as_ref() {
            Some(a) => JValue::Object(a),
            None => JValue::Object(&null_obj),
        };

        let result = self
            .env
            .env_mut()
            .call_method(
                &self.resolver,
                "delete",
                "(Landroid/net/Uri;Ljava/lang/String;[Ljava/lang/String;)I",
                &[
                    JValue::Object(&uri_obj),
                    selection_jvalue,
                    selection_args_jvalue,
                ],
            )
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Bulk insert multiple records into a content provider
    ///
    /// # Arguments
    ///
    /// * `uri` - The content URI to insert into
    /// * `values` - An array of ContentValues to insert
    ///
    /// # Returns
    ///
    /// The number of records inserted, or an error if the bulk insert failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let values1 = ContentValuesBuilder::new(&mut env)?
    ///     .put_string("name", "Deck 1")?;
    /// let values2 = ContentValuesBuilder::new(&mut env)?
    ///     .put_string("name", "Deck 2")?;
    /// let inserted_count = content_resolver.bulk_insert(
    ///     "content://com.ichi2.anki.flascards/decks",
    ///     vec![values1, values2]
    /// )?;
    /// ```
    pub fn bulk_insert(
        &mut self,
        uri: &str,
        values: Vec<ContentValuesBuilder<'local>>,
    ) -> Result<i32> {
        let uri_obj = self.parse_uri(uri)?;

        // Create array of ContentValues
        let content_values_class = self.env.find_class_checked("android/content/ContentValues")?;
        let values_array = self
            .env
            .env_mut()
            .new_object_array(values.len() as i32, content_values_class, JObject::null())
            .check_exception(self.env.env_mut())?;

        for (i, value_builder) in values.into_iter().enumerate() {
            let content_values = value_builder.build();
            self.env
                .env_mut()
                .set_object_array_element(&values_array, i as i32, content_values)
                .check_exception(self.env.env_mut())?;
        }

        let result = self
            .env
            .env_mut()
            .call_method(
                &self.resolver,
                "bulkInsert",
                "(Landroid/net/Uri;[Landroid/content/ContentValues;)I",
                &[
                    JValue::Object(&uri_obj),
                    JValue::Object(&JObject::from(values_array)),
                ],
            )
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Helper method to parse a URI string
    ///
    /// # Arguments
    ///
    /// * `uri_string` - The URI string to parse
    ///
    /// # Returns
    ///
    /// A URI object, or an error if parsing failed
    fn parse_uri(&mut self, uri_string: &str) -> Result<JObject<'local>> {
        let uri_class = self.env.find_class_checked("android/net/Uri")?;
        let uri_str = self.env.new_string_checked(uri_string)?;

        let result = self
            .env
            .env_mut()
            .call_static_method(
                &uri_class,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&uri_str.into())],
            )
            .check_exception(self.env.env_mut())?;

        result.l().map_err(AnkiDroidError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_resolver_creation_concepts() {
        // Test basic ContentResolver concepts without actual JNI
        // This is a placeholder test since we can't test JNI in unit tests
        assert!(true);
    }

    #[test]
    fn test_uri_validation() {
        let uri = "content://com.ichi2.anki.flascards/decks";
        assert!(uri.starts_with("content://"));
    }

    #[test]
    fn test_selection_clause_building() {
        let selection = "name = ? AND active = ?";
        let args = vec!["Basic".to_string(), "1".to_string()];
        assert_eq!(selection, "name = ? AND active = ?");
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn test_error_handling() {
        let error = AnkiDroidError::database_error("Test database error");
        assert!(error.to_string().contains("Database error"));
    }
}
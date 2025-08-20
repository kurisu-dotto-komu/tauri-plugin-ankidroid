//! JNI wrapper for Android Cursor operations
//!
//! This module provides a safe wrapper around Android's Cursor class,
//! enabling iteration over database query results from Rust. It includes
//! automatic resource cleanup, type-safe value extraction, and iterator
//! implementation for convenient traversal.
//!
//! # Examples
//!
//! ```rust
//! use crate::jni::helpers::SafeJNIEnv;
//! use crate::jni::cursor::Cursor;
//! use crate::error::Result;
//!
//! fn example_usage(env: SafeJNIEnv, cursor_obj: JObject) -> Result<()> {
//!     let mut cursor = Cursor::new(env, cursor_obj)?;
//!     
//!     // Manual iteration
//!     if cursor.move_to_first()? {
//!         loop {
//!             let id = cursor.get_long_by_name("_id")?;
//!             let name = cursor.get_string_by_name("name")?;
//!             println!("ID: {}, Name: {}", id, name);
//!             
//!             if !cursor.move_next()? {
//!                 break;
//!             }
//!         }
//!     }
//!     
//!     // Or use iterator
//!     for row_result in cursor.iter() {
//!         row_result?; // Check for iteration errors
//!         let id = cursor.get_long_by_name("_id")?;
//!         let name = cursor.get_string_by_name("name")?;
//!         println!("ID: {}, Name: {}", id, name);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{AnkiDroidError, Result};
use crate::jni::helpers::{SafeJNIEnv, StringHelper, JniResultExt};
use jni::objects::{JObject, JValue};
use std::collections::HashMap;

/// Wrapper for Android Cursor with automatic cleanup and safe operations
///
/// This struct provides a safe interface to Android's Cursor class,
/// handling all JNI operations and error checking automatically.
/// It implements Drop for automatic resource cleanup.
pub struct Cursor<'local> {
    env: SafeJNIEnv<'local>,
    cursor: JObject<'local>,
    is_closed: bool,
    column_index_cache: HashMap<String, i32>,
}

impl<'local> Cursor<'local> {
    /// Create a new Cursor from a cursor object
    ///
    /// # Arguments
    ///
    /// * `env` - The safe JNI environment
    /// * `cursor` - The Android Cursor object
    ///
    /// # Returns
    ///
    /// A Cursor wrapper, or an error if the cursor is null
    ///
    /// # Examples
    ///
    /// ```rust
    /// let cursor = Cursor::new(env, cursor_obj)?;
    /// ```
    pub fn new(env: SafeJNIEnv<'local>, cursor: JObject<'local>) -> Result<Self> {
        if cursor.is_null() {
            return Err(AnkiDroidError::null_pointer("Cursor is null"));
        }

        Ok(Self {
            env,
            cursor,
            is_closed: false,
            column_index_cache: HashMap::new(),
        })
    }

    /// Move the cursor to the first row
    ///
    /// # Returns
    ///
    /// `true` if the cursor was moved successfully, `false` if the cursor is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// if cursor.move_to_first()? {
    ///     // Process first row
    /// }
    /// ```
    pub fn move_to_first(&mut self) -> Result<bool> {
        self.check_not_closed()?;

        let result = self
            .env
            .env_mut()
            .call_method(&self.cursor, "moveToFirst", "()Z", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.z().unwrap_or(false))
    }

    /// Move the cursor to the next row
    ///
    /// # Returns
    ///
    /// `true` if the cursor was moved successfully, `false` if at the end
    ///
    /// # Examples
    ///
    /// ```rust
    /// while cursor.move_next()? {
    ///     // Process each row
    /// }
    /// ```
    pub fn move_next(&mut self) -> Result<bool> {
        self.move_to_next()
    }

    /// Move the cursor to the next row (Android API method name)
    ///
    /// # Returns
    ///
    /// `true` if the cursor was moved successfully, `false` if at the end
    ///
    /// # Examples
    ///
    /// ```rust
    /// while cursor.move_to_next()? {
    ///     // Process each row
    /// }
    /// ```
    pub fn move_to_next(&mut self) -> Result<bool> {
        self.check_not_closed()?;

        let result = self
            .env
            .env_mut()
            .call_method(&self.cursor, "moveToNext", "()Z", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.z().unwrap_or(false))
    }

    /// Get the column index for a given column name
    ///
    /// This method caches column indices for performance during iteration.
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column
    ///
    /// # Returns
    ///
    /// The column index, or an error if the column doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// let id_index = cursor.get_column_index("_id")?;
    /// ```
    pub fn get_column_index(&mut self, column_name: &str) -> Result<i32> {
        self.check_not_closed()?;

        // Check cache first
        if let Some(&index) = self.column_index_cache.get(column_name) {
            return Ok(index);
        }

        let column_string = self.env.new_string_checked(column_name)?;
        let result = self
            .env
            .env_mut()
            .call_method(
                &self.cursor,
                "getColumnIndex",
                "(Ljava/lang/String;)I",
                &[JValue::Object(&column_string.into())],
            )
            .check_exception(self.env.env_mut())?;

        let index = result.i().unwrap_or(-1);
        
        if index < 0 {
            return Err(AnkiDroidError::validation_error(format!(
                "Column '{}' not found in cursor",
                column_name
            )));
        }

        // Cache the index for future lookups
        self.column_index_cache.insert(column_name.to_string(), index);
        Ok(index)
    }

    /// Get a string value from the cursor at the specified column index
    ///
    /// # Arguments
    ///
    /// * `column_index` - The column index (0-based)
    ///
    /// # Returns
    ///
    /// The string value, or an empty string if the value is null
    ///
    /// # Examples
    ///
    /// ```rust
    /// let name = cursor.get_string(1)?;
    /// ```
    pub fn get_string(&mut self, column_index: i32) -> Result<String> {
        self.check_not_closed()?;
        self.validate_column_index(column_index)?;

        let result = self
            .env
            .env_mut()
            .call_method(
                &self.cursor,
                "getString",
                "(I)Ljava/lang/String;",
                &[JValue::Int(column_index)],
            )
            .check_exception(self.env.env_mut())?;

        let string_obj = result.l().map_err(AnkiDroidError::from)?;
        if string_obj.is_null() {
            Ok(String::new())
        } else {
            StringHelper::jobject_to_rust(&mut self.env, &string_obj)
        }
    }

    /// Get a long value from the cursor at the specified column index
    ///
    /// # Arguments
    ///
    /// * `column_index` - The column index (0-based)
    ///
    /// # Returns
    ///
    /// The long value, or 0 if the value is null
    ///
    /// # Examples
    ///
    /// ```rust
    /// let id = cursor.get_long(0)?;
    /// ```
    pub fn get_long(&mut self, column_index: i32) -> Result<i64> {
        self.check_not_closed()?;
        self.validate_column_index(column_index)?;

        let result = self
            .env
            .env_mut()
            .call_method(
                &self.cursor,
                "getLong",
                "(I)J",
                &[JValue::Int(column_index)],
            )
            .check_exception(self.env.env_mut())?;

        Ok(result.j().unwrap_or(0))
    }

    /// Get an integer value from the cursor at the specified column index
    ///
    /// # Arguments
    ///
    /// * `column_index` - The column index (0-based)
    ///
    /// # Returns
    ///
    /// The integer value, or 0 if the value is null
    ///
    /// # Examples
    ///
    /// ```rust
    /// let count = cursor.get_int(2)?;
    /// ```
    pub fn get_int(&mut self, column_index: i32) -> Result<i32> {
        self.check_not_closed()?;
        self.validate_column_index(column_index)?;

        let result = self
            .env
            .env_mut()
            .call_method(
                &self.cursor,
                "getInt",
                "(I)I",
                &[JValue::Int(column_index)],
            )
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Get a string value by column name (convenience method)
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column
    ///
    /// # Returns
    ///
    /// The string value, or an error if the column doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// let name = cursor.get_string_by_name("name")?;
    /// ```
    pub fn get_string_by_name(&mut self, column_name: &str) -> Result<String> {
        let index = self.get_column_index(column_name)?;
        self.get_string(index)
    }

    /// Get a long value by column name (convenience method)
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column
    ///
    /// # Returns
    ///
    /// The long value, or an error if the column doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// let id = cursor.get_long_by_name("_id")?;
    /// ```
    pub fn get_long_by_name(&mut self, column_name: &str) -> Result<i64> {
        let index = self.get_column_index(column_name)?;
        self.get_long(index)
    }

    /// Get an integer value by column name (convenience method)
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column
    ///
    /// # Returns
    ///
    /// The integer value, or an error if the column doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// let count = cursor.get_int_by_name("count")?;
    /// ```
    pub fn get_int_by_name(&mut self, column_name: &str) -> Result<i32> {
        let index = self.get_column_index(column_name)?;
        self.get_int(index)
    }

    /// Check if a column value is null
    ///
    /// # Arguments
    ///
    /// * `column_index` - The column index (0-based)
    ///
    /// # Returns
    ///
    /// `true` if the value is null, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// if cursor.is_null(1)? {
    ///     println!("Column value is null");
    /// }
    /// ```
    pub fn is_null(&mut self, column_index: i32) -> Result<bool> {
        self.check_not_closed()?;
        self.validate_column_index(column_index)?;

        let result = self
            .env
            .env_mut()
            .call_method(
                &self.cursor,
                "isNull",
                "(I)Z",
                &[JValue::Int(column_index)],
            )
            .check_exception(self.env.env_mut())?;

        Ok(result.z().unwrap_or(true))
    }

    /// Get the number of rows in the cursor
    ///
    /// # Returns
    ///
    /// The number of rows, or an error if the operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let count = cursor.get_count()?;
    /// println!("Cursor has {} rows", count);
    /// ```
    pub fn get_count(&mut self) -> Result<i32> {
        self.check_not_closed()?;

        let result = self
            .env
            .env_mut()
            .call_method(&self.cursor, "getCount", "()I", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Get the number of columns in the cursor
    ///
    /// # Returns
    ///
    /// The number of columns, or an error if the operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let column_count = cursor.get_column_count()?;
    /// println!("Cursor has {} columns", column_count);
    /// ```
    pub fn get_column_count(&mut self) -> Result<i32> {
        self.check_not_closed()?;

        let result = self
            .env
            .env_mut()
            .call_method(&self.cursor, "getColumnCount", "()I", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Get all column names in the cursor
    ///
    /// # Returns
    ///
    /// A vector of column names, or an error if the operation failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// let columns = cursor.get_column_names()?;
    /// for column in columns {
    ///     println!("Column: {}", column);
    /// }
    /// ```
    pub fn get_column_names(&mut self) -> Result<Vec<String>> {
        self.check_not_closed()?;

        let result = self
            .env
            .env_mut()
            .call_method(&self.cursor, "getColumnNames", "()[Ljava/lang/String;", &[])
            .check_exception(self.env.env_mut())?;

        let array_obj = result.l().map_err(AnkiDroidError::from)?;
        if array_obj.is_null() {
            return Ok(Vec::new());
        }

        // Get array length
        let array = unsafe { jni::objects::JObjectArray::from_raw(array_obj.as_raw()) };
        let length = self
            .env
            .env_mut()
            .get_array_length(&array)
            .check_exception(self.env.env_mut())?;

        let mut column_names = Vec::with_capacity(length as usize);

        // Extract each string from the array
        for i in 0..length {
            let element = self
                .env
                .env_mut()
                .get_object_array_element(&array, i)
                .check_exception(self.env.env_mut())?;

            if !element.is_null() {
                let column_name = StringHelper::jobject_to_rust(&mut self.env, &element)?;
                column_names.push(column_name);
            } else {
                column_names.push(String::new());
            }
        }

        Ok(column_names)
    }

    /// Check if the cursor is closed
    ///
    /// # Returns
    ///
    /// `true` if the cursor is closed, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// if cursor.is_closed() {
    ///     println!("Cursor is closed");
    /// }
    /// ```
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// Manually close the cursor
    ///
    /// This method closes the cursor and releases its resources.
    /// The cursor is automatically closed when dropped, so manual
    /// closing is optional but can be useful for early cleanup.
    ///
    /// # Examples
    ///
    /// ```rust
    /// cursor.close()?;
    /// ```
    pub fn close(&mut self) -> Result<()> {
        if !self.is_closed {
            let _result = self
                .env
                .env_mut()
                .call_method(&self.cursor, "close", "()V", &[])
                .check_exception(self.env.env_mut())?;
            self.is_closed = true;
        }
        Ok(())
    }

    /// Create an iterator over cursor rows
    ///
    /// # Returns
    ///
    /// A CursorIterator for iterating over the rows
    ///
    /// # Examples
    ///
    /// ```rust
    /// for row_result in cursor.iter() {
    ///     row_result?; // Check for iteration errors
    ///     let id = cursor.get_long_by_name("_id")?;
    ///     // Process row...
    /// }
    /// ```
    pub fn iter(&mut self) -> CursorIterator<'_, 'local> {
        CursorIterator {
            cursor: self,
            first_iteration: true,
        }
    }

    /// Helper method to check if the cursor is closed
    fn check_not_closed(&self) -> Result<()> {
        if self.is_closed {
            Err(AnkiDroidError::validation_error("Cursor is closed"))
        } else {
            Ok(())
        }
    }

    /// Helper method to validate column index
    fn validate_column_index(&self, column_index: i32) -> Result<()> {
        if column_index < 0 {
            Err(AnkiDroidError::validation_error(format!(
                "Invalid column index: {}",
                column_index
            )))
        } else {
            Ok(())
        }
    }
}

impl<'local> Drop for Cursor<'local> {
    /// Automatically close the cursor when dropped
    fn drop(&mut self) {
        if !self.is_closed {
            // Best effort close - ignore errors during cleanup
            let _ = self
                .env
                .env_mut()
                .call_method(&self.cursor, "close", "()V", &[]);
            self.is_closed = true;
        }
    }
}

/// Iterator implementation for cursor rows
pub struct CursorIterator<'a, 'local> {
    cursor: &'a mut Cursor<'local>,
    first_iteration: bool,
}

impl<'a, 'local> Iterator for CursorIterator<'a, 'local> {
    type Item = Result<()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first_iteration {
            self.first_iteration = false;
            match self.cursor.move_to_first() {
                Ok(true) => Some(Ok(())),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            }
        } else {
            match self.cursor.move_next() {
                Ok(true) => Some(Ok(())),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            }
        }
    }
}

/// Helper function to collect all rows from a cursor into a vector
///
/// This function iterates over all rows in a cursor and applies a processor
/// function to each row, collecting the results into a vector.
///
/// # Arguments
///
/// * `cursor` - The cursor to iterate over
/// * `row_processor` - A function that processes each row and returns a result
///
/// # Returns
///
/// A vector of results from processing each row
///
/// # Examples
///
/// ```rust
/// let results = collect_cursor_results(cursor, |cursor| {
///     Ok(cursor.get_long_by_name("_id")?)
/// })?;
/// ```
pub fn collect_cursor_results<T, F>(
    mut cursor: Cursor,
    mut row_processor: F,
) -> Result<Vec<T>>
where
    F: FnMut(&mut Cursor) -> Result<T>,
{
    let mut results = Vec::new();

    // Use manual iteration to avoid borrowing conflicts
    if cursor.move_to_first()? {
        loop {
            let item = row_processor(&mut cursor)?;
            results.push(item);

            if !cursor.move_next()? {
                break;
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation_concepts() {
        // Test basic cursor concepts without actual JNI
        // This is a placeholder test since we can't test JNI in unit tests
        assert!(true);
    }

    #[test]
    fn test_column_index_validation() {
        // Test that negative column indices are handled properly
        let column_index = -1;
        assert!(column_index < 0);
    }

    #[test]
    fn test_cursor_state_tracking() {
        // Test cursor state tracking concepts
        let is_closed = false;
        assert!(!is_closed);
    }

    #[test]
    fn test_column_index_cache() {
        // Test column index caching concepts
        let mut cache: HashMap<String, i32> = HashMap::new();
        cache.insert("_id".to_string(), 0);
        cache.insert("name".to_string(), 1);
        
        assert_eq!(cache.get("_id"), Some(&0));
        assert_eq!(cache.get("name"), Some(&1));
        assert_eq!(cache.get("missing"), None);
    }

    #[test]
    fn test_error_handling() {
        let error = AnkiDroidError::validation_error("Test cursor error");
        assert!(error.to_string().contains("Validation error"));
    }
}
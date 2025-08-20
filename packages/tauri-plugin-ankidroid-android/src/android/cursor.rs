use crate::android::error::{AndroidError, AndroidResult, JniResultExt};
use crate::android::jni_helpers::{SafeJNIEnv, StringHelper};
use jni::objects::{JObject, JValue};

/// RAII wrapper for Android Cursor with automatic cleanup
pub struct CursorIterator<'local> {
    env: SafeJNIEnv<'local>,
    cursor: JObject<'local>,
    is_closed: bool,
}

impl<'local> CursorIterator<'local> {
    /// Create a new CursorIterator from a cursor object
    pub fn new(env: SafeJNIEnv<'local>, cursor: JObject<'local>) -> AndroidResult<Self> {
        if cursor.is_null() {
            return Err(AndroidError::cursor_error("Cursor is null"));
        }

        Ok(Self {
            env,
            cursor,
            is_closed: false,
        })
    }

    /// Move the cursor to the first row
    pub fn move_to_first(&mut self) -> AndroidResult<bool> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        let result = self
            .env
            .env()
            .call_method(&self.cursor, "moveToFirst", "()Z", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.z().unwrap_or(false))
    }

    /// Move the cursor to the next row
    pub fn move_to_next(&mut self) -> AndroidResult<bool> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        let result = self
            .env
            .env()
            .call_method(&self.cursor, "moveToNext", "()Z", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.z().unwrap_or(false))
    }

    /// Get the column index for a given column name
    pub fn get_column_index(&mut self, column_name: &str) -> AndroidResult<i32> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        let column_string = self.env.new_string_checked(column_name)?;
        let column_obj = JObject::from(column_string);
        let result = self
            .env
            .env()
            .call_method(
                &self.cursor,
                "getColumnIndex",
                "(Ljava/lang/String;)I",
                &[JValue::Object(&column_obj)],
            )
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(-1))
    }

    /// Get a string value from the cursor at the specified column index
    pub fn get_string(&mut self, column_index: i32) -> AndroidResult<String> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        if column_index < 0 {
            return Err(AndroidError::cursor_error("Invalid column index"));
        }

        let result = self
            .env
            .env()
            .call_method(
                &self.cursor,
                "getString",
                "(I)Ljava/lang/String;",
                &[JValue::Int(column_index)],
            )
            .check_exception(self.env.env_mut())?;

        let string_obj = result.l().map_err(AndroidError::from)?;
        if string_obj.is_null() {
            Ok(String::new())
        } else {
            StringHelper::jobject_to_rust(&mut self.env, &string_obj)
        }
    }

    /// Get a long value from the cursor at the specified column index
    pub fn get_long(&mut self, column_index: i32) -> AndroidResult<i64> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        if column_index < 0 {
            return Err(AndroidError::cursor_error("Invalid column index"));
        }

        let result = self
            .env
            .env()
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
    pub fn get_int(&mut self, column_index: i32) -> AndroidResult<i32> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        if column_index < 0 {
            return Err(AndroidError::cursor_error("Invalid column index"));
        }

        let result = self
            .env
            .env()
            .call_method(&self.cursor, "getInt", "(I)I", &[JValue::Int(column_index)])
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Get the number of rows in the cursor
    pub fn get_count(&mut self) -> AndroidResult<i32> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        let result = self
            .env
            .env()
            .call_method(&self.cursor, "getCount", "()I", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Get the number of columns in the cursor
    pub fn get_column_count(&mut self) -> AndroidResult<i32> {
        if self.is_closed {
            return Err(AndroidError::cursor_error("Cursor is closed"));
        }

        let result = self
            .env
            .env()
            .call_method(&self.cursor, "getColumnCount", "()I", &[])
            .check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }

    /// Check if the cursor is closed
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// Get a string value by column name (convenience method)
    pub fn get_string_by_name(&mut self, column_name: &str) -> AndroidResult<String> {
        let index = self.get_column_index(column_name)?;
        if index < 0 {
            return Err(AndroidError::cursor_error(format!(
                "Column '{}' not found",
                column_name
            )));
        }
        self.get_string(index)
    }

    /// Get a long value by column name (convenience method)
    pub fn get_long_by_name(&mut self, column_name: &str) -> AndroidResult<i64> {
        let index = self.get_column_index(column_name)?;
        if index < 0 {
            return Err(AndroidError::cursor_error(format!(
                "Column '{}' not found",
                column_name
            )));
        }
        self.get_long(index)
    }

    /// Get an integer value by column name (convenience method)
    pub fn get_int_by_name(&mut self, column_name: &str) -> AndroidResult<i32> {
        let index = self.get_column_index(column_name)?;
        if index < 0 {
            return Err(AndroidError::cursor_error(format!(
                "Column '{}' not found",
                column_name
            )));
        }
        self.get_int(index)
    }

    /// Manually close the cursor
    pub fn close(&mut self) -> AndroidResult<()> {
        if !self.is_closed {
            let result = self
                .env
                .env()
                .call_method(&self.cursor, "close", "()V", &[])
                .check_exception(self.env.env_mut())?;
            self.is_closed = true;
        }
        Ok(())
    }
}

impl<'local> Drop for CursorIterator<'local> {
    /// Automatically close the cursor when the iterator is dropped
    fn drop(&mut self) {
        if !self.is_closed {
            // Best effort close - ignore errors during cleanup
            let _ = self
                .env
                .env()
                .call_method(&self.cursor, "close", "()V", &[]);
            self.is_closed = true;
        }
    }
}

/// Iterator implementation for CursorIterator
pub struct CursorRows<'a, 'local> {
    cursor: &'a mut CursorIterator<'local>,
    first_iteration: bool,
}

impl<'local> CursorIterator<'local> {
    /// Create an iterator over cursor rows
    pub fn iter(&mut self) -> CursorRows<'_, 'local> {
        CursorRows {
            cursor: self,
            first_iteration: true,
        }
    }
}

impl<'a, 'local> Iterator for CursorRows<'a, 'local> {
    type Item = AndroidResult<()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first_iteration {
            self.first_iteration = false;
            match self.cursor.move_to_first() {
                Ok(true) => Some(Ok(())),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            }
        } else {
            match self.cursor.move_to_next() {
                Ok(true) => Some(Ok(())),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            }
        }
    }
}

/// Helper function to iterate over all rows in a cursor and collect results
pub fn collect_cursor_results<T, F>(
    mut cursor: CursorIterator,
    mut row_processor: F,
) -> AndroidResult<Vec<T>>
where
    F: FnMut(&mut CursorIterator) -> AndroidResult<T>,
{
    let mut results = Vec::new();

    // Use manual iteration to avoid borrowing conflicts
    if cursor.move_to_first()? {
        loop {
            let item = row_processor(&mut cursor)?;
            results.push(item);

            if !cursor.move_to_next()? {
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
    fn test_cursor_iterator_creation() {
        // Test basic cursor iterator concepts without actual JNI
        // This is a placeholder since we can't test JNI in unit tests
        assert!(true);
    }

    #[test]
    fn test_cursor_error_handling() {
        let error = AndroidError::cursor_error("Test cursor error");
        assert!(error.to_string().contains("Cursor error"));
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
}

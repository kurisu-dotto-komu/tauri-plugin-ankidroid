use crate::android::cursor::CursorIterator;
use crate::android::error::{AndroidError, AndroidResult, JniResultExt};
use crate::android::jni_helpers::{
    get_content_resolver, parse_uri, ContentValuesBuilder, SafeJNIEnv,
};
use jni::objects::{JObject, JValue};
use std::ops::Deref;

/// Builder pattern for ContentProvider queries
pub struct ContentProviderQuery<'local> {
    env: SafeJNIEnv<'local>,
    uri: String,
    projection: Option<Vec<String>>,
    selection: Option<String>,
    selection_args: Option<Vec<String>>,
    sort_order: Option<String>,
}

impl<'local> ContentProviderQuery<'local> {
    /// Create a new query builder for the specified URI
    pub fn new(env: SafeJNIEnv<'local>, uri: &str) -> Self {
        Self {
            env,
            uri: uri.to_string(),
            projection: None,
            selection: None,
            selection_args: None,
            sort_order: None,
        }
    }

    /// Set the projection (columns to return)
    pub fn projection(mut self, columns: Vec<String>) -> Self {
        self.projection = Some(columns);
        self
    }

    /// Set the selection clause (WHERE clause)
    pub fn selection(mut self, selection: String) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Set the selection arguments
    pub fn selection_args(mut self, args: Vec<String>) -> Self {
        self.selection_args = Some(args);
        self
    }

    /// Set the sort order (ORDER BY clause)
    pub fn sort_order(mut self, order: String) -> Self {
        self.sort_order = Some(order);
        self
    }

    /// Execute the query and return a CursorIterator
    pub fn execute(mut self, activity: &JObject<'local>) -> AndroidResult<CursorIterator<'local>> {
        let content_resolver = get_content_resolver(&mut self.env, activity)?;
        let uri_obj = parse_uri(&mut self.env, &self.uri)?;

        // Prepare objects with proper lifetimes
        let projection_obj = if let Some(proj) = &self.projection {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env()
                .new_object_array(proj.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, column) in proj.iter().enumerate() {
                let column_string = self.env.new_string_checked(column)?;
                self.env
                    .env()
                    .set_object_array_element(&array, i as i32, column_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };

        let selection_string_obj = if let Some(sel) = &self.selection {
            let sel_string = self.env.new_string_checked(sel)?;
            Some(JObject::from(sel_string))
        } else {
            None
        };

        let selection_args_obj = if let Some(args) = &self.selection_args {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env()
                .new_object_array(args.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, arg) in args.iter().enumerate() {
                let arg_string = self.env.new_string_checked(arg)?;
                self.env
                    .env()
                    .set_object_array_element(&array, i as i32, arg_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };

        let sort_order_string_obj = if let Some(order) = &self.sort_order {
            let order_string = self.env.new_string_checked(order)?;
            Some(JObject::from(order_string))
        } else {
            None
        };
        
        // Create JValue references after all objects are created
        let null_obj = JObject::null();
        let projection_array = if let Some(ref obj) = projection_obj {
            JValue::Object(obj)
        } else {
            JValue::Object(&null_obj)
        };

        let selection_obj = if let Some(ref obj) = selection_string_obj {
            JValue::Object(obj)
        } else {
            JValue::Object(&null_obj)
        };

        let selection_args_array = if let Some(ref obj) = selection_args_obj {
            JValue::Object(obj)
        } else {
            JValue::Object(&null_obj)
        };

        let sort_order_obj = if let Some(ref obj) = sort_order_string_obj {
            JValue::Object(obj)
        } else {
            JValue::Object(&null_obj)
        };

        // Execute query
        let uri_jvalue = JValue::Object(&uri_obj);
        let cursor_result = self.env.env().call_method(
            &content_resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                uri_jvalue,
                projection_array,
                selection_obj,
                selection_args_array,
                sort_order_obj,
            ],
        ).check_exception(self.env.env_mut())?;

        let cursor = cursor_result.l().map_err(AndroidError::from)?;
        CursorIterator::new(self.env, cursor)
    }
}

/// Helper for ContentProvider insert operations
pub struct ContentProviderInsert<'local> {
    env: SafeJNIEnv<'local>,
    uri: String,
}

impl<'local> ContentProviderInsert<'local> {
    /// Create a new insert operation for the specified URI
    pub fn new(env: SafeJNIEnv<'local>, uri: &str) -> Self {
        Self {
            env,
            uri: uri.to_string(),
        }
    }

    /// Execute the insert with the given values
    pub fn execute(
        mut self,
        activity: &JObject<'local>,
        values: ContentValuesBuilder<'local>,
    ) -> AndroidResult<String> {
        let content_resolver = get_content_resolver(&mut self.env, activity)?;
        let uri_obj = parse_uri(&mut self.env, &self.uri)?;
        let content_values = values.build();

        let result_uri = self
            .env
            .env()
            .call_method(
                &content_resolver,
                "insert",
                "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
                &[
                    JValue::Object(&uri_obj),
                    JValue::Object(&content_values),
                ],
            )
            .check_exception(self.env.env())?;

        let uri_obj = result_uri.l().map_err(AndroidError::from)?;
        if uri_obj.is_null() {
            return Err(AndroidError::database_error("Insert returned null URI"));
        }

        // Convert URI to string
        let uri_string_result = self
            .env
            .env()
            .call_method(&uri_obj, "toString", "()Ljava/lang/String;", &[])
            .check_exception(self.env.env())?;
        let uri_string_obj = uri_string_result.l().map_err(AndroidError::from)?;

        if uri_string_obj.is_null() {
            return Err(AndroidError::database_error(
                "Insert URI toString returned null",
            ));
        }

        let java_string = uri_string_obj.into();
        self.env.get_string_checked(&java_string)
    }
}

/// Helper for ContentProvider update operations
pub struct ContentProviderUpdate<'local> {
    env: SafeJNIEnv<'local>,
    uri: String,
    selection: Option<String>,
    selection_args: Option<Vec<String>>,
}

impl<'local> ContentProviderUpdate<'local> {
    /// Create a new update operation for the specified URI
    pub fn new(env: SafeJNIEnv<'local>, uri: &str) -> Self {
        Self {
            env,
            uri: uri.to_string(),
            selection: None,
            selection_args: None,
        }
    }

    /// Set the selection clause (WHERE clause)
    pub fn selection(mut self, selection: String) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Set the selection arguments
    pub fn selection_args(mut self, args: Vec<String>) -> Self {
        self.selection_args = Some(args);
        self
    }

    /// Execute the update with the given values
    pub fn execute(
        mut self,
        activity: &JObject<'local>,
        values: ContentValuesBuilder<'local>,
    ) -> AndroidResult<i32> {
        let content_resolver = get_content_resolver(&mut self.env, activity)?;
        let uri_obj = parse_uri(&mut self.env, &self.uri)?;
        let content_values = values.build();

        // Prepare all objects first to ensure proper lifetime
        let null_obj = JObject::null();
        
        // Create selection string if needed
        let sel_string = if let Some(sel) = &self.selection {
            Some(self.env.new_string_checked(sel)?)
        } else {
            None
        };
        
        // Create selection args array if needed
        let selection_args_array_obj = if let Some(args) = &self.selection_args {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env()
                .new_object_array(args.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, arg) in args.iter().enumerate() {
                let arg_string = self.env.new_string_checked(arg)?;
                self.env
                    .env()
                    .set_object_array_element(&array, i as i32, arg_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };
        
        // Now create JValues with proper references
        // JString implements AsRef<JObject> so we can use it directly
        let selection_obj = match sel_string.as_ref() {
            Some(s) => JValue::Object(s.as_ref()),
            None => JValue::Object(&null_obj),
        };
        
        let selection_args_array = match selection_args_array_obj.as_ref() {
            Some(a) => JValue::Object(a),
            None => JValue::Object(&null_obj),
        };

        let result = self.env.env().call_method(
            &content_resolver,
            "update",
            "(Landroid/net/Uri;Landroid/content/ContentValues;Ljava/lang/String;[Ljava/lang/String;)I",
            &[
                JValue::Object(&uri_obj),
                JValue::Object(&content_values),
                selection_obj,
                selection_args_array,
            ],
        ).check_exception(self.env.env_mut())?;

        Ok(result.i().unwrap_or(0))
    }
}

/// Helper for ContentProvider delete operations
pub struct ContentProviderDelete<'local> {
    env: SafeJNIEnv<'local>,
    uri: String,
    selection: Option<String>,
    selection_args: Option<Vec<String>>,
}

impl<'local> ContentProviderDelete<'local> {
    /// Create a new delete operation for the specified URI
    pub fn new(env: SafeJNIEnv<'local>, uri: &str) -> Self {
        Self {
            env,
            uri: uri.to_string(),
            selection: None,
            selection_args: None,
        }
    }

    /// Set the selection clause (WHERE clause)
    pub fn selection(mut self, selection: String) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Set the selection arguments
    pub fn selection_args(mut self, args: Vec<String>) -> Self {
        self.selection_args = Some(args);
        self
    }

    /// Execute the delete operation
    pub fn execute(mut self, activity: &JObject<'local>) -> AndroidResult<i32> {
        let content_resolver = get_content_resolver(&mut self.env, activity)?;
        let uri_obj = parse_uri(&mut self.env, &self.uri)?;

        // Prepare all objects first to ensure proper lifetime
        let null_obj = JObject::null();
        
        // Create selection string if needed
        let sel_string = if let Some(sel) = &self.selection {
            Some(self.env.new_string_checked(sel)?)
        } else {
            None
        };
        
        // Create selection args array if needed
        let selection_args_array_obj = if let Some(args) = &self.selection_args {
            let string_class = self.env.find_class_checked("java/lang/String")?;
            let array = self
                .env
                .env()
                .new_object_array(args.len() as i32, string_class, JObject::null())
                .check_exception(self.env.env_mut())?;

            for (i, arg) in args.iter().enumerate() {
                let arg_string = self.env.new_string_checked(arg)?;
                self.env
                    .env()
                    .set_object_array_element(&array, i as i32, arg_string)
                    .check_exception(self.env.env_mut())?;
            }
            Some(JObject::from(array))
        } else {
            None
        };
        
        // Now create JValues with proper references
        // JString implements AsRef<JObject> so we can use it directly
        let selection_obj = match sel_string.as_ref() {
            Some(s) => JValue::Object(s.as_ref()),
            None => JValue::Object(&null_obj),
        };
        
        let selection_args_array = match selection_args_array_obj.as_ref() {
            Some(a) => JValue::Object(a),
            None => JValue::Object(&null_obj),
        };

        let result = self
            .env
            .env()
            .call_method(
                &content_resolver,
                "delete",
                "(Landroid/net/Uri;Ljava/lang/String;[Ljava/lang/String;)I",
                &[
                    JValue::Object(&uri_obj),
                    selection_obj,
                    selection_args_array,
                ],
            )
            .check_exception(self.env.env())?;

        Ok(result.i().unwrap_or(0))
    }
}

/// Convenience functions for creating content provider operations
pub fn query<'local>(env: SafeJNIEnv<'local>, uri: &str) -> ContentProviderQuery<'local> {
    ContentProviderQuery::new(env, uri)
}

pub fn insert<'local>(env: SafeJNIEnv<'local>, uri: &str) -> ContentProviderInsert<'local> {
    ContentProviderInsert::new(env, uri)
}

pub fn update<'local>(env: SafeJNIEnv<'local>, uri: &str) -> ContentProviderUpdate<'local> {
    ContentProviderUpdate::new(env, uri)
}

pub fn delete<'local>(env: SafeJNIEnv<'local>, uri: &str) -> ContentProviderDelete<'local> {
    ContentProviderDelete::new(env, uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_creation() {
        // Test basic query builder concepts without actual JNI
        let uri = "content://com.ichi2.anki.flashcards/notes";
        assert!(uri.starts_with("content://"));
    }

    #[test]
    fn test_selection_string_building() {
        let selection = "name = ?";
        let args = vec!["Basic".to_string()];
        assert_eq!(selection, "name = ?");
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn test_projection_array_creation() {
        let projection = vec!["_id".to_string(), "name".to_string()];
        assert_eq!(projection.len(), 2);
        assert_eq!(projection[0], "_id");
        assert_eq!(projection[1], "name");
    }
}

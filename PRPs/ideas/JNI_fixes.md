# JNI Integration Refactoring Plan

## Overview
The AnkiDroid plugin compilation has revealed significant architectural issues in the JNI integration layer. This document outlines a comprehensive refactoring plan to resolve the 30+ compilation errors and improve the overall architecture.

## Current Issues Identified

### 1. Ownership and Borrowing Problems
- `SafeJNIEnv` ownership conflicts when passed to multiple functions
- Functions expecting owned values vs references
- JNI environment cannot be safely copied/cloned

### 2. JNI API Version Incompatibilities  
- `JValue` lifetime parameter changes (now requires two lifetimes)
- `JNIEnv` mutability requirements
- Unsafe function calls requiring explicit unsafe blocks

### 3. Architectural Design Issues
- Mixed function signatures (some take `SafeJNIEnv`, others take `&SafeJNIEnv`)
- Content provider operations expecting owned values
- Complex dependency chains causing move conflicts

## Refactoring Strategy

### Phase 1: Standardize Function Signatures

#### Option A: Reference-Based Approach (Recommended)
```rust
// Standardize all functions to take references
pub fn create_card(
    env: &SafeJNIEnv,
    activity: &JObject,
    // ... other params
) -> AndroidResult<i64>

// Update content provider functions to accept references
impl<'local> ContentValuesBuilder<'local> {
    pub fn new(env: &SafeJNIEnv<'local>) -> AndroidResult<Self>
}
```

**Pros:**
- Eliminates ownership conflicts
- More intuitive for callers
- Follows Rust borrowing best practices

**Cons:** 
- Requires updating ~20 function signatures
- May need to restructure some internal operations

#### Option B: Clone-Based Approach
```rust
// Implement safe cloning for SafeJNIEnv
impl<'local> SafeJNIEnv<'local> {
    pub fn clone_env(&self) -> SafeJNIEnv<'local> {
        // Create new instance with same underlying JNIEnv
        Self::new(self.env)
    }
}
```

**Pros:**
- Minimal signature changes
- Preserves existing call patterns

**Cons:**
- May create resource management issues
- Unclear if JNIEnv can be safely duplicated

### Phase 2: JNI API Compatibility Layer

#### Update JValue Usage
```rust
// Current (broken)
where T: From<JValue<'local>>,

// Fixed
where T: From<JValue<'local, 'local>>,
```

#### Wrap Unsafe Operations
```rust
impl<'local> SafeJNIEnv<'local> {
    pub fn pop_local_frame_safe(&mut self) -> AndroidResult<()> {
        unsafe {
            let _ = self.env.pop_local_frame(&JObject::null());
        }
        Ok(())
    }
}
```

### Phase 3: Error Handling Improvements

#### Fix Mutability Issues
```rust
// Current (broken)
fn get_exception_message(env: &jni::JNIEnv) -> Option<String>

// Fixed
fn get_exception_message(env: &mut jni::JNIEnv) -> Option<String>
```

#### Enhanced Error Context
```rust
impl AndroidError {
    pub fn with_jni_context(err: jni::errors::Error, context: &str) -> Self {
        Self::JniError(format!("{}: {}", context, err))
    }
}
```

### Phase 4: Content Provider Refactoring

#### Streamlined Builder Pattern
```rust
impl<'local> ContentProviderQuery<'local> {
    pub fn new(env: &SafeJNIEnv<'local>, uri: &str) -> Self {
        Self {
            env: env.clone_env(), // or reference with proper lifetime management
            uri: uri.to_string(),
            // ...
        }
    }
}
```

## Implementation Plan

### Step 1: Create Compatibility Layer (2-3 days)
1. Update `SafeJNIEnv` with proper lifetime management
2. Fix JValue lifetime parameters throughout codebase
3. Add unsafe operation wrappers
4. Create comprehensive test suite for JNI operations

### Step 2: Function Signature Standardization (3-4 days)
1. Update all card operations functions
2. Update all deck operations functions  
3. Update all model operations functions
4. Update content provider operations
5. Update mobile.rs to use consistent patterns

### Step 3: Error Handling Improvements (1-2 days)
1. Fix mutability issues in error handling
2. Enhance error context and debugging
3. Add proper exception handling chains

### Step 4: Integration Testing (2-3 days)
1. Verify compilation with Android target
2. Test actual AnkiDroid integration
3. Validate e2e test functionality
4. Performance and memory leak testing

## Alternative Approaches

### Option 1: Simplified JNI Wrapper
Create a higher-level wrapper that handles the complexity:
```rust
pub struct AnkiDroidJNI {
    env: JNIEnv<'static>,
    activity: JObject<'static>,
}

impl AnkiDroidJNI {
    pub fn create_card(&mut self, front: &str, back: &str) -> AndroidResult<i64> {
        // Handle all JNI complexity internally
    }
}
```

### Option 2: Async-First Architecture
```rust
pub async fn create_card_async(
    front: String, 
    back: String
) -> AndroidResult<i64> {
    tokio::task::spawn_blocking(move || {
        // JNI operations in dedicated thread
    }).await?
}
```

### Option 3: C Bridge Layer
Create a C bridge to simplify JNI interactions:
```c
// ankidroid_bridge.c
int64_t ankidroid_create_card(const char* front, const char* back);
```

## Risk Assessment

### High Risk
- **JNI Lifetime Management**: Complex lifetime relationships could cause memory issues
- **Thread Safety**: JNI environments are thread-local
- **AnkiDroid API Changes**: External dependency on AnkiDroid's content provider

### Medium Risk  
- **Performance Impact**: Additional abstraction layers
- **Testing Complexity**: Native Android testing requirements
- **Maintenance Burden**: JNI code complexity

### Low Risk
- **Compilation Issues**: Well-defined problems with clear solutions
- **API Compatibility**: Tauri plugin system is stable

## Success Criteria

1. **Zero Compilation Errors**: All Rust code compiles successfully for Android target
2. **Functional Integration**: Can successfully create, read, update, delete AnkiDroid cards
3. **E2E Test Pass**: All automated tests pass with real AnkiDroid integration
4. **Performance Acceptable**: Operations complete within 1-2 seconds
5. **Memory Safety**: No memory leaks or crashes during extended use

## Resources Required

- **Development Time**: 8-12 days total
- **Testing Device**: Android device/emulator with AnkiDroid installed
- **Rust JNI Expertise**: Understanding of Rust FFI and JNI patterns
- **Android Development Knowledge**: ContentProvider and Android app integration

## Next Steps

1. Review and approve this refactoring plan
2. Set up dedicated development branch for JNI refactoring
3. Create detailed task breakdown with time estimates
4. Begin implementation with Phase 1 (Compatibility Layer)
5. Establish testing protocol for each phase

## Additional Considerations

### Documentation
- Add comprehensive JNI integration documentation
- Create troubleshooting guide for common JNI issues
- Document AnkiDroid ContentProvider API usage

### Future Enhancements
- Consider migrating to more modern JNI libraries (jni-rs updates)
- Explore WebView-based communication as alternative to direct JNI
- Investigate Tauri mobile plugin best practices for complex integrations

---

*This document should be reviewed by the development team and updated based on implementation findings.*
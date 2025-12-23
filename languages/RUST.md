# Rust Coding Standards

> **Version:** 2.0.0 | **Status:** Active | **Updated:** 2025-12-23

Universal Rust style guide for consistent, maintainable, safe code.

---

## Core Principle

> [!IMPORTANT]
> **Safe by default. Explicit over implicit. No panics in production.**

### Quick Reference

| Rule      | Do                           | Don't                              |
| --------- | ---------------------------- | ---------------------------------- |
| Errors    | `?` operator, `Result<T, E>` | `unwrap()`, `expect()`, `panic!()` |
| Imports   | Explicit items               | Wildcards `use foo::*`             |
| IDs       | Newtypes `UserId(u64)`       | Raw primitives                     |
| Logging   | `tracing` or `log`           | `println!()`                       |
| Strings   | `&str` for params            | `&String`                          |
| Iteration | Iterators                    | Index loops                        |
| Unsafe    | Documented, minimal          | Scattered, undocumented            |

---

## File Structure

### Module Layout

```rust
//! Module-level documentation.

// ============================================================================
// Imports
// ============================================================================

use std::collections::HashMap;      // std
use serde::Serialize;               // external
use crate::error::Result;           // crate
use super::Config;                  // parent

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_TIMEOUT_MS: u64 = 30_000;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct ServiceConfig { /* ... */ }

// ============================================================================
// Implementations
// ============================================================================

impl ServiceConfig { /* ... */ }

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests { /* ... */ }
```

### Section Order

| Order | Section               | Required    |
| ----- | --------------------- | ----------- |
| 1     | Module docs (`//!`)   | Yes         |
| 2     | Imports               | Yes         |
| 3     | Constants             | If any      |
| 4     | Types                 | If any      |
| 5     | Implementations       | If any      |
| 6     | Trait Implementations | If any      |
| 7     | Private helpers       | If any      |
| 8     | Tests                 | Recommended |

---

## Imports

### Grouping (with blank lines between)

| Group    | Order | Example                          |
| -------- | ----- | -------------------------------- |
| std      | 1     | `use std::collections::HashMap;` |
| External | 2     | `use serde::Serialize;`          |
| Crate    | 3     | `use crate::error::Result;`      |
| Parent   | 4     | `use super::Config;`             |

### Rules

| Rule          | Good                          | Bad                       |
| ------------- | ----------------------------- | ------------------------- |
| Explicit      | `use std::io::{Read, Write};` | `use std::io::*;`         |
| Top of file   | Imports at top                | Inline `use` in functions |
| Group related | `use std::io::{Read, Write};` | Separate `use` for each   |

---

## Naming

### Conventions

| Item          | Convention        | Example                 |
| ------------- | ----------------- | ----------------------- |
| Types, Traits | `PascalCase`      | `UserId`, `HttpClient`  |
| Functions     | `snake_case`      | `find_user`, `is_valid` |
| Constants     | `SCREAMING_SNAKE` | `MAX_CONNECTIONS`       |
| Modules       | `snake_case`      | `user_service`          |
| Lifetimes     | Short lowercase   | `'a`, `'de`             |
| Type params   | Single uppercase  | `T`, `E`, `Item`        |

### Prefixes/Suffixes

| Pattern         | Use For              | Example          |
| --------------- | -------------------- | ---------------- |
| `is_*`, `has_*` | Boolean getters      | `is_empty()`     |
| `with_*`        | Builder methods      | `with_timeout()` |
| `into_*`        | Consuming conversion | `into_inner()`   |
| `as_*`          | Borrowing conversion | `as_str()`       |
| `to_*`          | Expensive conversion | `to_string()`    |
| `try_*`         | Fallible operation   | `try_from()`     |
| `*_mut`         | Mutable access       | `get_mut()`      |

---

## Types

### Newtypes (Type Safety)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(u64);

impl UserId {
    pub fn new(id: u64) -> Self { Self(id) }

    #[inline]
    pub fn as_u64(&self) -> u64 { self.0 }
}
```

**When to use:** IDs, domain concepts, units, validated data.

### Enums

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
```

### Builder Pattern

```rust
impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }
}

impl ClientConfigBuilder {
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> ClientConfig { /* ... */ }
}
```

### Common Derives

| Type         | Derives                                           |
| ------------ | ------------------------------------------------- |
| Value types  | `Debug, Clone, Copy, PartialEq, Eq, Hash`         |
| Data structs | `Debug, Clone, PartialEq, Serialize, Deserialize` |
| Error types  | `Debug, Error` (thiserror)                        |
| Config       | `Debug, Clone, Default, Serialize, Deserialize`   |

### Arc Inner Pattern

```rust
struct ConnectionInner {
    id: ConnectionId,
    socket: Mutex<WebSocket>,
}

#[derive(Clone)]
pub struct Connection {
    inner: Arc<ConnectionInner>,
}
```

Use for: shared ownership, cheap cloning, state across threads.

---

## Functions

### Signature Documentation

````rust
/// Brief description.
///
/// # Arguments
/// * `user_id` - The ID of the user
///
/// # Returns
/// The user if found.
///
/// # Errors
/// Returns `Error::NotFound` if user doesn't exist.
///
/// # Examples
/// ```
/// let user = find_user(UserId::new(123)).await?;
/// ```
pub async fn find_user(user_id: UserId) -> Result<User> { }
````

### Parameter Guidelines

| Guideline                | Good                      | Bad                         |
| ------------------------ | ------------------------- | --------------------------- |
| Borrow when possible     | `fn process(data: &[u8])` | `fn process(data: Vec<u8>)` |
| Use `&str` not `&String` | `fn parse(s: &str)`       | `fn parse(s: &String)`      |
| Accept `impl Trait`      | `fn read(r: impl Read)`   | `fn read(r: &mut File)`     |

### Explicit Temporaries

```rust
// ✅ GOOD — easy to debug
let request = Request::new(user_id, command);
let response = connection.send(request).await?;
let result = response.into_result()?;

// ❌ BAD — nested, hard to debug
let result = connection.send(Request::new(user_id, command)).await?.into_result()?;
```

### Attributes

```rust
#[inline]           // Small accessor methods
#[must_use]         // Return value should not be ignored
#[must_use = "..."] // With explanation
```

---

## Error Handling

### Error Type (thiserror)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not found: {resource} with id {id}")]
    NotFound { resource: &'static str, id: String },

    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Result Alias

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### Error Constructors

```rust
impl Error {
    pub fn not_found(resource: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound { resource, id: id.into() }
    }
}
```

### Propagation

```rust
// ✅ GOOD
let content = tokio::fs::read_to_string(path).await?;

// ❌ BAD
let content = tokio::fs::read_to_string(path).await.unwrap();
```

### Adding Context

```rust
let file = File::open(path)
    .map_err(|e| Error::invalid_input(format!("cannot open {}: {}", path.display(), e)))?;
```

---

## Async

### Patterns

```rust
// Async function
pub async fn fetch_data(url: &str) -> Result<Data> {
    let response = client.get(url).send().await?;
    response.json().await.map_err(Error::from)
}

// Async trait (Rust 1.75+)
pub trait DataSource {
    async fn fetch(&self, id: &str) -> Result<Data>;
}
```

### No Blocking

```rust
// ✅ GOOD — async file ops
let content = tokio::fs::read_to_string(path).await?;

// ❌ BAD — blocks executor
let content = std::fs::read_to_string(path)?;

// ✅ GOOD — spawn blocking for CPU work
let result = tokio::task::spawn_blocking(|| expensive_computation()).await?;
```

### Concurrency

```rust
// Parallel
let (users, orders) = tokio::join!(fetch_users(), fetch_orders());

// Race
tokio::select! {
    result = operation_a() => handle_a(result),
    _ = tokio::time::sleep(timeout) => return Err(Error::Timeout),
}

// Concurrent stream
stream::iter(items)
    .map(|item| async move { process(item).await })
    .buffer_unordered(10)
    .collect()
    .await
```

### Timeouts

```rust
let result = timeout(Duration::from_secs(30), async_operation())
    .await
    .map_err(|_| Error::Timeout)??;
```

---

## Unsafe

### Requirements

Every `unsafe` block MUST have:

1. **Safety comment** explaining why it's safe
2. **Minimal scope**
3. **Safe wrapper**

```rust
/// # Safety
/// Caller must ensure `index < self.len()`.
pub unsafe fn get_unchecked(&self, index: usize) -> &T {
    // SAFETY: Caller guarantees index is in bounds.
    unsafe { &*self.ptr.add(index) }
}

// Safe wrapper
pub fn get(&self, index: usize) -> Option<&T> {
    if index < self.len() {
        Some(unsafe { self.get_unchecked(index) })
    } else {
        None
    }
}
```

### Allowed

| Allowed                | Not Allowed             |
| ---------------------- | ----------------------- |
| FFI bindings           | Convenience             |
| Measured hot paths     | Premature optimization  |
| Low-level abstractions | Avoiding borrow checker |

---

## Testing

### Location & Naming

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_input_returns_ok() { }

    #[test]
    fn test_parse_empty_string_returns_error() { }

    #[tokio::test]
    async fn test_async_operation() { }
}
```

### Assertions

```rust
// ✅ GOOD — descriptive
assert!(result.is_ok(), "Expected Ok, got {:?}", result);
assert_eq!(actual, expected, "User count mismatch");

// ❌ BAD — no context
assert!(result.is_ok());
```

### Test Types

| Type        | Location            | Purpose              |
| ----------- | ------------------- | -------------------- |
| Unit        | `mod tests` in file | Individual functions |
| Integration | `tests/` directory  | Public API           |
| Doc tests   | Doc comments        | Examples work        |

---

## Documentation

### Module Docs

````rust
//! User management and authentication.
//!
//! # Examples
//! ```
//! let service = UserService::new();
//! let user = service.create("alice").await?;
//! ```
````

### Item Docs

````rust
/// A user in the system.
///
/// # Examples
/// ```
/// let user = User::new("alice");
/// ```
pub struct User { }
````

### Sections

| Section       | When              |
| ------------- | ----------------- |
| Brief         | Always (one line) |
| `# Arguments` | Function params   |
| `# Returns`   | Return value      |
| `# Errors`    | Error conditions  |
| `# Safety`    | Unsafe functions  |
| `# Examples`  | Usage examples    |

---

## Forbidden Patterns

### Never in Production

| Pattern               | Why            | Alternative      |
| --------------------- | -------------- | ---------------- |
| `unwrap()`            | Panics         | `?`, `ok_or()`   |
| `expect()`            | Panics         | `?` with context |
| `panic!()`            | Crashes        | Return `Result`  |
| `use foo::*`          | Unclear        | Explicit imports |
| `println!()`          | Unstructured   | `tracing`        |
| Undocumented `unsafe` | Unclear safety | Document it      |

### Exceptions

| Pattern    | Allowed When                  |
| ---------- | ----------------------------- |
| `unwrap()` | Tests only                    |
| `expect()` | Tests, or proven can't fail   |
| `panic!()` | Tests, or truly unrecoverable |

---

## Pre-Commit Checklist

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

| Check                                | Status |
| ------------------------------------ | ------ |
| `cargo fmt` passes                   | ☐      |
| `cargo clippy -- -D warnings` passes | ☐      |
| `cargo test` passes                  | ☐      |
| No `unwrap()` in prod code           | ☐      |
| No `expect()` in prod code           | ☐      |
| All public items documented          | ☐      |
| All unsafe has safety comments       | ☐      |

---

## Common Patterns

### Builder

```rust
pub struct RequestBuilder {
    url: String,
    timeout: Duration,
}

impl RequestBuilder {
    #[must_use]
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }

    pub fn build(self) -> Request { /* ... */ }
}
```

### Validated Newtype

```rust
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.trim().is_empty() {
            return Err(Error::invalid_input("cannot be empty"));
        }
        Ok(Self(s))
    }
}
```

### RAII Resource Handle

```rust
pub struct TempFile { path: PathBuf }

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

---

## Recommended Crates

| Category      | Crate        | Use                |
| ------------- | ------------ | ------------------ |
| Errors        | `thiserror`  | Library errors     |
| Errors        | `anyhow`     | Application errors |
| Serialization | `serde`      | Framework          |
| Serialization | `serde_json` | JSON               |
| Async         | `tokio`      | Runtime            |
| Async         | `reqwest`    | HTTP client        |
| Logging       | `tracing`    | Structured logging |
| Testing       | `proptest`   | Property testing   |
| Testing       | `mockall`    | Mocking            |
| CLI           | `clap`       | Arg parsing        |

---

_End of Rust Coding Standards._

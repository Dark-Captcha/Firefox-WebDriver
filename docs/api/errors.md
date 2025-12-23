# Errors

Error types and handling patterns.

## Overview

All fallible operations return `Result<T>` which uses the `Error` enum.

```rust
use firefox_webdriver::{Result, Error};

async fn example(tab: &Tab) -> Result<()> {
    let element = tab.find_element("#submit").await?;
    element.click().await?;
    Ok(())
}
```

## Error Categories

| Category      | Variants                                                      |
| ------------- | ------------------------------------------------------------- |
| Configuration | `Config`, `Profile`, `FirefoxNotFound`, `ProcessLaunchFailed` |
| Connection    | `Connection`, `ConnectionTimeout`, `ConnectionClosed`         |
| Protocol      | `UnknownCommand`, `InvalidArgument`, `Protocol`               |
| Element       | `ElementNotFound`, `StaleElement`                             |
| Navigation    | `FrameNotFound`, `TabNotFound`                                |
| Execution     | `ScriptError`, `Timeout`, `RequestTimeout`                    |
| Network       | `InterceptNotFound`                                           |
| External      | `Io`, `Json`, `WebSocket`, `ChannelClosed`                    |

---

## Error Variants

### Configuration Errors

#### `Config`

Configuration error.

```rust
Error::Config { message: String }
```

#### `Profile`

Profile creation or setup failed.

```rust
Error::Profile { message: String }
```

#### `FirefoxNotFound`

Firefox binary not found at path.

```rust
Error::FirefoxNotFound { path: PathBuf }
```

#### `ProcessLaunchFailed`

Firefox process failed to start.

```rust
Error::ProcessLaunchFailed { message: String }
```

---

### Connection Errors

#### `Connection`

WebSocket connection failed.

```rust
Error::Connection { message: String }
```

#### `ConnectionTimeout`

Extension did not connect within timeout.

```rust
Error::ConnectionTimeout { timeout_ms: u64 }
```

#### `ConnectionClosed`

WebSocket connection closed unexpectedly.

```rust
Error::ConnectionClosed
```

---

### Element Errors

#### `ElementNotFound`

No Element matches CSS selector.

```rust
Error::ElementNotFound {
    selector: String,
    tab_id: TabId,
    frame_id: FrameId,
}
```

#### `StaleElement`

Element reference is no longer valid (Element removed from DOM).

```rust
Error::StaleElement { element_id: ElementId }
```

---

### Execution Errors

#### `Timeout`

Operation exceeded timeout duration.

```rust
Error::Timeout {
    operation: String,
    timeout_ms: u64,
}
```

#### `ScriptError`

JavaScript execution failed.

```rust
Error::ScriptError { message: String }
```

---

## Error Predicates

### `is_timeout`

Returns true if Error is a timeout error.

```rust
pub fn is_timeout(&self) -> bool
```

Matches: `ConnectionTimeout`, `Timeout`, `RequestTimeout`

---

### `is_element_error`

Returns true if Error is an element error.

```rust
pub fn is_element_error(&self) -> bool
```

Matches: `ElementNotFound`, `StaleElement`

---

### `is_connection_error`

Returns true if Error is a connection error.

```rust
pub fn is_connection_error(&self) -> bool
```

Matches: `Connection`, `ConnectionTimeout`, `ConnectionClosed`, `WebSocket`

---

### `is_recoverable`

Returns true if Error may succeed on retry.

```rust
pub fn is_recoverable(&self) -> bool
```

Matches: `ConnectionTimeout`, `Timeout`, `RequestTimeout`, `StaleElement`

---

## Handling Patterns

### Match on Error Type

```rust
use firefox_webdriver::{Error, Result};

async fn find_or_default(tab: &Tab, selector: &str) -> Result<Option<Element>> {
    match tab.find_element(selector).await {
        Ok(element) => Ok(Some(element)),
        Err(Error::ElementNotFound { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}
```

### Retry on Recoverable Errors

```rust
use firefox_webdriver::{Error, Result};
use std::time::Duration;
use tokio::time::sleep;

async fn retry<F, T>(mut f: F, max_retries: u32) -> Result<T>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempts < max_retries => {
                attempts += 1;
                sleep(Duration::from_millis(100 * attempts as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Check Error Category

```rust
use firefox_webdriver::Error;

fn handle_error(error: &Error) {
    if error.is_connection_error() {
        println!("Connection issue - may need to restart browser");
    } else if error.is_element_error() {
        println!("Element issue - selector may be wrong or element removed");
    } else if error.is_timeout() {
        println!("Timeout - operation took too long");
    }
}
```

---

## See Also

- [Getting Started](../getting-started.md) - Quick start guide
- [Tab](./tab.md) - Tab automation

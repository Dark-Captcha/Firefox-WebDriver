# Tab

Automates a browser tab with frame context.

## Overview

Tab provides methods for navigation, scripting, element interaction, network interception, and storage access. Each Tab has a frame context (main frame or iframe).

## Getting Tab

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();
    Ok(())
}
```

## Accessors

| Method            | Returns     | Description              |
| ----------------- | ----------- | ------------------------ |
| `tab_id()`        | `TabId`     | Tab identifier           |
| `frame_id()`      | `FrameId`   | Current frame identifier |
| `session_id()`    | `SessionId` | Session identifier       |
| `is_main_frame()` | `bool`      | True if in main frame    |

---

## Navigation

### `goto`

Navigates to a URL.

```rust
pub async fn goto(&self, url: &str) -> Result<()>
```

#### Examples

```rust
tab.goto("https://example.com").await?;
```

---

### `reload`

Reloads the current page.

```rust
pub async fn reload(&self) -> Result<()>
```

---

### `back`

Navigates back in history.

```rust
pub async fn back(&self) -> Result<()>
```

---

### `forward`

Navigates forward in history.

```rust
pub async fn forward(&self) -> Result<()>
```

---

### `get_title`

Returns the page title.

```rust
pub async fn get_title(&self) -> Result<String>
```

---

### `get_url`

Returns the current URL.

```rust
pub async fn get_url(&self) -> Result<String>
```

---

### `load_html`

Loads HTML content directly.

```rust
pub async fn load_html(&self, html: &str) -> Result<()>
```

#### Examples

```rust
tab.load_html("<html><body><h1>Test</h1></body></html>").await?;
```

---

### `focus`

Focuses Tab (makes Tab active).

```rust
pub async fn focus(&self) -> Result<()>
```

---

### `close`

Closes Tab.

```rust
pub async fn close(&self) -> Result<()>
```

---

## Frame Switching

### `switch_to_frame`

Switches to a frame by iframe Element.

```rust
pub async fn switch_to_frame(&self, iframe: &Element) -> Result<Tab>
```

Returns a new Tab handle with updated frame context.

#### Examples

```rust
let iframe = tab.find_element("iframe#content").await?;
let frame_tab = tab.switch_to_frame(&iframe).await?;
frame_tab.find_element("button").await?;
```

---

### `switch_to_frame_by_index`

Switches to a frame by index (0-based).

```rust
pub async fn switch_to_frame_by_index(&self, index: usize) -> Result<Tab>
```

---

### `switch_to_frame_by_url`

Switches to a frame by URL pattern (supports `*` and `?` wildcards).

```rust
pub async fn switch_to_frame_by_url(&self, url_pattern: &str) -> Result<Tab>
```

---

### `switch_to_parent_frame`

Switches to the parent frame.

```rust
pub async fn switch_to_parent_frame(&self) -> Result<Tab>
```

---

### `switch_to_main_frame`

Switches to the main (top-level) frame.

```rust
pub fn switch_to_main_frame(&self) -> Tab
```

---

### `get_frame_count`

Returns the count of direct child frames.

```rust
pub async fn get_frame_count(&self) -> Result<usize>
```

---

### `get_all_frames`

Returns information about all frames.

```rust
pub async fn get_all_frames(&self) -> Result<Vec<FrameInfo>>
```

---

## Element Search

### `find_element`

Finds a single Element by CSS selector.

```rust
pub async fn find_element(&self, selector: &str) -> Result<Element>
```

#### Errors

| Error             | When                        |
| ----------------- | --------------------------- |
| `ElementNotFound` | No Element matches selector |

#### Examples

```rust
let button = tab.find_element("button.submit").await?;
```

---

### `find_elements`

Finds all Elements matching a CSS selector.

```rust
pub async fn find_elements(&self, selector: &str) -> Result<Vec<Element>>
```

#### Examples

```rust
let items = tab.find_elements("li.item").await?;
for item in items {
    println!("{}", item.get_text().await?);
}
```

---

### `wait_for_element`

Waits for an Element to appear (30 second timeout).

```rust
pub async fn wait_for_element(&self, selector: &str) -> Result<Element>
```

Uses MutationObserver (no polling).

#### Errors

| Error     | When                                      |
| --------- | ----------------------------------------- |
| `Timeout` | Element does not appear within 30 seconds |

---

### `wait_for_element_timeout`

Waits for an Element with custom timeout.

```rust
pub async fn wait_for_element_timeout(
    &self,
    selector: &str,
    timeout: Duration
) -> Result<Element>
```

---

## Element Observation

### `on_element_added`

Registers a callback for when Elements appear.

```rust
pub async fn on_element_added<F>(&self, selector: &str, callback: F) -> Result<SubscriptionId>
where
    F: Fn(Element) + Send + Sync + 'static
```

Returns `SubscriptionId` for later unsubscription.

---

### `on_element_removed`

Registers a callback for when an Element is removed.

```rust
pub async fn on_element_removed<F>(&self, element_id: &ElementId, callback: F) -> Result<()>
where
    F: Fn() + Send + Sync + 'static
```

---

### `unsubscribe`

Unsubscribes from element observation.

```rust
pub async fn unsubscribe(&self, subscription_id: &SubscriptionId) -> Result<()>
```

---

## Script Execution

### `execute_script`

Executes synchronous JavaScript.

```rust
pub async fn execute_script(&self, script: &str) -> Result<Value>
```

#### Examples

```rust
let title = tab.execute_script("return document.title").await?;
```

---

### `execute_async_script`

Executes asynchronous JavaScript.

```rust
pub async fn execute_async_script(&self, script: &str) -> Result<Value>
```

---

## Network

### `set_block_rules`

Sets URL patterns to block.

```rust
pub async fn set_block_rules(&self, patterns: &[&str]) -> Result<()>
```

#### Examples

```rust
tab.set_block_rules(&["*ads*", "*tracking*"]).await?;
```

---

### `clear_block_rules`

Clears all URL block rules.

```rust
pub async fn clear_block_rules(&self) -> Result<()>
```

---

### `intercept_request`

Intercepts network requests.

```rust
pub async fn intercept_request<F>(&self, callback: F) -> Result<InterceptId>
where
    F: Fn(InterceptedRequest) -> RequestAction + Send + Sync + 'static
```

See [Network API](./network.md) for details.

---

### `stop_intercept`

Stops network interception.

```rust
pub async fn stop_intercept(&self, intercept_id: &InterceptId) -> Result<()>
```

---

## Storage

### Cookies

| Method                | Description           |
| --------------------- | --------------------- |
| `get_cookie(name)`    | Gets a cookie by name |
| `set_cookie(cookie)`  | Sets a cookie         |
| `delete_cookie(name)` | Deletes a cookie      |
| `get_all_cookies()`   | Gets all cookies      |

### localStorage

| Method                          | Description   |
| ------------------------------- | ------------- |
| `local_storage_get(key)`        | Gets a value  |
| `local_storage_set(key, value)` | Sets a value  |
| `local_storage_delete(key)`     | Deletes a key |
| `local_storage_clear()`         | Clears all    |

### sessionStorage

| Method                            | Description   |
| --------------------------------- | ------------- |
| `session_storage_get(key)`        | Gets a value  |
| `session_storage_set(key, value)` | Sets a value  |
| `session_storage_delete(key)`     | Deletes a key |
| `session_storage_clear()`         | Clears all    |

---

## Proxy

### `set_proxy`

Sets a proxy for Tab (overrides Window proxy).

```rust
pub async fn set_proxy(&self, config: ProxyConfig) -> Result<()>
```

---

### `clear_proxy`

Clears Tab proxy.

```rust
pub async fn clear_proxy(&self) -> Result<()>
```

---

## See Also

- [Element](./element.md) - Element interaction
- [Network](./network.md) - Network interception
- [Storage](./storage.md) - Cookie and storage APIs
- [Frames Guide](../guides/frames.md) - Frame switching patterns

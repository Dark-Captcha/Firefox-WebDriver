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

## Screenshot

### `screenshot`

Creates a screenshot builder for capturing page screenshots.

```rust
pub fn screenshot(&self) -> ScreenshotBuilder<'_>
```

#### Examples

```rust
// PNG screenshot as base64
let data = tab.screenshot().png().capture().await?;

// JPEG screenshot saved to file
tab.screenshot().jpeg(85).save("page.jpg").await?;

// Get raw bytes
let bytes = tab.screenshot().png().capture_bytes().await?;
```

---

### `capture_screenshot`

Captures a PNG screenshot and returns base64-encoded data.

```rust
pub async fn capture_screenshot(&self) -> Result<String>
```

Shorthand for `tab.screenshot().png().capture().await`.

---

### `save_screenshot`

Captures a screenshot and saves to a file.

```rust
pub async fn save_screenshot(&self, path: impl AsRef<Path>) -> Result<()>
```

Format is determined by file extension (.png or .jpg/.jpeg).

---

### ScreenshotBuilder Methods

| Method            | Description                    |
| ----------------- | ------------------------------ |
| `png()`           | Sets PNG format (default)      |
| `jpeg(quality)`   | Sets JPEG format (0-100)       |
| `format(format)`  | Sets format via `ImageFormat`  |
| `capture()`       | Returns base64 string          |
| `capture_bytes()` | Returns raw bytes              |
| `save(path)`      | Saves to file                  |

---

## Scroll

### `scroll_by`

Scrolls the page by the specified amount.

```rust
pub async fn scroll_by(&self, x: i32, y: i32) -> Result<()>
```

#### Examples

```rust
// Scroll down 500 pixels
tab.scroll_by(0, 500).await?;

// Scroll right 200 pixels
tab.scroll_by(200, 0).await?;
```

---

### `scroll_to`

Scrolls the page to the specified position.

```rust
pub async fn scroll_to(&self, x: i32, y: i32) -> Result<()>
```

---

### `scroll_to_top`

Scrolls to the top of the page.

```rust
pub async fn scroll_to_top(&self) -> Result<()>
```

---

### `scroll_to_bottom`

Scrolls to the bottom of the page.

```rust
pub async fn scroll_to_bottom(&self) -> Result<()>
```

---

### `get_scroll_position`

Gets the current scroll position.

```rust
pub async fn get_scroll_position(&self) -> Result<(i32, i32)>
```

Returns tuple of (x, y) scroll position in pixels.

---

### `get_page_size`

Gets the page dimensions (scrollable area).

```rust
pub async fn get_page_size(&self) -> Result<(i32, i32)>
```

Returns tuple of (width, height) in pixels.

---

### `get_viewport_size`

Gets the viewport dimensions.

```rust
pub async fn get_viewport_size(&self) -> Result<(i32, i32)>
```

Returns tuple of (width, height) in pixels.

---

### `get_page_source`

Gets the page source HTML.

```rust
pub async fn get_page_source(&self) -> Result<String>
```

---

## Element Search with Strategies

### `find_element`

Finds a single element using a locator strategy.

```rust
pub async fn find_element(&self, by: By) -> Result<Element>
```

#### Examples

```rust
use firefox_webdriver::By;

// CSS selector
let btn = tab.find_element(By::Css("#submit")).await?;

// By ID
let form = tab.find_element(By::Id("login-form")).await?;

// By text content
let link = tab.find_element(By::Text("Click here")).await?;

// By XPath
let btn = tab.find_element(By::XPath("//button[@type='submit']")).await?;

// By partial text
let link = tab.find_element(By::PartialText("Read")).await?;
```

---

### `find_elements`

Finds all elements using a locator strategy.

```rust
pub async fn find_elements(&self, by: By) -> Result<Vec<Element>>
```

#### Examples

```rust
use firefox_webdriver::By;

let buttons = tab.find_elements(By::Tag("button")).await?;
let links = tab.find_elements(By::PartialText("Read")).await?;
```

---

### By Strategies

| Strategy          | Description            | Example                            |
| ----------------- | ---------------------- | ---------------------------------- |
| `By::Css`         | CSS selector (default) | `By::Css("#login")`                |
| `By::XPath`       | XPath expression       | `By::XPath("//button")`            |
| `By::Text`        | Exact text content     | `By::Text("Submit")`               |
| `By::PartialText` | Partial text content   | `By::PartialText("Read")`          |
| `By::Id`          | Element ID             | `By::Id("username")`               |
| `By::Tag`         | Tag name               | `By::Tag("button")`                |
| `By::Name`        | Name attribute         | `By::Name("email")`                |
| `By::Class`       | Class name             | `By::Class("btn-primary")`         |
| `By::LinkText`    | Link text (`<a>`)      | `By::LinkText("Home")`             |
| `By::PartialLinkText` | Partial link text  | `By::PartialLinkText("Read")`      |

---

## See Also

- [Element](./element.md) - Element interaction
- [Network](./network.md) - Network interception
- [Storage](./storage.md) - Cookie and storage APIs
- [Frames Guide](../guides/frames.md) - Frame switching patterns
- [Screenshot Guide](../guides/screenshot.md) - Screenshot patterns
- [Forms Guide](../guides/forms.md) - Form interaction patterns

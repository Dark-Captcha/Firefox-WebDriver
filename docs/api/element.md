# Element

Interacts with DOM elements.

## Overview

Element represents a DOM element identified by UUID. Elements are stored in the content script's internal Map, making Element references undetectable.

## Getting Element

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();
    tab.goto("https://example.com").await?;

    let element = tab.find_element("button.submit").await?;
    Ok(())
}
```

## Accessors

| Method       | Returns      | Description                |
| ------------ | ------------ | -------------------------- |
| `id()`       | `&ElementId` | Element UUID               |
| `tab_id()`   | `TabId`      | Tab where Element exists   |
| `frame_id()` | `FrameId`    | Frame where Element exists |

---

## Actions

### `click`

Clicks Element using `element.click()`.

```rust
pub async fn click(&self) -> Result<()>
```

#### Examples

```rust
let button = tab.find_element("button.submit").await?;
button.click().await?;
```

---

### `focus`

Focuses Element.

```rust
pub async fn focus(&self) -> Result<()>
```

---

### `blur`

Blurs (unfocuses) Element.

```rust
pub async fn blur(&self) -> Result<()>
```

---

### `clear`

Clears Element value (sets `element.value = ""`).

```rust
pub async fn clear(&self) -> Result<()>
```

---

## Properties

### `get_text`

Returns Element text content.

```rust
pub async fn get_text(&self) -> Result<String>
```

---

### `get_inner_html`

Returns Element inner HTML.

```rust
pub async fn get_inner_html(&self) -> Result<String>
```

---

### `get_value`

Returns Element value (for input elements).

```rust
pub async fn get_value(&self) -> Result<String>
```

---

### `set_value`

Sets Element value (for input elements).

```rust
pub async fn set_value(&self, value: &str) -> Result<()>
```

#### Examples

```rust
let input = tab.find_element("input[name='email']").await?;
input.set_value("user@example.com").await?;
```

---

### `get_attribute`

Returns an attribute value.

```rust
pub async fn get_attribute(&self, name: &str) -> Result<Option<String>>
```

#### Examples

```rust
let link = tab.find_element("a").await?;
let href = link.get_attribute("href").await?;
```

---

### `is_displayed`

Returns true if Element is visible.

```rust
pub async fn is_displayed(&self) -> Result<bool>
```

---

### `is_enabled`

Returns true if Element is enabled.

```rust
pub async fn is_enabled(&self) -> Result<bool>
```

---

## Keyboard Input

### `type_text`

Types text character by character.

```rust
pub async fn type_text(&self, text: &str) -> Result<()>
```

Each character goes through full keyboard event sequence (keydown → input → keypress → keyup). type_text is slower but more realistic than `set_value`.

#### Examples

```rust
let input = tab.find_element("input").await?;
input.type_text("Hello, World!").await?;
```

---

### `type_key`

Types a single key with modifiers.

```rust
pub async fn type_key(
    &self,
    key: &str,
    code: &str,
    key_code: u32,
    printable: bool,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
) -> Result<()>
```

#### Parameters

| Name        | Type   | Description                         |
| ----------- | ------ | ----------------------------------- |
| `key`       | `&str` | Key value (e.g., "a", "Enter")      |
| `code`      | `&str` | Key code (e.g., "KeyA", "Enter")    |
| `key_code`  | `u32`  | Legacy keyCode number               |
| `printable` | `bool` | Whether key produces visible output |
| `ctrl`      | `bool` | Ctrl modifier                       |
| `shift`     | `bool` | Shift modifier                      |
| `alt`       | `bool` | Alt modifier                        |
| `meta`      | `bool` | Meta modifier                       |

---

### `type_char`

Types a single character.

```rust
pub async fn type_char(&self, c: char) -> Result<()>
```

---

## Mouse Input

### `mouse_click`

Clicks Element using mouse events.

```rust
pub async fn mouse_click(&self, button: u8) -> Result<()>
```

Dispatches: mousemove → mousedown → mouseup → click. mouse_click is more realistic than `click()`.

| Button | Value |
| ------ | ----- |
| Left   | 0     |
| Middle | 1     |
| Right  | 2     |

---

### `mouse_move`

Moves mouse to Element center.

```rust
pub async fn mouse_move(&self) -> Result<()>
```

---

### `mouse_down`

Presses mouse button down (without release).

```rust
pub async fn mouse_down(&self, button: u8) -> Result<()>
```

---

### `mouse_up`

Releases mouse button.

```rust
pub async fn mouse_up(&self, button: u8) -> Result<()>
```

---

## Nested Search

### `find_element`

Finds a child Element by CSS selector.

```rust
pub async fn find_element(&self, selector: &str) -> Result<Element>
```

#### Errors

| Error             | When                              |
| ----------------- | --------------------------------- |
| `ElementNotFound` | No child Element matches selector |

#### Examples

```rust
let form = tab.find_element("form").await?;
let submit = form.find_element("button[type='submit']").await?;
```

---

### `find_elements`

Finds all child Elements matching a CSS selector.

```rust
pub async fn find_elements(&self, selector: &str) -> Result<Vec<Element>>
```

---

## Generic Property Access

### `get_property`

Gets a property value via `element[name]`.

```rust
pub async fn get_property(&self, name: &str) -> Result<Value>
```

---

### `set_property`

Sets a property value via `element[name] = value`.

```rust
pub async fn set_property(&self, name: &str, value: Value) -> Result<()>
```

---

### `call_method`

Calls a method via `element[name](...args)`.

```rust
pub async fn call_method(&self, name: &str, args: Vec<Value>) -> Result<Value>
```

#### Examples

```rust
use serde_json::json;

let element = tab.find_element("div").await?;
element.call_method("scrollIntoView", vec![json!({"behavior": "smooth"})]).await?;
```

---

## See Also

- [Tab](./tab.md) - Tab automation
- [Element Interaction Guide](../guides/element-interaction.md) - Patterns for finding and interacting with elements

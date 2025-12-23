# Element Interaction Guide

Patterns for finding and interacting with DOM elements.

## Problem

You need to find elements on a page and interact with them (click, type, read values).

## Solution

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com/login").await?;

    // Find elements
    let email_input = tab.find_element("input[name='email']").await?;
    let password_input = tab.find_element("input[name='password']").await?;
    let submit_button = tab.find_element("button[type='submit']").await?;

    // Type into inputs
    email_input.type_text("user@example.com").await?;
    password_input.type_text("password123").await?;

    // Click button
    submit_button.click().await?;

    Ok(())
}
```

## Finding Elements

### By CSS Selector

```rust
// By ID
let element = tab.find_element("#submit").await?;

// By class
let element = tab.find_element(".btn-primary").await?;

// By attribute
let element = tab.find_element("input[name='email']").await?;

// By tag
let element = tab.find_element("button").await?;

// Complex selector
let element = tab.find_element("form.login input[type='email']").await?;
```

### Multiple Elements

```rust
let items = tab.find_elements("li.item").await?;
for item in items {
    let text = item.get_text().await?;
    println!("{}", text);
}
```

### Nested Search

```rust
let form = tab.find_element("form.login").await?;
let email = form.find_element("input[name='email']").await?;
let password = form.find_element("input[name='password']").await?;
```

### Handle Missing Elements

```rust
use firefox_webdriver::Error;

match tab.find_element("#optional-element").await {
    Ok(element) => {
        element.click().await?;
    }
    Err(Error::ElementNotFound { .. }) => {
        println!("Element not found, skipping");
    }
    Err(e) => return Err(e),
}
```

---

## Clicking Elements

### Simple Click

```rust
let button = tab.find_element("button").await?;
button.click().await?;
```

### Mouse Click (More Realistic)

```rust
let button = tab.find_element("button").await?;
button.mouse_click(0).await?; // 0 = left button
```

### Right Click

```rust
let element = tab.find_element("div.context-menu-target").await?;
element.mouse_click(2).await?; // 2 = right button
```

---

## Typing Text

### type_text vs set_value

| Method      | Description                                       | Use When                                  |
| ----------- | ------------------------------------------------- | ----------------------------------------- |
| `type_text` | Types character by character with keyboard events | Need realistic input, trigger JS handlers |
| `set_value` | Sets value property directly                      | Fast, no event handlers needed            |

### type_text (Realistic)

```rust
let input = tab.find_element("input").await?;
input.type_text("Hello, World!").await?;
```

### set_value (Fast)

```rust
let input = tab.find_element("input").await?;
input.set_value("Hello, World!").await?;
```

### Clear Before Typing

```rust
let input = tab.find_element("input").await?;
input.clear().await?;
input.type_text("new value").await?;
```

---

## Reading Element Data

### Text Content

```rust
let heading = tab.find_element("h1").await?;
let text = heading.get_text().await?;
```

### Input Value

```rust
let input = tab.find_element("input").await?;
let value = input.get_value().await?;
```

### Attribute

```rust
let link = tab.find_element("a").await?;
let href = link.get_attribute("href").await?;
```

### Check State

```rust
let button = tab.find_element("button").await?;

if button.is_displayed().await? && button.is_enabled().await? {
    button.click().await?;
}
```

---

## Common Mistakes

| Mistake                                     | Why Wrong                   | Fix                          |
| ------------------------------------------- | --------------------------- | ---------------------------- |
| Using `unwrap()` on find_element            | Panics if element not found | Use `?` operator             |
| Not waiting for element                     | Element may not exist yet   | Use `wait_for_element`       |
| Using `set_value` for forms with validation | JS handlers not triggered   | Use `type_text`              |
| Clicking hidden elements                    | Click may fail silently     | Check `is_displayed()` first |

---

## See Also

- [Element API](../api/element.md) - Element methods
- [Waiting Guide](./waiting.md) - Waiting for elements
- [Tab API](../api/tab.md) - Tab methods

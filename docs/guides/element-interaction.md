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

## Using By Strategies

### Available Strategies

```rust
use firefox_webdriver::By;

// CSS selector (default, most common)
let btn = tab.find_element(By::Css("#submit")).await?;

// By ID (shorthand for #id)
let form = tab.find_element(By::Id("login-form")).await?;

// By XPath
let btn = tab.find_element(By::XPath("//button[@type='submit']")).await?;

// By text content (exact match)
let link = tab.find_element(By::Text("Click here")).await?;

// By partial text
let link = tab.find_element(By::PartialText("Click")).await?;

// By tag name
let inputs = tab.find_elements(By::Tag("input")).await?;

// By name attribute
let email = tab.find_element(By::Name("email")).await?;

// By class name
let btn = tab.find_element(By::Class("btn-primary")).await?;

// By link text (for <a> elements)
let link = tab.find_element(By::LinkText("Home")).await?;
```

### Strategy Reference

| Strategy              | Description            | Example                       |
| --------------------- | ---------------------- | ----------------------------- |
| `By::Css`             | CSS selector           | `By::Css("#login")`           |
| `By::XPath`           | XPath expression       | `By::XPath("//button")`       |
| `By::Text`            | Exact text content     | `By::Text("Submit")`          |
| `By::PartialText`     | Partial text content   | `By::PartialText("Read")`     |
| `By::Id`              | Element ID             | `By::Id("username")`          |
| `By::Tag`             | Tag name               | `By::Tag("button")`           |
| `By::Name`            | Name attribute         | `By::Name("email")`           |
| `By::Class`           | Class name             | `By::Class("btn-primary")`    |
| `By::LinkText`        | Link text (`<a>`)      | `By::LinkText("Home")`        |
| `By::PartialLinkText` | Partial link text      | `By::PartialLinkText("Read")` |

---

## Mouse Actions

### Double Click

```rust
let button = tab.find_element(By::Css("button")).await?;
button.double_click().await?;
```

### Right Click (Context Menu)

```rust
let element = tab.find_element(By::Css("div.target")).await?;
element.context_click().await?;
```

### Hover

```rust
let menu = tab.find_element(By::Css(".dropdown")).await?;
menu.hover().await?;
// Now dropdown should be visible
let item = tab.find_element(By::Css(".dropdown-item")).await?;
item.click().await?;
```

---

## Scrolling

### Scroll Element Into View

```rust
let element = tab.find_element(By::Css("#footer")).await?;
element.scroll_into_view().await?; // Smooth animation
// or
element.scroll_into_view_instant().await?; // Instant
```

### Page Scrolling

```rust
// Scroll down 500 pixels
tab.scroll_by(0, 500).await?;

// Scroll to specific position
tab.scroll_to(0, 1000).await?;

// Scroll to top/bottom
tab.scroll_to_top().await?;
tab.scroll_to_bottom().await?;

// Get current scroll position
let (x, y) = tab.get_scroll_position().await?;
```

---

## See Also

- [Element API](../api/element.md) - Element methods
- [Waiting Guide](./waiting.md) - Waiting for elements
- [Tab API](../api/tab.md) - Tab methods
- [Forms Guide](./forms.md) - Form interaction patterns

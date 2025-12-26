# Forms Guide

Patterns for interacting with form elements: inputs, checkboxes, radio buttons, and dropdowns.

## Problem

You need to fill out forms, select options, and interact with various form controls.

## Solution

```rust
use firefox_webdriver::{Driver, By, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com/form").await?;

    // Text input
    let email = tab.find_element(By::Name("email")).await?;
    email.type_text("user@example.com").await?;

    // Checkbox
    let agree = tab.find_element(By::Id("agree")).await?;
    agree.check().await?;

    // Dropdown
    let country = tab.find_element(By::Id("country")).await?;
    country.select_by_text("United States").await?;

    // Submit
    let submit = tab.find_element(By::Css("button[type='submit']")).await?;
    submit.click().await?;

    Ok(())
}
```

## Text Inputs

### type_text vs set_value

| Method      | Description                       | Use When                          |
| ----------- | --------------------------------- | --------------------------------- |
| `type_text` | Types character by character      | Need realistic input, JS handlers |
| `set_value` | Sets value property directly      | Fast, no event handlers needed    |

### Typing Text

```rust
let input = tab.find_element(By::Name("username")).await?;

// Realistic typing (triggers keyboard events)
input.type_text("john_doe").await?;

// Fast value setting (no events)
input.set_value("john_doe").await?;
```

### Clear and Type

```rust
let input = tab.find_element(By::Name("search")).await?;
input.clear().await?;
input.type_text("new search term").await?;
```

### Special Keys

```rust
use firefox_webdriver::Key;

let input = tab.find_element(By::Name("search")).await?;
input.type_text("search term").await?;
input.press(Key::Enter).await?; // Submit
```

---

## Checkboxes

### Check/Uncheck

```rust
let checkbox = tab.find_element(By::Id("newsletter")).await?;

// Check (does nothing if already checked)
checkbox.check().await?;

// Uncheck (does nothing if already unchecked)
checkbox.uncheck().await?;

// Toggle
checkbox.toggle().await?;

// Set specific state
checkbox.set_checked(true).await?;
```

### Check State

```rust
let checkbox = tab.find_element(By::Id("agree")).await?;

if !checkbox.is_checked().await? {
    checkbox.check().await?;
}
```

---

## Radio Buttons

Radio buttons work like checkboxes but only one can be selected in a group.

```rust
// Select by clicking the specific radio button
let option_b = tab.find_element(By::Css("input[name='plan'][value='premium']")).await?;
option_b.click().await?;

// Or use check()
option_b.check().await?;
```

---

## Dropdowns (Select)

### Select by Text

```rust
let select = tab.find_element(By::Id("country")).await?;
select.select_by_text("United States").await?;
```

### Select by Value

```rust
let select = tab.find_element(By::Id("country")).await?;
select.select_by_value("US").await?;
```

### Select by Index

```rust
let select = tab.find_element(By::Id("country")).await?;
select.select_by_index(0).await?; // First option
```

### Get Selected Option

```rust
let select = tab.find_element(By::Id("country")).await?;

// Get selected value
let value = select.get_selected_value().await?;

// Get selected text
let text = select.get_selected_text().await?;

// Get selected index
let index = select.get_selected_index().await?;
```

### Check if Multi-Select

```rust
let select = tab.find_element(By::Id("tags")).await?;
if select.is_multiple().await? {
    // Can select multiple options
}
```

---

## Complete Form Example

```rust
use firefox_webdriver::{Driver, By, Key, Result};

async fn fill_registration_form(driver: &Driver) -> Result<()> {
    let window = driver.window().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com/register").await?;

    // Text fields
    tab.find_element(By::Name("firstName")).await?.type_text("John").await?;
    tab.find_element(By::Name("lastName")).await?.type_text("Doe").await?;
    tab.find_element(By::Name("email")).await?.type_text("john@example.com").await?;
    tab.find_element(By::Name("password")).await?.type_text("SecurePass123!").await?;

    // Dropdown
    tab.find_element(By::Id("country")).await?.select_by_text("United States").await?;

    // Checkboxes
    tab.find_element(By::Id("newsletter")).await?.check().await?;
    tab.find_element(By::Id("terms")).await?.check().await?;

    // Radio button
    tab.find_element(By::Css("input[name='plan'][value='premium']")).await?.click().await?;

    // Submit
    tab.find_element(By::Css("button[type='submit']")).await?.click().await?;

    Ok(())
}
```

---

## Common Mistakes

| Mistake                          | Why Wrong                    | Fix                           |
| -------------------------------- | ---------------------------- | ----------------------------- |
| Using `set_value` on dropdowns   | Doesn't trigger change event | Use `select_by_*` methods     |
| Not clearing input before typing | Appends to existing value    | Call `clear()` first          |
| Clicking disabled elements       | No effect                    | Check `is_enabled()` first    |
| Using `check()` on non-checkbox  | Unexpected behavior          | Use `click()` for buttons     |

---

## See Also

- [Element API](../api/element.md) - Element methods
- [Element Interaction Guide](./element-interaction.md) - General interaction patterns
- [Waiting Guide](./waiting.md) - Waiting for elements

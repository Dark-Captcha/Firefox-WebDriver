# Waiting Guide

Event-driven waiting for elements and conditions.

## Problem

You need to wait for elements to appear before interacting with them.

## Solution

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com").await?;

    // Wait for element to appear (30 second timeout)
    let element = tab.wait_for_element(".dynamic-content").await?;
    element.click().await?;

    Ok(())
}
```

## Why Event-Driven?

This library uses MutationObserver instead of polling.

| Approach         | How It Works                   | Pros               | Cons                              |
| ---------------- | ------------------------------ | ------------------ | --------------------------------- |
| Polling          | Check every N ms               | Simple             | Wastes CPU, may miss fast changes |
| MutationObserver | Browser notifies on DOM change | Efficient, instant | Requires extension support        |

## wait_for_element

Waits for an element to appear with 30 second default timeout.

```rust
let element = tab.wait_for_element("button.submit").await?;
```

### Custom Timeout

```rust
use std::time::Duration;

let element = tab.wait_for_element_timeout(
    "button.submit",
    Duration::from_secs(10)
).await?;
```

### Handle Timeout

```rust
use firefox_webdriver::Error;

match tab.wait_for_element("button.submit").await {
    Ok(element) => {
        element.click().await?;
    }
    Err(Error::Timeout { .. }) => {
        println!("Element did not appear within timeout");
    }
    Err(e) => return Err(e),
}
```

---

## Element Observation

### on_element_added

Registers a callback for when elements appear.

```rust
let subscription_id = tab.on_element_added(".notification", |element| {
    println!("Notification appeared!");
}).await?;

// Later: stop observing
tab.unsubscribe(&subscription_id).await?;
```

### on_element_removed

Registers a callback for when an element is removed.

```rust
let element = tab.find_element(".modal").await?;

tab.on_element_removed(element.id(), || {
    println!("Modal was closed");
}).await?;
```

---

## Patterns

### Wait Then Interact

```rust
let button = tab.wait_for_element("button.submit").await?;
button.click().await?;
```

### Wait for Multiple Elements

```rust
// Wait for first element
let container = tab.wait_for_element(".results-container").await?;

// Then find children (they should exist now)
let items = container.find_elements(".result-item").await?;
```

### Wait After Navigation

```rust
tab.goto("https://example.com/dashboard").await?;

// Wait for dashboard to load
let dashboard = tab.wait_for_element(".dashboard-content").await?;
```

### Wait for Dynamic Content

```rust
// Click button that loads content
let load_button = tab.find_element("button.load-more").await?;
load_button.click().await?;

// Wait for new content
let new_item = tab.wait_for_element(".item:nth-child(11)").await?;
```

---

## Common Mistakes

| Mistake                                  | Why Wrong                 | Fix                                  |
| ---------------------------------------- | ------------------------- | ------------------------------------ |
| Using `find_element` for dynamic content | Element may not exist yet | Use `wait_for_element`               |
| Very long timeouts                       | Hides real issues         | Use reasonable timeout, handle error |
| Not unsubscribing                        | Memory leak               | Call `unsubscribe` when done         |
| Polling in a loop                        | Inefficient               | Use `wait_for_element`               |

---

## See Also

- [Element Interaction Guide](./element-interaction.md) - Finding and clicking elements
- [Tab API](../api/tab.md) - Tab methods

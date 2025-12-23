# Frames Guide

Patterns for working with iframes.

## Problem

You need to interact with elements inside iframes.

## Solution

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com").await?;

    // Find iframe element
    let iframe = tab.find_element("iframe#content").await?;

    // Switch to frame (returns new Tab handle)
    let frame_tab = tab.switch_to_frame(&iframe).await?;

    // Now interact with elements inside iframe
    let button = frame_tab.find_element("button").await?;
    button.click().await?;

    // Switch back to main frame
    let main_tab = frame_tab.switch_to_main_frame();

    Ok(())
}
```

## Frame Model

Each Tab has a frame context. When you switch frames, you get a new Tab handle with the new frame context.

```
Main Frame (tab)
├── iframe#header (frame_tab1)
├── iframe#content (frame_tab2)
│   └── iframe#nested (frame_tab3)
└── iframe#footer (frame_tab4)
```

## Switching Frames

### By Element

```rust
let iframe = tab.find_element("iframe#content").await?;
let frame_tab = tab.switch_to_frame(&iframe).await?;
```

### By Index

```rust
// Switch to first iframe (0-based index)
let frame_tab = tab.switch_to_frame_by_index(0).await?;
```

### By URL Pattern

```rust
// Supports * and ? wildcards
let frame_tab = tab.switch_to_frame_by_url("*example.com/embed*").await?;
```

---

## Returning to Parent/Main Frame

### Parent Frame

```rust
let parent_tab = frame_tab.switch_to_parent_frame().await?;
```

### Main Frame

```rust
let main_tab = frame_tab.switch_to_main_frame();
```

---

## Frame Information

### Check if Main Frame

```rust
if tab.is_main_frame() {
    println!("In main frame");
}
```

### Get Frame Count

```rust
let count = tab.get_frame_count().await?;
println!("Direct child frames: {}", count);
```

### Get All Frames

```rust
let frames = tab.get_all_frames().await?;
for frame in frames {
    println!("Frame ID: {:?}, URL: {}", frame.frame_id, frame.url);
}
```

---

## Patterns

### Interact with Nested Iframe

```rust
// Main frame -> iframe#outer -> iframe#inner
let outer = tab.find_element("iframe#outer").await?;
let outer_tab = tab.switch_to_frame(&outer).await?;

let inner = outer_tab.find_element("iframe#inner").await?;
let inner_tab = outer_tab.switch_to_frame(&inner).await?;

// Interact with element in nested iframe
let button = inner_tab.find_element("button").await?;
button.click().await?;

// Return to main frame
let main_tab = inner_tab.switch_to_main_frame();
```

### Find Frame by Content

```rust
let frames = tab.get_all_frames().await?;

for frame_info in frames {
    if frame_info.url.contains("login") {
        let frame_tab = tab.switch_to_frame_by_url(&frame_info.url).await?;
        // Found the login frame
        break;
    }
}
```

### Handle Dynamic Iframes

```rust
// Wait for iframe to appear
let iframe = tab.wait_for_element("iframe.dynamic").await?;
let frame_tab = tab.switch_to_frame(&iframe).await?;

// Wait for content inside iframe
let content = frame_tab.wait_for_element(".loaded-content").await?;
```

---

## Common Mistakes

| Mistake                               | Why Wrong                         | Fix                           |
| ------------------------------------- | --------------------------------- | ----------------------------- |
| Using old Tab after switching         | Old Tab still points to old frame | Use returned Tab handle       |
| Not switching back to main            | Subsequent operations fail        | Call `switch_to_main_frame`   |
| Finding iframe element in wrong frame | Element not found                 | Switch to correct frame first |

---

## See Also

- [Tab API](../api/tab.md) - Tab methods
- [Waiting Guide](./waiting.md) - Waiting for elements

# Screenshot Guide

Patterns for capturing screenshots of pages and elements.

## Problem

You need to capture screenshots of pages or specific elements for testing, debugging, or documentation.

## Solution

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com").await?;

    // Quick save to file
    tab.save_screenshot("page.png").await?;

    Ok(())
}
```

## Page Screenshots

### Quick Capture

```rust
// Save PNG (format from extension)
tab.save_screenshot("screenshot.png").await?;

// Save JPEG
tab.save_screenshot("screenshot.jpg").await?;

// Get base64 data
let base64_data = tab.capture_screenshot().await?;
```

### Builder Pattern

```rust
// PNG with builder
let data = tab.screenshot().png().capture().await?;

// JPEG with quality
let data = tab.screenshot().jpeg(85).capture().await?;

// Get raw bytes
let bytes = tab.screenshot().png().capture_bytes().await?;

// Save to file
tab.screenshot().jpeg(90).save("high-quality.jpg").await?;
```

---

## Element Screenshots

Capture specific elements instead of the full page.

```rust
use firefox_webdriver::By;

// Find element
let chart = tab.find_element(By::Css("#chart")).await?;

// Capture as PNG
let base64_data = chart.screenshot().await?;

// Capture as JPEG with quality
let jpeg_data = chart.screenshot_jpeg(80).await?;

// Get raw bytes
let bytes = chart.screenshot_bytes().await?;

// Save to file
chart.save_screenshot("chart.png").await?;
```

---

## Image Formats

| Format | Method       | Use Case                    |
| ------ | ------------ | --------------------------- |
| PNG    | `.png()`     | Lossless, larger file size  |
| JPEG   | `.jpeg(q)`   | Lossy, smaller file size    |

### JPEG Quality

```rust
// Low quality (smaller file)
tab.screenshot().jpeg(50).save("low.jpg").await?;

// Medium quality
tab.screenshot().jpeg(75).save("medium.jpg").await?;

// High quality (larger file)
tab.screenshot().jpeg(95).save("high.jpg").await?;
```

---

## Common Patterns

### Screenshot Before/After

```rust
// Before action
tab.save_screenshot("before.png").await?;

// Perform action
let button = tab.find_element(By::Css("#submit")).await?;
button.click().await?;

// After action
tab.save_screenshot("after.png").await?;
```

### Screenshot on Error

```rust
async fn safe_action(tab: &Tab) -> Result<()> {
    match tab.find_element(By::Css("#element")).await {
        Ok(el) => el.click().await,
        Err(e) => {
            tab.save_screenshot("error.png").await?;
            Err(e)
        }
    }
}
```

### Scroll and Capture

```rust
// Scroll to element first
let element = tab.find_element(By::Css("#target")).await?;
element.scroll_into_view().await?;

// Small delay for scroll animation
tokio::time::sleep(Duration::from_millis(300)).await;

// Capture
element.save_screenshot("element.png").await?;
```

---

## See Also

- [Tab API](../api/tab.md) - Tab screenshot methods
- [Element API](../api/element.md) - Element screenshot methods

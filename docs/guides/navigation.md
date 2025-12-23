# Navigation Guide

Patterns for navigating pages.

## Problem

You need to navigate to URLs, handle history, and check page state.

## Solution

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    // Navigate to URL
    tab.goto("https://example.com").await?;

    // Get page info
    let title = tab.get_title().await?;
    let url = tab.get_url().await?;

    println!("Title: {}", title);
    println!("URL: {}", url);

    Ok(())
}
```

## Basic Navigation

### Navigate to URL

```rust
tab.goto("https://example.com").await?;
```

### Reload Page

```rust
tab.reload().await?;
```

### History Navigation

```rust
// Go back
tab.back().await?;

// Go forward
tab.forward().await?;
```

---

## Page Information

### Get Title

```rust
let title = tab.get_title().await?;
```

### Get URL

```rust
let url = tab.get_url().await?;
```

---

## Load HTML Directly

Load HTML content without a server.

```rust
tab.load_html(r#"
    <html>
    <body>
        <h1>Test Page</h1>
        <button id="test">Click Me</button>
    </body>
    </html>
"#).await?;

let button = tab.find_element("#test").await?;
button.click().await?;
```

---

## Tab Management

### Focus Tab

```rust
tab.focus().await?;
```

### Focus Window

```rust
tab.focus_window().await?;
```

### Close Tab

```rust
tab.close().await?;
```

### Create New Tab

```rust
let new_tab = window.new_tab().await?;
new_tab.goto("https://example.com").await?;
```

---

## Patterns

### Wait After Navigation

```rust
tab.goto("https://example.com/dashboard").await?;

// Wait for page content to load
let content = tab.wait_for_element(".dashboard-content").await?;
```

### Check URL After Action

```rust
let login_button = tab.find_element("button.login").await?;
login_button.click().await?;

// Wait for redirect
tokio::time::sleep(std::time::Duration::from_secs(1)).await;

let url = tab.get_url().await?;
if url.contains("/dashboard") {
    println!("Login successful");
}
```

### Multiple Tabs

```rust
let tab1 = window.tab();
let tab2 = window.new_tab().await?;

tab1.goto("https://example.com/page1").await?;
tab2.goto("https://example.com/page2").await?;

// Switch focus
tab1.focus().await?;
```

---

## See Also

- [Tab API](../api/tab.md) - Tab methods
- [Window API](../api/window.md) - Window methods
- [Waiting Guide](./waiting.md) - Waiting for elements

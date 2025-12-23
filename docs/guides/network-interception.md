# Network Interception Guide

Patterns for intercepting and modifying network traffic.

## Problem

You need to block requests, modify headers, or change response content.

## Solution

```rust
use firefox_webdriver::{Driver, RequestAction, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    // Intercept requests
    let intercept_id = tab.intercept_request(|req| {
        if req.url.contains("ads") {
            RequestAction::block()
        } else {
            RequestAction::allow()
        }
    }).await?;

    tab.goto("https://example.com").await?;

    // Stop interception when done
    tab.stop_intercept(&intercept_id).await?;

    Ok(())
}
```

## Blocking URLs

### Simple Blocking

```rust
tab.set_block_rules(&["*ads*", "*tracking*"]).await?;
tab.goto("https://example.com").await?;
```

### Clear Block Rules

```rust
tab.clear_block_rules().await?;
```

---

## Request Interception

### Block Requests

```rust
use firefox_webdriver::RequestAction;

let intercept_id = tab.intercept_request(|req| {
    if req.resource_type == "image" {
        RequestAction::block()
    } else {
        RequestAction::allow()
    }
}).await?;
```

### Redirect Requests

```rust
use firefox_webdriver::RequestAction;

let intercept_id = tab.intercept_request(|req| {
    if req.url.contains("old-api.example.com") {
        RequestAction::redirect(req.url.replace("old-api", "new-api"))
    } else {
        RequestAction::allow()
    }
}).await?;
```

### Log Requests

```rust
use firefox_webdriver::RequestAction;

let intercept_id = tab.intercept_request(|req| {
    println!("{} {} ({})", req.method, req.url, req.resource_type);
    RequestAction::allow()
}).await?;
```

---

## Header Interception

### Modify Request Headers

```rust
use firefox_webdriver::HeadersAction;
use std::collections::HashMap;

let intercept_id = tab.intercept_request_headers(|req| {
    let mut headers = req.headers.clone();
    headers.insert("X-Custom-Header".to_string(), "value".to_string());
    headers.insert("User-Agent".to_string(), "Custom Agent".to_string());
    HeadersAction::modify_headers(headers)
}).await?;
```

### Modify Response Headers

```rust
use firefox_webdriver::HeadersAction;

let intercept_id = tab.intercept_response(|res| {
    let mut headers = res.headers.clone();
    headers.remove("X-Frame-Options");
    HeadersAction::modify_headers(headers)
}).await?;
```

---

## Body Interception

### Read Request Body

```rust
let intercept_id = tab.intercept_request_body(|req| {
    if let Some(body) = &req.body {
        println!("Request to {} has body: {:?}", req.url, body);
    }
}).await?;
```

### Modify Response Body

```rust
use firefox_webdriver::BodyAction;

let intercept_id = tab.intercept_response_body(|res| {
    if res.url.contains("config.json") {
        BodyAction::modify_body(r#"{"modified": true, "debug": true}"#)
    } else {
        BodyAction::allow()
    }
}).await?;
```

---

## Patterns

### Block Ads and Tracking

```rust
use firefox_webdriver::RequestAction;

let intercept_id = tab.intercept_request(|req| {
    let block_patterns = [
        "ads", "tracking", "analytics", "facebook.com/tr",
        "google-analytics", "doubleclick", "adservice"
    ];

    if block_patterns.iter().any(|p| req.url.contains(p)) {
        RequestAction::block()
    } else {
        RequestAction::allow()
    }
}).await?;
```

### Mock API Responses

```rust
use firefox_webdriver::BodyAction;

let intercept_id = tab.intercept_response_body(|res| {
    if res.url.contains("/api/user") {
        BodyAction::modify_body(r#"{"id": 1, "name": "Test User"}"#)
    } else {
        BodyAction::allow()
    }
}).await?;
```

### Log All Network Traffic

```rust
use firefox_webdriver::RequestAction;

let intercept_id = tab.intercept_request(|req| {
    println!("[REQUEST] {} {} ({})", req.method, req.url, req.resource_type);
    RequestAction::allow()
}).await?;

let response_id = tab.intercept_response(|res| {
    println!("[RESPONSE] {} {} {}", res.status, res.status_text, res.url);
    firefox_webdriver::HeadersAction::allow()
}).await?;
```

---

## Stopping Interception

Always stop interception when done:

```rust
tab.stop_intercept(&intercept_id).await?;
```

---

## Common Mistakes

| Mistake                   | Why Wrong                 | Fix                   |
| ------------------------- | ------------------------- | --------------------- |
| Not stopping interception | Memory leak, performance  | Call `stop_intercept` |
| Blocking too broadly      | Breaks page functionality | Use specific patterns |
| Modifying all responses   | Performance impact        | Filter by URL         |

---

## See Also

- [Network API](../api/network.md) - Network methods
- [Tab API](../api/tab.md) - Tab methods

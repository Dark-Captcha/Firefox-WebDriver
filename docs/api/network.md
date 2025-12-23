# Network

Network interception and blocking.

## Overview

Tab provides methods to block URLs and intercept network requests/responses.

## Blocking URLs

### `set_block_rules`

Sets URL patterns to block. Patterns support `*` wildcard.

```rust
pub async fn set_block_rules(&self, patterns: &[&str]) -> Result<()>
```

#### Examples

```rust
// Block ads and tracking
tab.set_block_rules(&["*ads*", "*tracking*", "*analytics*"]).await?;

// Block specific domains
tab.set_block_rules(&["*facebook.com*", "*google-analytics.com*"]).await?;
```

### `clear_block_rules`

Clears all URL block rules.

```rust
pub async fn clear_block_rules(&self) -> Result<()>
```

---

## Request Interception

### `intercept_request`

Intercepts network requests with a callback.

```rust
pub async fn intercept_request<F>(&self, callback: F) -> Result<InterceptId>
where
    F: Fn(InterceptedRequest) -> RequestAction + Send + Sync + 'static
```

#### Returns

`InterceptId` - Use with `stop_intercept` to stop interception.

#### Examples

```rust
use firefox_webdriver::RequestAction;

let intercept_id = tab.intercept_request(|req| {
    if req.url.contains("ads") {
        RequestAction::block()
    } else if req.url.contains("old-api") {
        RequestAction::redirect("https://new-api.example.com")
    } else {
        RequestAction::allow()
    }
}).await?;

// Later: stop interception
tab.stop_intercept(&intercept_id).await?;
```

---

## InterceptedRequest

Data about an intercepted request.

| Field           | Type                  | Description                                 |
| --------------- | --------------------- | ------------------------------------------- |
| `request_id`    | `String`              | Unique request ID                           |
| `url`           | `String`              | Request URL                                 |
| `method`        | `String`              | HTTP method (GET, POST, etc.)               |
| `resource_type` | `String`              | Resource type (document, script, xhr, etc.) |
| `tab_id`        | `u32`                 | Tab ID                                      |
| `frame_id`      | `u64`                 | Frame ID                                    |
| `body`          | `Option<RequestBody>` | Request body (if available)                 |

---

## RequestAction

Action to take for an intercepted request.

| Variant         | Description               |
| --------------- | ------------------------- |
| `Allow`         | Allow request to proceed  |
| `Block`         | Block/cancel request      |
| `Redirect(url)` | Redirect to different URL |

### Constructors

```rust
RequestAction::allow()
RequestAction::block()
RequestAction::redirect("https://example.com")
```

---

## Request Headers Interception

### `intercept_request_headers`

Intercepts request headers.

```rust
pub async fn intercept_request_headers<F>(&self, callback: F) -> Result<InterceptId>
where
    F: Fn(InterceptedRequestHeaders) -> HeadersAction + Send + Sync + 'static
```

#### Examples

```rust
use firefox_webdriver::HeadersAction;
use std::collections::HashMap;

let intercept_id = tab.intercept_request_headers(|req| {
    let mut headers = req.headers.clone();
    headers.insert("X-Custom-Header".to_string(), "value".to_string());
    HeadersAction::modify_headers(headers)
}).await?;
```

---

## Request Body Interception

### `intercept_request_body`

Intercepts request body (read-only, cannot be modified).

```rust
pub async fn intercept_request_body<F>(&self, callback: F) -> Result<InterceptId>
where
    F: Fn(InterceptedRequestBody) + Send + Sync + 'static
```

#### Examples

```rust
let intercept_id = tab.intercept_request_body(|req| {
    if let Some(body) = &req.body {
        println!("Request to {} has body: {:?}", req.url, body);
    }
}).await?;
```

---

## Response Interception

### `intercept_response`

Intercepts response headers.

```rust
pub async fn intercept_response<F>(&self, callback: F) -> Result<InterceptId>
where
    F: Fn(InterceptedResponse) -> HeadersAction + Send + Sync + 'static
```

#### Examples

```rust
use firefox_webdriver::HeadersAction;

let intercept_id = tab.intercept_response(|res| {
    println!("Response from {}: {} {}", res.url, res.status, res.status_text);
    HeadersAction::allow()
}).await?;
```

---

## InterceptedResponse

Data about an intercepted response.

| Field         | Type                      | Description       |
| ------------- | ------------------------- | ----------------- |
| `request_id`  | `String`                  | Unique request ID |
| `url`         | `String`                  | Request URL       |
| `status`      | `u16`                     | HTTP status code  |
| `status_text` | `String`                  | HTTP status text  |
| `headers`     | `HashMap<String, String>` | Response headers  |
| `tab_id`      | `u32`                     | Tab ID            |
| `frame_id`    | `u64`                     | Frame ID          |

---

## Response Body Interception

### `intercept_response_body`

Intercepts response body.

```rust
pub async fn intercept_response_body<F>(&self, callback: F) -> Result<InterceptId>
where
    F: Fn(InterceptedResponseBody) -> BodyAction + Send + Sync + 'static
```

#### Examples

```rust
use firefox_webdriver::BodyAction;

let intercept_id = tab.intercept_response_body(|res| {
    if res.url.contains("config.json") {
        BodyAction::modify_body(r#"{"modified": true}"#)
    } else {
        BodyAction::allow()
    }
}).await?;
```

---

## BodyAction

Action to take for intercepted response body.

| Variant            | Description          |
| ------------------ | -------------------- |
| `Allow`            | Allow body unchanged |
| `ModifyBody(body)` | Replace body content |

### Constructors

```rust
BodyAction::allow()
BodyAction::modify_body("new content")
```

---

## HeadersAction

Action to take for intercepted headers.

| Variant                  | Description             |
| ------------------------ | ----------------------- |
| `Allow`                  | Allow headers unchanged |
| `ModifyHeaders(headers)` | Replace headers         |

### Constructors

```rust
HeadersAction::allow()
HeadersAction::modify_headers(headers_map)
```

---

## Stopping Interception

### `stop_intercept`

Stops network interception.

```rust
pub async fn stop_intercept(&self, intercept_id: &InterceptId) -> Result<()>
```

---

## See Also

- [Tab](./tab.md) - Tab automation
- [Network Interception Guide](../guides/network-interception.md) - Detailed patterns

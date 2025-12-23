# Storage

Cookie and web storage APIs.

## Overview

Tab provides methods for managing cookies, localStorage, and sessionStorage.

## Cookies

### `get_cookie`

Gets a cookie by name.

```rust
pub async fn get_cookie(&self, name: &str) -> Result<Option<Cookie>>
```

#### Examples

```rust
if let Some(cookie) = tab.get_cookie("session").await? {
    println!("Session: {}", cookie.value);
}
```

---

### `set_cookie`

Sets a cookie.

```rust
pub async fn set_cookie(&self, cookie: Cookie) -> Result<()>
```

#### Examples

```rust
use firefox_webdriver::Cookie;

tab.set_cookie(Cookie::new("session", "abc123")).await?;
```

---

### `delete_cookie`

Deletes a cookie by name.

```rust
pub async fn delete_cookie(&self, name: &str) -> Result<()>
```

---

### `get_all_cookies`

Gets all cookies for the current page.

```rust
pub async fn get_all_cookies(&self) -> Result<Vec<Cookie>>
```

#### Examples

```rust
let cookies = tab.get_all_cookies().await?;
for cookie in cookies {
    println!("{}: {}", cookie.name, cookie.value);
}
```

---

## Cookie Type

| Field       | Type             | Description              |
| ----------- | ---------------- | ------------------------ |
| `name`      | `String`         | Cookie name              |
| `value`     | `String`         | Cookie value             |
| `domain`    | `Option<String>` | Cookie domain            |
| `path`      | `Option<String>` | Cookie path              |
| `secure`    | `bool`           | HTTPS only               |
| `http_only` | `bool`           | HTTP only (no JS access) |
| `same_site` | `Option<String>` | SameSite attribute       |
| `expiry`    | `Option<i64>`    | Expiration timestamp     |

---

## localStorage

### `local_storage_get`

Gets a value from localStorage.

```rust
pub async fn local_storage_get(&self, key: &str) -> Result<Option<String>>
```

#### Examples

```rust
if let Some(value) = tab.local_storage_get("user_id").await? {
    println!("User ID: {}", value);
}
```

---

### `local_storage_set`

Sets a value in localStorage.

```rust
pub async fn local_storage_set(&self, key: &str, value: &str) -> Result<()>
```

#### Examples

```rust
tab.local_storage_set("user_id", "12345").await?;
```

---

### `local_storage_delete`

Deletes a key from localStorage.

```rust
pub async fn local_storage_delete(&self, key: &str) -> Result<()>
```

---

### `local_storage_clear`

Clears all localStorage.

```rust
pub async fn local_storage_clear(&self) -> Result<()>
```

---

## sessionStorage

### `session_storage_get`

Gets a value from sessionStorage.

```rust
pub async fn session_storage_get(&self, key: &str) -> Result<Option<String>>
```

---

### `session_storage_set`

Sets a value in sessionStorage.

```rust
pub async fn session_storage_set(&self, key: &str, value: &str) -> Result<()>
```

---

### `session_storage_delete`

Deletes a key from sessionStorage.

```rust
pub async fn session_storage_delete(&self, key: &str) -> Result<()>
```

---

### `session_storage_clear`

Clears all sessionStorage.

```rust
pub async fn session_storage_clear(&self) -> Result<()>
```

---

## Examples

### Managing Session

```rust
use firefox_webdriver::{Driver, Cookie, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com").await?;

    // Set session cookie
    tab.set_cookie(Cookie::new("session", "abc123")).await?;

    // Store user preferences
    tab.local_storage_set("theme", "dark").await?;
    tab.local_storage_set("language", "en").await?;

    // Reload to apply
    tab.reload().await?;

    Ok(())
}
```

### Reading Storage

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();

    tab.goto("https://example.com").await?;

    // Read all cookies
    let cookies = tab.get_all_cookies().await?;
    println!("Cookies: {:?}", cookies);

    // Read localStorage
    if let Some(theme) = tab.local_storage_get("theme").await? {
        println!("Theme: {}", theme);
    }

    Ok(())
}
```

---

## See Also

- [Tab](./tab.md) - Tab automation

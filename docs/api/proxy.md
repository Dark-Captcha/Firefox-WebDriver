# Proxy

Proxy configuration for Windows and Tabs.

## Overview

Proxies can be set at Window level (applies to all Tabs) or Tab level (overrides Window proxy for that Tab).

## ProxyConfig

Configuration for proxy settings.

### Constructors

#### `http`

Creates an HTTP proxy configuration.

```rust
pub fn http(host: impl Into<String>, port: u16) -> Self
```

#### `https`

Creates an HTTPS proxy configuration.

```rust
pub fn https(host: impl Into<String>, port: u16) -> Self
```

#### `socks4`

Creates a SOCKS4 proxy configuration.

```rust
pub fn socks4(host: impl Into<String>, port: u16) -> Self
```

#### `socks5`

Creates a SOCKS5 proxy configuration.

```rust
pub fn socks5(host: impl Into<String>, port: u16) -> Self
```

#### `direct`

Creates a direct (no proxy) configuration.

```rust
pub fn direct() -> Self
```

---

### Builder Methods

#### `with_credentials`

Sets authentication credentials.

```rust
pub fn with_credentials(self, username: impl Into<String>, password: impl Into<String>) -> Self
```

#### `with_proxy_dns`

Enables DNS proxying (SOCKS4/SOCKS5 only).

```rust
pub fn with_proxy_dns(self, proxy_dns: bool) -> Self
```

---

### Predicates

| Method       | Description                         |
| ------------ | ----------------------------------- |
| `has_auth()` | Returns true if credentials are set |
| `is_socks()` | Returns true if SOCKS4 or SOCKS5    |
| `is_http()`  | Returns true if HTTP or HTTPS       |

---

## ProxyType

Proxy protocol type.

| Variant  | Description    |
| -------- | -------------- |
| `Http`   | HTTP proxy     |
| `Https`  | HTTPS proxy    |
| `Socks4` | SOCKS v4 proxy |
| `Socks5` | SOCKS v5 proxy |
| `Direct` | No proxy       |

---

## Examples

### HTTP Proxy

```rust
use firefox_webdriver::{Driver, ProxyConfig, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;

    window.set_proxy(ProxyConfig::http("proxy.example.com", 8080)).await?;

    let tab = window.tab();
    tab.goto("https://example.com").await?;

    Ok(())
}
```

### SOCKS5 Proxy with Authentication

```rust
use firefox_webdriver::{Driver, ProxyConfig, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;

    let proxy = ProxyConfig::socks5("proxy.example.com", 1080)
        .with_credentials("username", "password")
        .with_proxy_dns(true);

    window.set_proxy(proxy).await?;

    Ok(())
}
```

### Tab-Level Proxy

```rust
use firefox_webdriver::{Driver, ProxyConfig, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;

    // Window proxy for all tabs
    window.set_proxy(ProxyConfig::http("proxy1.example.com", 8080)).await?;

    let tab = window.tab();

    // Tab-specific proxy (overrides window proxy)
    tab.set_proxy(ProxyConfig::http("proxy2.example.com", 8080)).await?;

    // Clear tab proxy (falls back to window proxy)
    tab.clear_proxy().await?;

    Ok(())
}
```

### Clear Proxy

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;

    // Clear window proxy (direct connection)
    window.clear_proxy().await?;

    Ok(())
}
```

---

## See Also

- [Window](./window.md) - Window management
- [Tab](./tab.md) - Tab automation

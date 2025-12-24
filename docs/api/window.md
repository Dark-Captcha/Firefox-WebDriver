# Window

Manages a Firefox browser window.

## Overview

Window owns a Firefox process and profile directory, and holds a reference to the shared ConnectionPool for WebSocket communication. When Window is dropped or closed, the Firefox process is killed and the session is removed from the pool.

## Creating Window

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window()
        .headless()
        .window_size(1920, 1080)
        .spawn()
        .await?;
    Ok(())
}
```

## Accessors

### `session_id`

Returns the session ID.

```rust
pub fn session_id(&self) -> SessionId
```

### `port`

Returns the WebSocket port.

```rust
pub fn port(&self) -> u16
```

### `pid`

Returns the Firefox process ID.

```rust
pub fn pid(&self) -> u32
```

---

## Tab Management

### `tab`

Returns the initial Tab for Window.

#### Signature

```rust
pub fn tab(&self) -> Tab
```

#### Returns

`Tab` - The initial Tab created when Firefox opens.

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let tab = window.tab();
    tab.goto("https://example.com").await?;
    Ok(())
}
```

---

### `new_tab`

Creates a new Tab in Window.

#### Signature

```rust
pub async fn new_tab(&self) -> Result<Tab>
```

#### Returns

`Result<Tab>` - New Tab instance.

#### Errors

| Error      | When               |
| ---------- | ------------------ |
| `Protocol` | Tab creation fails |

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    let new_tab = window.new_tab().await?;
    new_tab.goto("https://example.com").await?;
    Ok(())
}
```

---

### `tab_count`

Returns the number of Tabs in Window.

```rust
pub fn tab_count(&self) -> usize
```

---

## Proxy

### `set_proxy`

Sets a proxy for all Tabs in Window.

#### Signature

```rust
pub async fn set_proxy(&self, config: ProxyConfig) -> Result<()>
```

#### Parameters

| Name     | Type          | Description         |
| -------- | ------------- | ------------------- |
| `config` | `ProxyConfig` | Proxy configuration |

#### Examples

```rust
use firefox_webdriver::{Driver, ProxyConfig, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;

    // HTTP proxy
    window.set_proxy(ProxyConfig::http("proxy.example.com", 8080)).await?;

    // SOCKS5 proxy with auth
    window.set_proxy(
        ProxyConfig::socks5("proxy.example.com", 1080)
            .with_credentials("user", "pass")
            .with_proxy_dns(true)
    ).await?;

    Ok(())
}
```

---

### `clear_proxy`

Clears the proxy for Window.

#### Signature

```rust
pub async fn clear_proxy(&self) -> Result<()>
```

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    window.clear_proxy().await?;
    Ok(())
}
```

---

## Lifecycle

### `close`

Closes Window and kills Firefox process.

#### Signature

```rust
pub async fn close(&self) -> Result<()>
```

#### Errors

| Error   | When                     |
| ------- | ------------------------ |
| Various | Process cannot be killed |

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window().headless().spawn().await?;
    // ... use window ...
    window.close().await?;
    Ok(())
}
```

---

## WindowBuilder

Builder for spawning Windows.

### `headless`

Enables headless mode (no visible window).

```rust
pub fn headless(self) -> Self
```

### `window_size`

Sets window dimensions.

```rust
pub fn window_size(self, width: u32, height: u32) -> Self
```

| Parameter | Type  | Description             |
| --------- | ----- | ----------------------- |
| `width`   | `u32` | Window width in pixels  |
| `height`  | `u32` | Window height in pixels |

### `profile`

Uses a custom profile directory.

```rust
pub fn profile(self, path: impl Into<PathBuf>) -> Self
```

### `spawn`

Spawns the Window.

```rust
pub async fn spawn(self) -> Result<Window>
```

#### Errors

| Error                 | When                       |
| --------------------- | -------------------------- |
| `Profile`             | Profile creation fails     |
| `Connection`          | WebSocket binding fails    |
| `ProcessLaunchFailed` | Firefox fails to start     |
| `ConnectionTimeout`   | Extension fails to connect |

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window()
        .headless()
        .window_size(1920, 1080)
        .profile("./my_profile")
        .spawn()
        .await?;
    Ok(())
}
```

---

## See Also

- [Driver](./driver.md) - Driver factory
- [Tab](./tab.md) - Tab automation
- [Proxy](./proxy.md) - Proxy configuration

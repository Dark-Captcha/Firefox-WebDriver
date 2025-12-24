# Driver

Creates and manages browser Windows.

## Overview

Driver is the entry point for browser automation. Driver creates Window instances, each owning a Firefox process.

## Creating Driver

```rust
use firefox_webdriver::{Driver, Result};

fn main() -> Result<()> {
    let driver = Driver::builder()
        .binary("/usr/bin/firefox")
        .extension("./extension")
        .build().await?;
    Ok(())
}
```

## Methods

### `builder`

Creates a DriverBuilder for configuration.

#### Signature

```rust
pub fn builder() -> DriverBuilder
```

#### Returns

`DriverBuilder` - Builder for configuring Driver.

#### Examples

```rust
use firefox_webdriver::Driver;

let builder = Driver::builder();
```

---

### `window`

Creates a WindowBuilder for spawning browser Windows.

#### Signature

```rust
pub fn window(&self) -> WindowBuilder<'_>
```

#### Returns

`WindowBuilder` - Builder for configuring and spawning a Window.

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    let window = driver.window()
        .headless()
        .spawn()
        .await?;
    Ok(())
}
```

---

### `window_count`

Returns the number of active Windows.

#### Signature

```rust
pub fn window_count(&self) -> usize
```

#### Returns

`usize` - Number of active Windows tracked by Driver.

#### Examples

```rust
use firefox_webdriver::Driver;

fn example(driver: &Driver) {
    let count = driver.window_count();
    println!("Active windows: {}", count);
}
```

---

### `close`

Closes all active Windows and shuts down Driver.

#### Signature

```rust
pub async fn close(&self) -> Result<()>
```

#### Returns

`Result<()>` - Ok if all Windows closed successfully.

#### Errors

| Error   | When                      |
| ------- | ------------------------- |
| Various | Any Window fails to close |

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

async fn example(driver: &Driver) -> Result<()> {
    driver.close().await?;
    Ok(())
}
```

---

## DriverBuilder

Builder for configuring Driver.

### `binary`

Sets the path to Firefox binary.

#### Signature

```rust
pub fn binary(self, path: impl Into<PathBuf>) -> Self
```

#### Parameters

| Name   | Type                 | Description                |
| ------ | -------------------- | -------------------------- |
| `path` | `impl Into<PathBuf>` | Path to Firefox executable |

#### Examples

```rust
use firefox_webdriver::Driver;

let builder = Driver::builder()
    .binary("/usr/bin/firefox");
```

---

### `extension`

Sets the path to WebDriver extension.

#### Signature

```rust
pub fn extension(self, path: impl Into<PathBuf>) -> Self
```

#### Parameters

| Name   | Type                 | Description                                |
| ------ | -------------------- | ------------------------------------------ |
| `path` | `impl Into<PathBuf>` | Path to extension directory or `.xpi` file |

#### Examples

```rust
use firefox_webdriver::Driver;

// Unpacked extension directory
let builder = Driver::builder()
    .extension("./extension");

// Packed .xpi file
let builder = Driver::builder()
    .extension("./extension.xpi");
```

---

### `extension_base64`

Sets extension from base64-encoded string.

#### Signature

```rust
pub fn extension_base64(self, data: impl Into<String>) -> Self
```

#### Parameters

| Name   | Type                | Description                   |
| ------ | ------------------- | ----------------------------- |
| `data` | `impl Into<String>` | Base64-encoded `.xpi` content |

#### Examples

```rust
use firefox_webdriver::Driver;

let extension_data = include_str!("extension.b64");
let builder = Driver::builder()
    .extension_base64(extension_data);
```

---

### `build`

Builds Driver with validation.

#### Signature

```rust
pub fn build(self) -> Result<Driver>
```

#### Returns

`Result<Driver>` - Configured Driver instance.

#### Errors

| Error             | When                          |
| ----------------- | ----------------------------- |
| `Config`          | Binary or extension not set   |
| `FirefoxNotFound` | Binary path does not exist    |
| `Config`          | Extension path does not exist |

#### Examples

```rust
use firefox_webdriver::{Driver, Result};

fn example() -> Result<()> {
    let driver = Driver::builder()
        .binary("/usr/bin/firefox")
        .extension("./extension")
        .build().await?;
    Ok(())
}
```

---

## See Also

- [Window](./window.md) - Browser window management
- [Getting Started](../getting-started.md) - Quick start guide

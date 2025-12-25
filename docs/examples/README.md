# Examples

Index of example files in the `examples/` directory.

## Running Examples

```bash
cargo run --example 001_basic_launch
cargo run --example 001_basic_launch -- --no-wait  # Auto-close after run
cargo run --example 001_basic_launch -- --debug    # Enable debug logging
cargo run --example 001_basic_launch -- --clean    # Clean profile (001 only)
```

## Example Index

| File                         | Complexity | Description                                          |
| ---------------------------- | :--------: | ---------------------------------------------------- |
| `001_basic_launch.rs`        |     ⭐     | Driver setup, window spawning, profile management    |
| `002_navigation.rs`          |     ⭐     | Navigate, back, forward, reload, get URL/title       |
| `003_script_execution.rs`    |     ⭐     | Sync/async JavaScript execution                      |
| `004_element_query.rs`       |    ⭐⭐    | Find elements, properties, attributes, nested search |
| `005_element_interaction.rs` |    ⭐⭐    | Click, type, focus, blur, isTrusted events           |
| `006_storage.rs`             |    ⭐⭐    | Cookies, localStorage, sessionStorage                |
| `007_frame_switching.rs`     |   ⭐⭐⭐   | iframe navigation, frame hierarchy                   |
| `008_element_observer.rs`    |   ⭐⭐⭐   | MutationObserver, wait_for_element, callbacks        |
| `009_proxy.rs`               |   ⭐⭐⭐   | HTTP/SOCKS5 proxy, per-tab/window configuration      |
| `010_network_intercept.rs`   |  ⭐⭐⭐⭐  | Block rules, request/response interception           |
| `011_canvas_fingerprint.rs`  |  ⭐⭐⭐⭐  | Canvas randomization verification                    |

## Shared Utilities

All examples use `common.rs` which provides:

- `Args` - Command-line argument parsing (`--debug`, `--no-wait`, `--clean`)
- `init_logging()` - Tracing/logging initialization
- `wait_for_exit()` - Graceful Ctrl+C handling
- `print_logs()` - Extension log output
- `FIREFOX_BINARY` / `EXTENSION_PATH` - Default paths

## Quick Start Example

```rust
use firefox_webdriver::{Driver, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let driver = Driver::builder()
        .binary("./bin/firefox")
        .extension("firefox-webdriver-extension-0.1.2.xpi")
        .build().await?;

    let window = driver.window()
        .window_size(1280, 720)
        .spawn()
        .await?;

    let tab = window.tab();
    tab.goto("https://example.com").await?;

    let title = tab.get_title().await?;
    println!("Title: {}", title);

    driver.close().await?;
    Ok(())
}
```

## See Also

- [Getting Started](../getting-started.md) - Quick start guide
- [API Documentation](../api/) - API reference
- [Guides](../guides/) - How-to guides

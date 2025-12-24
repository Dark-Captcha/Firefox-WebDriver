# Getting Started

Automates Firefox browser using a WebExtension-based architecture.

## Prerequisites

| Requirement | Description                              |
| ----------- | ---------------------------------------- |
| Firefox     | Firefox browser installed                |
| Extension   | Built extension directory or `.xpi` file |
| Rust        | Rust 1.92.0+ with async runtime (tokio)  |

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
firefox_webdriver = "0.1"
tokio = { version = "1", features = ["full"] }
```

## First Script

```rust
use firefox_webdriver::{Driver, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Build Driver with Firefox binary and extension paths
    let driver = Driver::builder()
        .binary("/usr/bin/firefox")
        .extension("./extension")
        .build()
        .await?;

    // Spawn a headless browser Window
    let window = driver.window()
        .headless()
        .window_size(1920, 1080)
        .spawn()
        .await?;

    // Get the initial Tab
    let tab = window.tab();

    // Navigate to a URL
    tab.goto("https://example.com").await?;

    // Get page title
    let title = tab.get_title().await?;
    println!("Page title: {}", title);

    // Find an Element and get text
    let heading = tab.find_element("h1").await?;
    let text = heading.get_text().await?;
    println!("Heading: {}", text);

    // Close Window (kills Firefox process)
    window.close().await?;

    Ok(())
}
```

## Core Concepts

| Concept   | Description                                  |
| --------- | -------------------------------------------- |
| `Driver`  | Factory for creating browser Windows         |
| `Window`  | Owns Firefox process, references shared pool |
| `Tab`     | Browser tab with frame context               |
| `Element` | DOM element reference (UUID-based)           |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Your Rust Code                      │
│                                                          │
│  Driver ──► ConnectionPool ──► Window ──► Tab ──► Element│
└─────────────────────────────────────────────────────────┘
                          │
                          │ WebSocket (shared port)
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   Firefox + Extension                    │
│                                                          │
│  Background Script ◄──► Content Script ◄──► DOM         │
└─────────────────────────────────────────────────────────┘
```

Each Window owns:

- One Firefox process
- Reference to shared ConnectionPool
- One profile directory

## Next Steps

| Guide                                                        | Description                   |
| ------------------------------------------------------------ | ----------------------------- |
| [Driver API](./api/driver.md)                                | Driver configuration          |
| [Window API](./api/window.md)                                | Window management             |
| [Tab API](./api/tab.md)                                      | Navigation, scripts, elements |
| [Element API](./api/element.md)                              | Element interaction           |
| [Element Interaction Guide](./guides/element-interaction.md) | Finding and clicking elements |
| [Waiting Guide](./guides/waiting.md)                         | Event-driven waiting          |

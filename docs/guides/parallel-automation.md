# Parallel Automation Guide

Patterns for running multiple browser instances.

## Problem

You need to automate multiple browser sessions concurrently.

## Solution

```rust
use firefox_webdriver::{Driver, Result};
use tokio::task::JoinSet;

async fn example() -> Result<()> {
    let driver = Driver::builder()
        .binary("/usr/bin/firefox")
        .extension("./extension")
        .build().await?;

    let mut tasks = JoinSet::new();

    // Spawn 10 concurrent browser windows
    for i in 0..10 {
        let driver = driver.clone();
        tasks.spawn(async move {
            let window = driver.window().headless().spawn().await?;
            let tab = window.tab();

            tab.goto(&format!("https://example.com/page/{}", i)).await?;
            let title = tab.get_title().await?;

            window.close().await?;
            Ok::<_, firefox_webdriver::Error>(title)
        });
    }

    // Collect results
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(title)) => println!("Got title: {}", title),
            Ok(Err(e)) => println!("Task error: {}", e),
            Err(e) => println!("Join error: {}", e),
        }
    }

    driver.close().await?;
    Ok(())
}
```

## Architecture

Each Window owns:

- One Firefox process
- Reference to shared ConnectionPool (single WebSocket port)
- One profile directory

Windows are isolated by SessionId, all sharing the same WebSocket server.

```
Driver
├── ConnectionPool (single port, e.g., 9000)
│   ├── Session 1 → Window 1 (Firefox process, profile_1/)
│   ├── Session 2 → Window 2 (Firefox process, profile_2/)
│   └── Session 3 → Window 3 (Firefox process, profile_3/)
```

## Resource Isolation

| Resource     | Isolation                           |
| ------------ | ----------------------------------- |
| Process      | Each Window has own Firefox process |
| Memory       | Separate memory space               |
| Cookies      | Separate profile                    |
| localStorage | Separate profile                    |
| Network      | Independent connections             |

---

## Patterns

### Worker Pool

```rust
use firefox_webdriver::{Driver, Result, Window};
use tokio::sync::mpsc;

async fn worker(window: Window, mut rx: mpsc::Receiver<String>) -> Result<()> {
    let tab = window.tab();

    while let Some(url) = rx.recv().await {
        tab.goto(&url).await?;
        let title = tab.get_title().await?;
        println!("Processed: {} -> {}", url, title);
    }

    window.close().await?;
    Ok(())
}

async fn example(driver: &Driver) -> Result<()> {
    let (tx, rx) = mpsc::channel(100);

    // Spawn worker
    let window = driver.window().headless().spawn().await?;
    tokio::spawn(worker(window, rx));

    // Send work
    for i in 0..10 {
        tx.send(format!("https://example.com/page/{}", i)).await.unwrap();
    }

    Ok(())
}
```

### Batch Processing

```rust
use firefox_webdriver::{Driver, Result};

async fn process_urls(driver: &Driver, urls: Vec<String>, concurrency: usize) -> Result<Vec<String>> {
    use futures::stream::{self, StreamExt};

    let results = stream::iter(urls)
        .map(|url| {
            let driver = driver.clone();
            async move {
                let window = driver.window().headless().spawn().await?;
                let tab = window.tab();

                tab.goto(&url).await?;
                let title = tab.get_title().await?;

                window.close().await?;
                Ok::<_, firefox_webdriver::Error>(title)
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    results.into_iter().collect()
}
```

### Reuse Windows

```rust
use firefox_webdriver::{Driver, Result, Window};

struct BrowserPool {
    windows: Vec<Window>,
}

impl BrowserPool {
    async fn new(driver: &Driver, size: usize) -> Result<Self> {
        let mut windows = Vec::with_capacity(size);
        for _ in 0..size {
            let window = driver.window().headless().spawn().await?;
            windows.push(window);
        }
        Ok(Self { windows })
    }

    fn get(&self, index: usize) -> Option<&Window> {
        self.windows.get(index)
    }

    async fn close_all(&self) -> Result<()> {
        for window in &self.windows {
            window.close().await?;
        }
        Ok(())
    }
}
```

---

## Scaling

Tested capacity: 300+ concurrent Windows on a single machine.

| Factor           | Impact                     |
| ---------------- | -------------------------- |
| RAM              | ~200-500MB per Window      |
| CPU              | Depends on page complexity |
| Ports            | One shared port for all    |
| File descriptors | Multiple per Window        |

### Tips for Scaling

1. Use headless mode (less memory)
2. Block unnecessary resources (images, fonts)
3. Close Windows when done
4. Monitor system resources

---

## Common Mistakes

| Mistake                     | Why Wrong       | Fix                          |
| --------------------------- | --------------- | ---------------------------- |
| Not closing Windows         | Resource leak   | Always call `window.close()` |
| Too many concurrent Windows | OOM             | Limit concurrency            |
| Sharing Tab between tasks   | Race conditions | Each task gets own Window    |

---

## See Also

- [Driver API](../api/driver.md) - Driver methods
- [Window API](../api/window.md) - Window methods

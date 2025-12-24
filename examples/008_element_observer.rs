//! Element observer demonstration.
//!
//! Demonstrates:
//! - wait_for_element (one-shot, blocks until element appears)
//! - on_element_added (persistent callback)
//! - wait_for_element_timeout (timeout handling)
//!
//! Usage:
//!   cargo run --example 008_element_observer
//!   cargo run --example 008_element_observer -- --no-wait
//!   cargo run --example 008_element_observer -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{Driver, Result};

// ============================================================================
// Constants
// ============================================================================

const TEST_URL: &str = "https://example.com";

const BASE_HTML: &str = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Observer Test</title></head>
    <body>
        <h1>Element Observer Test</h1>
        <div id="container"></div>
    </body>
    </html>
"#;

const DYNAMIC_ELEMENTS_SCRIPT: &str = r#"
    // Add element after 2 seconds
    setTimeout(() => {
        const el = document.createElement('div');
        el.id = 'dynamic-element';
        el.textContent = 'I appeared dynamically!';
        el.style.padding = '20px';
        el.style.background = '#4CAF50';
        el.style.color = 'white';
        el.style.marginTop = '20px';
        document.getElementById('container').appendChild(el);
        console.log('Dynamic element added');
    }, 2000);
    
    // Add multiple items after 4 seconds
    setTimeout(() => {
        for (let i = 0; i < 3; i++) {
            const item = document.createElement('div');
            item.className = 'dynamic-item';
            item.textContent = 'Item ' + (i + 1);
            item.style.padding = '10px';
            item.style.margin = '5px';
            item.style.background = '#2196F3';
            item.style.color = 'white';
            document.getElementById('container').appendChild(item);
        }
        console.log('3 dynamic items added');
    }, 4000);
"#;

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    let args = Args::parse();
    common::init_logging(args.debug);

    if let Err(e) = run(args).await {
        eprintln!("\n[ERROR] {e}");
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<()> {
    println!("=== 008: Element Observer ===\n");

    // ========================================================================
    // Setup
    // ========================================================================

    println!("[Setup] Creating driver and window...");

    let driver = Driver::builder()
        .binary(firefox_binary())
        .extension(extension_path())
        .build()
        .await?;

    let window = driver.window().window_size(1280, 720).spawn().await?;
    let tab = window.tab();
    println!(
        "        ✓ Window spawned (session={})\n",
        window.session_id()
    );

    println!("[Setup] Loading test page...");
    tab.goto(TEST_URL).await?;
    sleep(Duration::from_millis(200)).await;

    tab.load_html(BASE_HTML).await?;
    sleep(Duration::from_millis(200)).await;
    println!("        ✓ Base page loaded");

    println!("[Setup] Scheduling dynamic element creation...");
    tab.execute_script(DYNAMIC_ELEMENTS_SCRIPT).await?;
    println!("        ✓ Dynamic elements scheduled\n");

    // ========================================================================
    // wait_for_element (one-shot)
    // ========================================================================

    println!("[1] wait_for_element('#dynamic-element')");
    println!("    Element will appear in ~2 seconds...");

    let start = std::time::Instant::now();
    let element = tab.wait_for_element("#dynamic-element").await?;
    let elapsed = start.elapsed();

    println!("    ✓ Element found in {elapsed:?}");

    let text = element.get_text().await?;
    println!("    Text: '{text}'\n");

    // ========================================================================
    // on_element_added (persistent callback)
    // ========================================================================

    println!("[2] on_element_added('.dynamic-item')");
    println!("    Items will appear in ~2 seconds...");

    let item_count = Arc::new(AtomicUsize::new(0));
    let item_count_clone = Arc::clone(&item_count);

    let subscription_id = tab
        .on_element_added(".dynamic-item", move |_element| {
            let count = item_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
            println!("    → Callback: Item #{count} added!");
        })
        .await?;

    println!("    ✓ Subscribed (id: {subscription_id})");

    sleep(Duration::from_secs(5)).await;

    let final_count = item_count.load(Ordering::SeqCst);
    println!("    ✓ Total items received: {final_count}\n");

    tab.unsubscribe(&subscription_id).await?;
    println!("    ✓ Unsubscribed\n");

    // ========================================================================
    // wait_for_element_timeout (should timeout)
    // ========================================================================

    println!("[3] wait_for_element_timeout (expect timeout)");
    println!("    Waiting 2s for non-existent element...");

    let result = tab
        .wait_for_element_timeout("#nonexistent-element", Duration::from_secs(2))
        .await;

    match result {
        Ok(_) => println!("    ✗ Unexpectedly found element!"),
        Err(e) => println!("    ✓ Correctly timed out: {e}\n"),
    }

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All element observer tests complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

//! JavaScript execution demonstration.
//!
//! Demonstrates:
//! - Synchronous script execution
//! - Asynchronous script execution (Promises)
//! - Returning values from scripts
//! - DOM manipulation via scripts
//!
//! Usage:
//!   cargo run --example 003_script_execution
//!   cargo run --example 003_script_execution -- --no-wait
//!   cargo run --example 003_script_execution -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{Driver, Result};

// ============================================================================
// Constants
// ============================================================================

const TEST_URL: &str = "https://example.com";

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
    println!("=== 003: Script Execution ===\n");

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

    println!("[Setup] Navigating to {TEST_URL}...");
    tab.goto(TEST_URL).await?;
    println!("        ✓ Navigated\n");

    // ========================================================================
    // Sync: Basic arithmetic
    // ========================================================================

    println!("[1] Sync script: return 2 + 2");
    let result = tab.execute_script("return 2 + 2").await?;
    println!("    Result: {result}");
    assert_eq!(result, serde_json::json!(4), "2 + 2 should equal 4");
    println!("    ✓ Passed\n");

    // ========================================================================
    // Sync: Get document title
    // ========================================================================

    println!("[2] Sync script: return document.title");
    let result = tab.execute_script("return document.title").await?;
    println!("    Result: {result}");
    assert!(result.as_str().is_some(), "Should return string");
    println!("    ✓ Passed\n");

    // ========================================================================
    // Sync: Return object
    // ========================================================================

    println!("[3] Sync script: return object");
    let result = tab
        .execute_script("return { name: 'test', value: 42, nested: { a: 1 } }")
        .await?;
    println!("    Result: {result}");
    assert_eq!(result["name"], "test");
    assert_eq!(result["value"], 42);
    assert_eq!(result["nested"]["a"], 1);
    println!("    ✓ Passed\n");

    // ========================================================================
    // Sync: Return array
    // ========================================================================

    println!("[4] Sync script: return array");
    let result = tab.execute_script("return [1, 2, 3, 'four', true]").await?;
    println!("    Result: {result}");
    assert!(result.is_array());
    assert_eq!(result.as_array().map(|a| a.len()), Some(5));
    println!("    ✓ Passed\n");

    // ========================================================================
    // Async: Promise.resolve
    // ========================================================================

    println!("[5] Async script: Promise.resolve(42)");
    let result = tab
        .execute_async_script("return await Promise.resolve(42)")
        .await?;
    println!("    Result: {result}");
    assert_eq!(result, serde_json::json!(42));
    println!("    ✓ Passed\n");

    // ========================================================================
    // Async: Delayed promise
    // ========================================================================

    println!("[6] Async script: delayed promise (100ms)");
    let result = tab
        .execute_async_script(
            r#"
            return await new Promise(resolve => {
                setTimeout(() => resolve('delayed'), 100);
            });
        "#,
        )
        .await?;
    println!("    Result: {result}");
    assert_eq!(result, serde_json::json!("delayed"));
    println!("    ✓ Passed\n");

    // ========================================================================
    // Async: Fetch simulation
    // ========================================================================

    println!("[7] Async script: simulated async operation");
    let result = tab
        .execute_async_script(
            r#"
            const data = await Promise.resolve({ status: 'ok', items: [1, 2, 3] });
            return data;
        "#,
        )
        .await?;
    println!("    Result: {result}");
    assert_eq!(result["status"], "ok");
    println!("    ✓ Passed\n");

    // ========================================================================
    // DOM manipulation
    // ========================================================================

    println!("[8] DOM manipulation: add element");
    tab.execute_script(
        r#"
        const div = document.createElement('div');
        div.id = 'injected';
        div.textContent = 'Injected by script';
        document.body.appendChild(div);
    "#,
    )
    .await?;

    let result = tab
        .execute_script("return document.getElementById('injected').textContent")
        .await?;
    println!("    Injected element text: {result}");
    assert_eq!(result, serde_json::json!("Injected by script"));
    println!("    ✓ Passed\n");

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All script tests passed ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

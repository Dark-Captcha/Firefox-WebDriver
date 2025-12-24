//! Storage operations demonstration.
//!
//! Demonstrates:
//! - Cookie operations (get, set, delete, getAll)
//! - localStorage operations (get, set, delete, clear)
//! - sessionStorage operations (get, set, delete, clear)
//!
//! Usage:
//!   cargo run --example 006_storage
//!   cargo run --example 006_storage -- --no-wait
//!   cargo run --example 006_storage -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{Cookie, Driver, Result};

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
    println!("=== 006: Storage ===\n");

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
    sleep(Duration::from_millis(1000)).await;
    println!("        ✓ Page loaded\n");

    test_cookies(&tab).await?;
    test_local_storage(&tab).await?;
    test_session_storage(&tab).await?;

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 20).await?;

    println!("\n=== All storage tests complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

// ============================================================================
// Cookie Tests
// ============================================================================

async fn test_cookies(tab: &firefox_webdriver::Tab) -> Result<()> {
    println!("[1] Cookie operations\n");

    // Set cookie
    let cookie = Cookie::new("test_session", "abc123")
        .with_path("/")
        .with_secure(false);
    tab.set_cookie(cookie).await?;
    println!("    ✓ Set cookie: test_session=abc123");

    // Get cookie
    let retrieved = tab.get_cookie("test_session").await?;
    if let Some(c) = &retrieved {
        println!("    ✓ Got cookie: {}={}", c.name, c.value);
        assert_eq!(c.value, "abc123", "Cookie value mismatch");
    } else {
        println!("    ✗ Cookie not found");
    }

    // Set another cookie
    let cookie2 = Cookie::new("user_id", "12345");
    tab.set_cookie(cookie2).await?;
    println!("    ✓ Set cookie: user_id=12345");

    // Get all cookies
    let all_cookies = tab.get_all_cookies().await?;
    println!("    ✓ Got {} cookies total", all_cookies.len());
    for c in &all_cookies {
        println!("      - {}={}", c.name, c.value);
    }

    // Delete cookie
    tab.delete_cookie("test_session").await?;
    println!("    ✓ Deleted cookie: test_session");

    let deleted = tab.get_cookie("test_session").await?;
    if deleted.is_none() {
        println!("    ✓ Cookie successfully deleted");
    } else {
        println!("    ✗ Cookie still exists after deletion");
    }

    // Cleanup
    tab.delete_cookie("user_id").await?;
    println!("    ✓ Cleaned up test cookies\n");

    Ok(())
}

// ============================================================================
// localStorage Tests
// ============================================================================

async fn test_local_storage(tab: &firefox_webdriver::Tab) -> Result<()> {
    println!("[2] localStorage operations\n");

    // Set value
    tab.local_storage_set("test_key", "test_value").await?;
    println!("    ✓ Set: test_key=test_value");

    // Get value
    let value = tab.local_storage_get("test_key").await?;
    if let Some(v) = &value {
        println!("    ✓ Got: test_key={v}");
        assert_eq!(v, "test_value", "Value mismatch");
    } else {
        println!("    ✗ Key not found");
    }

    // Non-existent key
    let missing = tab.local_storage_get("non_existent_key").await?;
    if missing.is_none() {
        println!("    ✓ Non-existent key returns None");
    }

    // Set multiple keys
    tab.local_storage_set("key1", "value1").await?;
    tab.local_storage_set("key2", "value2").await?;
    tab.local_storage_set("key3", "value3").await?;
    println!("    ✓ Set multiple keys: key1, key2, key3");

    // Delete one key
    tab.local_storage_delete("key2").await?;
    println!("    ✓ Deleted key2");

    let deleted = tab.local_storage_get("key2").await?;
    if deleted.is_none() {
        println!("    ✓ key2 successfully deleted");
    }

    // Verify others still exist
    let key1 = tab.local_storage_get("key1").await?;
    let key3 = tab.local_storage_get("key3").await?;
    if key1.is_some() && key3.is_some() {
        println!("    ✓ Other keys still exist");
    }

    // Clear all
    tab.local_storage_clear().await?;
    println!("    ✓ Cleared localStorage");

    let after_clear = tab.local_storage_get("key1").await?;
    if after_clear.is_none() {
        println!("    ✓ localStorage successfully cleared\n");
    }

    Ok(())
}

// ============================================================================
// sessionStorage Tests
// ============================================================================

async fn test_session_storage(tab: &firefox_webdriver::Tab) -> Result<()> {
    println!("[3] sessionStorage operations\n");

    // Set value
    tab.session_storage_set("session_key", "session_value")
        .await?;
    println!("    ✓ Set: session_key=session_value");

    // Get value
    let value = tab.session_storage_get("session_key").await?;
    if let Some(v) = &value {
        println!("    ✓ Got: session_key={v}");
        assert_eq!(v, "session_value", "Value mismatch");
    } else {
        println!("    ✗ Key not found");
    }

    // Non-existent key
    let missing = tab.session_storage_get("non_existent_session_key").await?;
    if missing.is_none() {
        println!("    ✓ Non-existent key returns None");
    }

    // Set multiple keys
    tab.session_storage_set("skey1", "svalue1").await?;
    tab.session_storage_set("skey2", "svalue2").await?;
    tab.session_storage_set("skey3", "svalue3").await?;
    println!("    ✓ Set multiple keys: skey1, skey2, skey3");

    // Delete one key
    tab.session_storage_delete("skey2").await?;
    println!("    ✓ Deleted skey2");

    let deleted = tab.session_storage_get("skey2").await?;
    if deleted.is_none() {
        println!("    ✓ skey2 successfully deleted");
    }

    // Verify others still exist
    let skey1 = tab.session_storage_get("skey1").await?;
    let skey3 = tab.session_storage_get("skey3").await?;
    if skey1.is_some() && skey3.is_some() {
        println!("    ✓ Other keys still exist");
    }

    // Clear all
    tab.session_storage_clear().await?;
    println!("    ✓ Cleared sessionStorage");

    let after_clear = tab.session_storage_get("skey1").await?;
    if after_clear.is_none() {
        println!("    ✓ sessionStorage successfully cleared\n");
    }

    Ok(())
}

//! Navigation commands demonstration.
//!
//! Demonstrates:
//! - Navigate to URL (goto)
//! - Get current URL and title
//! - History navigation (back, forward)
//! - Page reload
//!
//! Usage:
//!   cargo run --example 002_navigation
//!   cargo run --example 002_navigation -- --no-wait
//!   cargo run --example 002_navigation -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, EXTENSION_PATH, FIREFOX_BINARY};
use firefox_webdriver::{Driver, Result};

// ============================================================================
// Constants
// ============================================================================

const URL_1: &str = "https://example.com";
const URL_2: &str = "https://httpbin.org/html";

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
    println!("=== 002: Navigation ===\n");

    // ========================================================================
    // Setup
    // ========================================================================

    println!("[Setup] Creating driver and window...");

    let driver = Driver::builder()
        .binary(FIREFOX_BINARY)
        .extension(EXTENSION_PATH)
        .build()?;

    let window = driver.window().window_size(1280, 720).spawn().await?;
    let tab = window.tab();
    println!(
        "        ✓ Window spawned (session={})\n",
        window.session_id()
    );

    // ========================================================================
    // Navigate to URL
    // ========================================================================

    println!("[1] Navigate to {URL_1}...");
    tab.goto(URL_1).await?;
    println!("    ✓ Navigated");

    let url = tab.get_url().await?;
    println!("    URL: {url}");
    assert!(
        url.contains("example.com"),
        "URL should contain example.com"
    );

    let title = tab.get_title().await?;
    println!("    Title: {title}");
    assert!(!title.is_empty(), "Title should not be empty");
    println!();

    // ========================================================================
    // Navigate to second URL
    // ========================================================================

    println!("[2] Navigate to {URL_2}...");
    tab.goto(URL_2).await?;
    println!("    ✓ Navigated");

    let url = tab.get_url().await?;
    println!("    URL: {url}\n");

    // ========================================================================
    // History: Back
    // ========================================================================

    println!("[3] Go back...");
    tab.back().await?;
    sleep(Duration::from_millis(500)).await;

    let url = tab.get_url().await?;
    println!("    ✓ Back to: {url}");
    assert!(url.contains("example.com"), "Should be back at example.com");
    println!();

    // ========================================================================
    // History: Forward
    // ========================================================================

    println!("[4] Go forward...");
    tab.forward().await?;
    sleep(Duration::from_millis(500)).await;

    let url = tab.get_url().await?;
    println!("    ✓ Forward to: {url}");
    assert!(url.contains("httpbin"), "Should be forward at httpbin");
    println!();

    // ========================================================================
    // Reload
    // ========================================================================

    println!("[5] Reload page...");
    tab.reload().await?;
    println!("    ✓ Reloaded\n");

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All navigation tests passed ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

//! Mouse actions demonstration.
//!
//! Demonstrates:
//! - Click, double-click
//! - Hover (mouse_move)
//! - Mouse down/up for drag operations
//! - Scroll into view
//! - Focus and blur
//! - Using By selectors
//!
//! Usage:
//!   cargo run --example 016_mouse
//!   cargo run --example 016_mouse -- --no-wait
//!   cargo run --example 016_mouse -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{By, Driver, Result};

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
    println!("=== 016: Mouse Actions ===\n");

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

    // ========================================================================
    // Click
    // ========================================================================

    println!("[1] Click action...");
    tab.goto("https://the-internet.herokuapp.com/add_remove_elements/")
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Using By::css() for attribute selector
    let add_btn = tab
        .find_element(By::css("button[onclick='addElement()']"))
        .await?;

    // Click 3 times to add elements
    add_btn.click().await?;
    add_btn.click().await?;
    add_btn.click().await?;

    // Using By::class() to find added elements
    let elements = tab.find_elements(By::class("added-manually")).await?;
    println!("    Elements after 3 clicks: {}", elements.len());
    assert_eq!(elements.len(), 3);

    // Click to remove one
    if !elements.is_empty() {
        elements[0].click().await?;
        let remaining = tab.find_elements(By::class("added-manually")).await?;
        println!("    After removing one: {}", remaining.len());
        assert_eq!(remaining.len(), 2);
    }
    println!("    ✓ Passed\n");

    // ========================================================================
    // Mouse click with button
    // ========================================================================

    println!("[2] Mouse click with button...");
    let add_btn = tab
        .find_element(By::css("button[onclick='addElement()']"))
        .await?;

    // Left click (button 0)
    add_btn.mouse_click(0).await?;
    let count = tab.find_elements(By::class("added-manually")).await?.len();
    println!("    After left click (button=0): {count} elements");
    println!("    ✓ Passed\n");

    // ========================================================================
    // Double-click
    // ========================================================================

    println!("[3] Double-click...");
    tab.goto("https://the-internet.herokuapp.com/").await?;
    sleep(Duration::from_millis(500)).await;

    let link = tab
        .find_element(By::css("a[href='/add_remove_elements/']"))
        .await?;
    link.double_click().await?;
    println!("    Double-click dispatched on link");
    println!("    ✓ Passed\n");

    // ========================================================================
    // Hover
    // ========================================================================

    println!("[4] Hover (mouse_move)...");
    tab.goto("https://the-internet.herokuapp.com/hovers")
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Using By::class()
    let figures = tab.find_elements(By::class("figure")).await?;
    println!("    Found {} hoverable figures", figures.len());

    if !figures.is_empty() {
        figures[0].hover().await?;
        sleep(Duration::from_millis(300)).await;
        println!("    Hovered over first figure");
    }

    // Steal logs right after hover to see what happened
    println!("\n    [Hover Logs]:");
    let logs = window.steal_logs().await?;
    for log in logs.iter().rev().take(15).rev() {
        if let Some(msg) = log.get("message").and_then(|v| v.as_str()) {
            let level = log.get("level").and_then(|v| v.as_str()).unwrap_or("?");
            let module = log.get("module").and_then(|v| v.as_str()).unwrap_or("?");
            println!("        [{level}] [{module}] {msg}");
        }
    }
    println!();
    println!("    ✓ Passed\n");

    // ========================================================================
    // Scroll into view
    // ========================================================================

    println!("[5] Scroll into view...");
    tab.goto("https://the-internet.herokuapp.com/large").await?;
    sleep(Duration::from_millis(500)).await;

    // Find element at bottom of page using By::id()
    let bottom = tab.find_element(By::id("page-footer")).await?;

    // Get initial position
    let (_, y_before, _, _) = bottom.get_bounding_rect().await?;
    println!("    Element Y before scroll: {y_before:.0}");

    // Scroll into view
    bottom.scroll_into_view().await?;
    sleep(Duration::from_millis(500)).await;

    let (_, y_after, _, _) = bottom.get_bounding_rect().await?;
    println!("    Element Y after scroll: {y_after:.0}");
    println!("    ✓ Passed\n");

    // ========================================================================
    // Focus and blur
    // ========================================================================

    println!("[6] Focus and blur...");
    tab.goto("https://the-internet.herokuapp.com/login").await?;
    sleep(Duration::from_millis(500)).await;

    // Using By::id()
    let username = tab.find_element(By::id("username")).await?;
    let password = tab.find_element(By::id("password")).await?;

    // Focus username, type, blur
    username.focus().await?;
    username.type_text("testuser").await?;
    username.blur().await?;
    println!("    Username: focused → typed → blurred");

    // Focus password, type, blur
    password.focus().await?;
    password.type_text("secret123").await?;
    password.blur().await?;
    println!("    Password: focused → typed → blurred");

    println!("    ✓ Passed\n");

    // ========================================================================
    // Mouse down/up
    // ========================================================================

    println!("[7] Mouse down/up...");
    tab.goto("https://example.com").await?;
    sleep(Duration::from_millis(500)).await;

    // Using By::tag()
    let heading = tab.find_element(By::tag("h1")).await?;

    // Mouse down (button 0 = left)
    heading.mouse_down(0).await?;
    println!("    Mouse down on heading");

    // Mouse up
    heading.mouse_up(0).await?;
    println!("    Mouse up on heading");

    println!("    ✓ Passed\n");

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All mouse tests passed ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

//! Element querying demonstration.
//!
//! Demonstrates:
//! - Find single element (find_element)
//! - Find multiple elements (find_elements)
//! - Get element properties and attributes
//! - Nested element search
//! - Error handling for missing elements
//!
//! Usage:
//!   cargo run --example 004_element_query
//!   cargo run --example 004_element_query -- --no-wait
//!   cargo run --example 004_element_query -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use common::{Args, EXTENSION_PATH, FIREFOX_BINARY};
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
    println!("=== 004: Element Query ===\n");

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

    println!("[Setup] Navigating to {TEST_URL}...");
    tab.goto(TEST_URL).await?;
    println!("        ✓ Navigated\n");

    // ========================================================================
    // Find single element
    // ========================================================================

    println!("[1] find_element('h1')");
    let h1 = tab.find_element("h1").await?;
    println!("    ✓ Found element: {}", h1.id());

    let text = h1.get_text().await?;
    println!("    Text: '{text}'");
    assert!(!text.is_empty(), "h1 should have text");
    println!();

    // ========================================================================
    // Find link element
    // ========================================================================

    println!("[2] find_element('a')");
    let link = tab.find_element("a").await?;
    println!("    ✓ Found link: {}", link.id());
    println!();

    // ========================================================================
    // Find multiple elements
    // ========================================================================

    println!("[3] find_elements('p')");
    let paragraphs = tab.find_elements("p").await?;
    println!("    ✓ Found {} paragraph(s)", paragraphs.len());

    for (i, p) in paragraphs.iter().enumerate() {
        let text = p.get_text().await?;
        let preview = if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text
        };
        println!("    p[{i}]: {preview}");
    }
    println!();

    // ========================================================================
    // Get element properties
    // ========================================================================

    println!("[4] Element properties");

    let tag_name = link.get_property("tagName").await?;
    println!("    tagName: {tag_name}");

    let inner_html = link.get_inner_html().await?;
    println!("    innerHTML: {inner_html}");

    let href = link.get_attribute("href").await?;
    println!("    href: {:?}", href);
    println!();

    // ========================================================================
    // Nested element search
    // ========================================================================

    println!("[5] Nested element search");

    let body = tab.find_element("body").await?;
    println!("    ✓ Found body");

    let nested_h1 = body.find_element("h1").await?;
    let nested_text = nested_h1.get_text().await?;
    println!("    ✓ body.find('h1') text: '{nested_text}'");
    println!();

    // ========================================================================
    // Error handling: element not found
    // ========================================================================

    println!("[6] Error handling: element not found");

    let result = tab.find_element("#nonexistent-element-12345").await;
    match result {
        Ok(_) => println!("    ✗ Should have failed!"),
        Err(e) => println!("    ✓ Correctly failed: {e}"),
    }
    println!();

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All element query tests passed ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

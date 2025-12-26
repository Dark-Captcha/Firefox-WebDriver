//! Form elements demonstration.
//!
//! Demonstrates:
//! - Select/dropdown interaction (select_by_text, select_by_value, select_by_index)
//! - Checkbox interaction (check, uncheck, toggle, is_checked)
//! - Getting selected values
//! - Using By selectors (By::Id, By::css, etc.)
//!
//! Usage:
//!   cargo run --example 014_forms
//!   cargo run --example 014_forms -- --no-wait
//!   cargo run --example 014_forms -- --debug

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
    println!("=== 014: Form Elements ===\n");

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
    // Select/Dropdown
    // ========================================================================

    println!("[1] Select/Dropdown interaction...");
    tab.goto("https://the-internet.herokuapp.com/dropdown")
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Using By::id() selector
    let dropdown = tab.find_element(By::id("dropdown")).await?;
    println!("    Found dropdown with By::id(\"dropdown\")");

    // Check if multi-select
    let is_multiple = dropdown.is_multiple().await?;
    println!("    Is multi-select: {is_multiple}");

    // Select by text
    dropdown.select_by_text("Option 1").await?;
    let value = dropdown.get_selected_value().await?;
    println!("    Selected by text 'Option 1': {:?}", value);

    // Select by value
    dropdown.select_by_value("2").await?;
    let value = dropdown.get_selected_value().await?;
    println!("    Selected by value '2': {:?}", value);

    // Select by index
    dropdown.select_by_index(1).await?;
    let index = dropdown.get_selected_index().await?;
    println!("    Selected by index 1: index={index}");

    // Get selected text
    let text = dropdown.get_selected_text().await?;
    println!("    Selected text: {:?}\n", text);

    // ========================================================================
    // Checkboxes
    // ========================================================================

    println!("[2] Checkbox interaction...");
    tab.goto("https://the-internet.herokuapp.com/checkboxes")
        .await?;
    sleep(Duration::from_millis(500)).await;

    // Using By::css() for attribute selector
    let checkboxes = tab.find_elements(By::css("input[type='checkbox']")).await?;
    println!("    Found {} checkboxes with By::css()", checkboxes.len());

    if checkboxes.len() >= 2 {
        let cb1 = &checkboxes[0];
        let cb2 = &checkboxes[1];

        // Initial state
        let cb1_checked = cb1.is_checked().await?;
        let cb2_checked = cb2.is_checked().await?;
        println!("    Initial: cb1={cb1_checked}, cb2={cb2_checked}");

        // Check cb1
        cb1.check().await?;
        println!("    After cb1.check(): {}", cb1.is_checked().await?);

        // Toggle cb2
        cb2.toggle().await?;
        println!("    After cb2.toggle(): {}", cb2.is_checked().await?);

        // Uncheck cb1
        cb1.uncheck().await?;
        println!("    After cb1.uncheck(): {}", cb1.is_checked().await?);

        // Set checked state
        cb1.set_checked(true).await?;
        println!(
            "    After cb1.set_checked(true): {}",
            cb1.is_checked().await?
        );
    }
    println!();

    // ========================================================================
    // Text Input with By selectors
    // ========================================================================

    println!("[3] Text input with By selectors...");
    tab.goto("https://the-internet.herokuapp.com/login").await?;
    sleep(Duration::from_millis(500)).await;

    // Using By::id() - cleaner than "#username"
    let username = tab.find_element(By::id("username")).await?;
    let password = tab.find_element(By::id("password")).await?;

    // Type text
    username.type_text("tomsmith").await?;
    println!("    Username (By::id): {:?}", username.get_value().await?);

    password.type_text("SuperSecretPassword!").await?;
    println!("    Password (By::id): {:?}", password.get_value().await?);

    // Clear and set value
    username.clear().await?;
    username.set_value("newuser").await?;
    println!(
        "    After clear + set_value: {:?}",
        username.get_value().await?
    );

    // Find submit button by tag
    let buttons = tab.find_elements(By::tag("button")).await?;
    println!("    Found {} buttons with By::tag()", buttons.len());
    println!();

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All form tests passed ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

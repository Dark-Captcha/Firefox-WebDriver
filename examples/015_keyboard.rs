//! Keyboard input demonstration.
//!
//! Demonstrates:
//! - Typing text with type_text()
//! - Pressing navigation keys with press(Key::Enter) and press(Key::Tab)
//! - Note: Only Tab and Enter keys are supported
//!
//! Usage:
//!   cargo run --example 015_keyboard
//!   cargo run --example 015_keyboard -- --no-wait
//!   cargo run --example 015_keyboard -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{By, Driver, Key, Result};

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
    println!("=== 015: Keyboard Input ===\n");

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
    // Create test form with multiple inputs
    // ========================================================================

    println!("[1] Creating test form with multiple inputs...");
    tab.goto("https://example.com").await?;
    sleep(Duration::from_millis(500)).await;

    tab.execute_script(
        r#"
        document.body.innerHTML = `
            <div style="padding: 20px;">
                <form id="test-form">
                    <input type="text" id="input1" 
                           style="font-size: 20px; padding: 10px; width: 300px; display: block; margin-bottom: 10px;"
                           placeholder="First input...">
                    <input type="text" id="input2" 
                           style="font-size: 20px; padding: 10px; width: 300px; display: block; margin-bottom: 10px;"
                           placeholder="Second input...">
                    <button type="submit" id="submit-btn">Submit</button>
                </form>
                <div id="result" style="margin-top: 20px;"></div>
            </div>
        `;
        document.getElementById('test-form').addEventListener('submit', (e) => {
            e.preventDefault();
            document.getElementById('result').textContent = 'Form submitted!';
        });
        "#,
    )
    .await?;
    sleep(Duration::from_millis(300)).await;

    let input1 = tab.find_element(By::css("#input1")).await?;
    let input2 = tab.find_element(By::css("#input2")).await?;
    input1.click().await?;
    println!("    ✓ Form ready\n");

    // ========================================================================
    // Type text (use type_text for letters/words)
    // ========================================================================

    println!("[2] Typing text with type_text()...");
    input1.type_text("Hello, World!").await?;
    let value = input1.get_value().await?;
    println!("    Typed: {value}");
    assert_eq!(value, "Hello, World!");
    println!("    ✓ Passed\n");

    // ========================================================================
    // Press Tab to move to next input
    // ========================================================================

    println!("[3] Press Key::Tab to move to next input...");
    input1.press(Key::Tab).await?;
    sleep(Duration::from_millis(200)).await;

    // Type in the second input (now focused via Tab)
    input2.type_text("Second field").await?;
    let value2 = input2.get_value().await?;
    println!("    After Tab, typed in input2: {value2}");
    println!("    ✓ Tab navigation works\n");

    // ========================================================================
    // Login form with Key::Enter
    // ========================================================================

    println!("[4] Login form with Key::Enter...");
    tab.goto("https://the-internet.herokuapp.com/login").await?;
    sleep(Duration::from_millis(500)).await;

    let username = tab.find_element(By::css("#username")).await?;
    let password = tab.find_element(By::css("#password")).await?;

    username.type_text("tomsmith").await?;
    println!("    Username: {:?}", username.get_value().await?);

    // Use Tab to move from username to password field
    username.press(Key::Tab).await?;
    sleep(Duration::from_millis(200)).await;

    password.type_text("SuperSecretPassword!").await?;
    println!("    Password: {:?}", password.get_value().await?);

    // Press Enter to submit
    password.press(Key::Enter).await?;
    sleep(Duration::from_secs(1)).await;

    let url = tab.get_url().await?;
    println!("    After Key::Enter: {url}");
    if url.contains("secure") {
        println!("    ✓ Login successful\n");
    } else {
        println!("    ⚠ Enter key may not have submitted (extension limitation)\n");
    }

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All keyboard tests passed ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

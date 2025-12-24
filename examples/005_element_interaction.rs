//! Element interaction demonstration.
//!
//! Demonstrates:
//! - Typing text into input fields
//! - Clicking elements
//! - Focus and blur
//! - Clearing input fields
//! - isTrusted event verification
//!
//! Usage:
//!   cargo run --example 005_element_interaction
//!   cargo run --example 005_element_interaction -- --no-wait
//!   cargo run --example 005_element_interaction -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{Driver, Result};

// ============================================================================
// Constants
// ============================================================================

const TEST_URL: &str = "https://example.com";

const TEST_ELEMENTS_SCRIPT: &str = r#"
    const container = document.createElement('div');
    container.innerHTML = `
        <div id="trusted-log" style="background: #f0f0f0; padding: 10px; margin-bottom: 10px; font-family: monospace; font-size: 12px;">
            Event log (isTrusted status):
        </div>
        <input id="test-input" type="text" style="padding: 8px; width: 300px;">
        <button id="test-btn" style="padding: 10px 20px;">Click Me</button>
    `;
    document.body.insertBefore(container, document.body.firstChild);
    
    const logEl = document.getElementById('trusted-log');
    const input = document.getElementById('test-input');
    const btn = document.getElementById('test-btn');
    
    function logEvent(e) {
        const status = e.isTrusted ? '✓ TRUSTED' : '✗ NOT TRUSTED';
        const color = e.isTrusted ? 'green' : 'red';
        logEl.innerHTML += `<br><span style="color: ${color}">[${e.type}] isTrusted=${e.isTrusted} ${status}</span>`;
    }
    
    ['keydown', 'keypress', 'keyup', 'input', 'focus', 'blur'].forEach(evt => {
        input.addEventListener(evt, logEvent);
    });
    
    ['mousedown', 'mouseup', 'click', 'mouseover', 'mouseenter'].forEach(evt => {
        btn.addEventListener(evt, logEvent);
    });
"#;

const CLICK_HANDLER_SCRIPT: &str = r#"
    document.getElementById('test-btn').addEventListener('click', function() {
        this.textContent = 'Clicked!';
    });
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
    println!("=== 005: Element Interaction ===\n");

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
    sleep(Duration::from_millis(500)).await;

    tab.execute_script(TEST_ELEMENTS_SCRIPT).await?;
    sleep(Duration::from_millis(200)).await;

    let title = tab.get_title().await?;
    println!("        Page title: {title}");
    println!("        ✓ Test page ready\n");

    // ========================================================================
    // Type text
    // ========================================================================

    println!("[1] type_text: Type into input field");

    let input = tab.find_element("#test-input").await?;
    input.focus().await?;
    input.type_text("Hello World").await?;

    let value = input.get_value().await?;
    println!("    Input value: \"{value}\"");

    if value == "Hello World" {
        println!("    ✓ type_text works\n");
    } else {
        println!("    ✗ Expected \"Hello World\", got \"{value}\"\n");
    }

    // ========================================================================
    // Type single character
    // ========================================================================

    println!("[2] type_char: Type single character");

    input.type_char('!').await?;

    let value = input.get_value().await?;
    println!("    Input value: \"{value}\"");

    if value == "Hello World!" {
        println!("    ✓ type_char works\n");
    } else {
        println!("    ✗ Expected \"Hello World!\", got \"{value}\"\n");
    }

    // ========================================================================
    // Clear and type
    // ========================================================================

    println!("[3] clear + type_text");

    input.clear().await?;
    input.type_text("New text").await?;

    let value = input.get_value().await?;
    println!("    Input value: \"{value}\"");

    if value == "New text" {
        println!("    ✓ clear + type_text works\n");
    } else {
        println!("    ✗ Expected \"New text\", got \"{value}\"\n");
    }

    // ========================================================================
    // Mouse click
    // ========================================================================

    println!("[4] mouse_click: Click button");

    tab.execute_script(CLICK_HANDLER_SCRIPT).await?;

    let button = tab.find_element("#test-btn").await?;
    let text_before = button.get_text().await?;
    println!("    Button text before: \"{text_before}\"");

    button.mouse_click(0).await?;
    sleep(Duration::from_millis(100)).await;

    let text_after = button.get_text().await?;
    println!("    Button text after: \"{text_after}\"");

    if text_after == "Clicked!" {
        println!("    ✓ mouse_click works\n");
    } else {
        println!("    ✗ Expected \"Clicked!\", got \"{text_after}\"\n");
    }

    // ========================================================================
    // isTrusted verification
    // ========================================================================

    println!("[5] isTrusted verification");

    let trusted_log: String = tab
        .execute_script("return document.getElementById('trusted-log').innerText;")
        .await?
        .as_str()
        .unwrap_or("")
        .to_string();

    println!("    Event log:");
    for line in trusted_log.lines().take(10) {
        println!("      {line}");
    }

    let all_trusted: bool = tab
        .execute_script(
            "const log = document.getElementById('trusted-log').innerHTML; \
             return !log.includes('NOT TRUSTED');",
        )
        .await?
        .as_bool()
        .unwrap_or(false);

    if all_trusted {
        println!("    ✓ All events have isTrusted=true\n");
    } else {
        println!("    ⚠ Some events have isTrusted=false\n");
    }

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== Element interaction tests complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

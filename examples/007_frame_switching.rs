//! Frame switching demonstration.
//!
//! Demonstrates:
//! - Get frame count
//! - Get all frames
//! - Switch to frame by element
//! - Switch to parent frame
//! - Switch to frame by index
//! - Switch to main frame
//!
//! Usage:
//!   cargo run --example 007_frame_switching
//!   cargo run --example 007_frame_switching -- --no-wait
//!   cargo run --example 007_frame_switching -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{By, Driver, Result};

// ============================================================================
// Constants
// ============================================================================

const TEST_URL: &str = "https://example.com";

const FRAME_TEST_HTML: &str = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Frame Test</title></head>
    <body>
        <h1>Frame Test Page</h1>
        <p id="main-content">Main frame content</p>
        <iframe id="frame1" src="about:blank" style="width:400px;height:100px;border:2px solid blue;"></iframe>
        <iframe id="frame2" src="about:blank" style="width:400px;height:100px;border:2px solid green;"></iframe>
    </body>
    </html>
"#;

const INJECT_FRAME_CONTENT_SCRIPT: &str = r#"
    const frame1 = document.getElementById('frame1');
    const frame2 = document.getElementById('frame2');
    if (frame1 && frame1.contentDocument) {
        frame1.contentDocument.body.innerHTML = '<h2>Frame 1</h2><p id="frame1-text">Hello from frame 1!</p>';
    }
    if (frame2 && frame2.contentDocument) {
        frame2.contentDocument.body.innerHTML = '<h2>Frame 2</h2><p id="frame2-text">Hello from frame 2!</p>';
    }
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
    println!("=== 007: Frame Switching ===\n");

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

    println!("[Setup] Loading test page with iframes...");
    tab.goto(TEST_URL).await?;
    sleep(Duration::from_millis(200)).await;

    tab.load_html(FRAME_TEST_HTML).await?;
    sleep(Duration::from_millis(200)).await;

    tab.execute_script(INJECT_FRAME_CONTENT_SCRIPT).await?;
    sleep(Duration::from_millis(200)).await;
    println!("        ✓ Test page with iframes loaded\n");

    // ========================================================================
    // Get frame count
    // ========================================================================

    println!("[1] get_frame_count");

    let frame_count = tab.get_frame_count().await?;
    println!("    Frame count: {frame_count}");

    if frame_count >= 2 {
        println!("    ✓ Found {frame_count} child frames\n");
    } else {
        println!("    ✗ Expected at least 2 frames\n");
    }

    // ========================================================================
    // Get all frames
    // ========================================================================

    println!("[2] get_all_frames");

    let frames = tab.get_all_frames().await?;
    println!("    Total frames: {}", frames.len());

    for frame in &frames {
        let parent = frame
            .parent_frame_id
            .map(|p| p.as_u64().to_string())
            .unwrap_or_else(|| "none".to_string());
        let url_short = if frame.url.len() > 40 {
            format!("{}...", &frame.url[..40])
        } else {
            frame.url.clone()
        };
        println!(
            "    - Frame {}: parent={parent}, url={url_short}",
            frame.frame_id.as_u64()
        );
    }
    println!("    ✓ Listed all frames\n");

    // ========================================================================
    // Switch to frame by element
    // ========================================================================

    println!("[3] switch_to_frame (by element)");

    let iframe1 = tab.find_element(By::css("#frame1")).await?;
    println!("    Found iframe element: {}", iframe1.id());

    let frame1_tab = tab.switch_to_frame(&iframe1).await?;
    println!(
        "    Switched to frame ID: {}",
        frame1_tab.frame_id().as_u64()
    );

    match frame1_tab.find_element(By::css("#frame1-text")).await {
        Ok(el) => {
            let text = el.get_text().await?;
            println!("    Frame content: '{text}'");
            println!("    ✓ Successfully switched to frame 1\n");
        }
        Err(e) => {
            println!("    ✗ Could not find frame content: {e}\n");
        }
    }

    // ========================================================================
    // Switch to parent frame
    // ========================================================================

    println!("[4] switch_to_parent_frame");

    let parent_tab = frame1_tab.switch_to_parent_frame().await?;
    println!(
        "    Switched to frame ID: {}",
        parent_tab.frame_id().as_u64()
    );

    if parent_tab.is_main_frame() {
        println!("    ✓ Back in main frame\n");
    } else {
        println!("    ✗ Expected main frame\n");
    }

    // ========================================================================
    // Switch to frame by index
    // ========================================================================

    println!("[5] switch_to_frame_by_index(1)");

    let frame2_tab = tab.switch_to_frame_by_index(1).await?;
    println!(
        "    Switched to frame ID: {}",
        frame2_tab.frame_id().as_u64()
    );

    match frame2_tab.find_element(By::css("#frame2-text")).await {
        Ok(el) => {
            let text = el.get_text().await?;
            println!("    Frame content: '{text}'");
            println!("    ✓ Successfully switched to frame 2\n");
        }
        Err(e) => {
            println!("    ✗ Could not find frame content: {e}\n");
        }
    }

    // ========================================================================
    // Switch to main frame
    // ========================================================================

    println!("[6] switch_to_main_frame");

    let main_tab = frame2_tab.switch_to_main_frame();
    println!("    Switched to frame ID: {}", main_tab.frame_id().as_u64());

    if main_tab.is_main_frame() {
        match main_tab.find_element(By::css("#main-content")).await {
            Ok(el) => {
                let text = el.get_text().await?;
                println!("    Main content: '{text}'");
                println!("    ✓ Back in main frame\n");
            }
            Err(e) => {
                println!("    ✗ Could not find main content: {e}\n");
            }
        }
    } else {
        println!("    ✗ Expected main frame\n");
    }

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== All frame switching tests complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

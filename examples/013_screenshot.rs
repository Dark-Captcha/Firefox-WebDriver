//! Screenshot capture demonstration.
//!
//! Demonstrates:
//! - Capturing full page screenshot (PNG)
//! - Capturing page screenshot as JPEG with quality
//! - Capturing element screenshot
//! - Saving screenshots to files
//! - Using By selectors
//!
//! Usage:
//!   cargo run --example 013_screenshot
//!   cargo run --example 013_screenshot -- --no-wait
//!   cargo run --example 013_screenshot -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::path::Path;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{By, Driver, Result};

// ============================================================================
// Constants
// ============================================================================

const TEST_URL: &str = "https://example.com";
const SCREENSHOT_DIR: &str = "./screenshots";

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
    println!("=== 013: Screenshot ===\n");

    // ========================================================================
    // Setup
    // ========================================================================

    println!("[Setup] Creating screenshot directory...");
    std::fs::create_dir_all(SCREENSHOT_DIR).ok();
    println!("        ✓ Directory ready: {SCREENSHOT_DIR}\n");

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
    // Page Screenshot (PNG)
    // ========================================================================

    println!("[1] Capture page screenshot (PNG)...");
    let png_path = Path::new(SCREENSHOT_DIR).join("page.png");
    match tab.screenshot().png().save(&png_path).await {
        Ok(()) => {
            println!("    ✓ Saved to: {}", png_path.display());
            let metadata = std::fs::metadata(&png_path)?;
            println!("    Size: {} bytes\n", metadata.len());
        }
        Err(e) => {
            println!("    ⚠ Screenshot not available: {e}");
            println!("    (Extension may not implement captureScreenshot yet)\n");
        }
    }

    // ========================================================================
    // Page Screenshot (JPEG)
    // ========================================================================

    println!("[2] Capture page screenshot (JPEG, quality=85)...");
    let jpeg_path = Path::new(SCREENSHOT_DIR).join("page.jpg");
    match tab.screenshot().jpeg(85).save(&jpeg_path).await {
        Ok(()) => {
            println!("    ✓ Saved to: {}", jpeg_path.display());
            let metadata = std::fs::metadata(&jpeg_path)?;
            println!("    Size: {} bytes\n", metadata.len());
        }
        Err(_) => {
            println!("    ⚠ Skipped (captureScreenshot not implemented)\n");
        }
    }

    // ========================================================================
    // Page Screenshot (Base64)
    // ========================================================================

    println!("[3] Capture page screenshot as base64...");
    match tab.screenshot().png().capture().await {
        Ok(base64_data) => {
            println!(
                "    ✓ Captured {} characters of base64 data\n",
                base64_data.len()
            );
        }
        Err(_) => {
            println!("    ⚠ Skipped (captureScreenshot not implemented)\n");
        }
    }

    // ========================================================================
    // Element Screenshot
    // ========================================================================

    println!("[4] Capture element screenshot...");
    // Using By::tag() to find heading
    let heading = tab.find_element(By::tag("h1")).await?;
    let heading_text = heading.get_text().await?;
    println!("    Found element: <h1>{}</h1>", heading_text);

    let element_path = Path::new(SCREENSHOT_DIR).join("element.png");
    match heading.save_screenshot(&element_path).await {
        Ok(()) => {
            println!("    ✓ Saved to: {}", element_path.display());
            let metadata = std::fs::metadata(&element_path)?;
            println!("    Size: {} bytes\n", metadata.len());
        }
        Err(_) => {
            println!("    ⚠ Skipped (element.captureScreenshot not implemented)\n");
        }
    }

    // ========================================================================
    // Element Screenshot (JPEG)
    // ========================================================================

    println!("[5] Capture element screenshot (JPEG)...");
    let element_jpeg_path = Path::new(SCREENSHOT_DIR).join("element.jpg");
    match heading.save_screenshot(&element_jpeg_path).await {
        Ok(()) => {
            println!("    ✓ Saved to: {}", element_jpeg_path.display());
            let metadata = std::fs::metadata(&element_jpeg_path)?;
            println!("    Size: {} bytes\n", metadata.len());
        }
        Err(_) => {
            println!("    ⚠ Skipped (element.captureScreenshot not implemented)\n");
        }
    }

    // ========================================================================
    // Shorthand Methods
    // ========================================================================

    println!("[6] Using shorthand methods...");

    // tab.capture_screenshot() - returns base64
    match tab.capture_screenshot().await {
        Ok(base64) => println!("    tab.capture_screenshot(): {} chars", base64.len()),
        Err(_) => println!("    tab.capture_screenshot(): ⚠ not implemented"),
    }

    // tab.save_screenshot() - auto-detects format from extension
    let auto_png = Path::new(SCREENSHOT_DIR).join("auto.png");
    match tab.save_screenshot(&auto_png).await {
        Ok(()) => println!("    tab.save_screenshot(auto.png): ✓"),
        Err(_) => println!("    tab.save_screenshot(auto.png): ⚠ not implemented"),
    }

    let auto_jpg = Path::new(SCREENSHOT_DIR).join("auto.jpg");
    match tab.save_screenshot(&auto_jpg).await {
        Ok(()) => println!("    tab.save_screenshot(auto.jpg): ✓"),
        Err(_) => println!("    tab.save_screenshot(auto.jpg): ⚠ not implemented"),
    }
    println!();

    // ========================================================================
    // Done
    // ========================================================================

    println!("[Summary] Screenshots directory:");
    match std::fs::read_dir(SCREENSHOT_DIR) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Ok(metadata) = std::fs::metadata(&path)
                {
                    println!("    - {} ({} bytes)", path.display(), metadata.len());
                }
            }
        }
        Err(_) => println!("    (no screenshots saved)"),
    }

    common::print_logs(&window, 10).await?;

    println!("\n=== Screenshot example complete ===\n");
    println!("NOTE: If screenshots failed, the extension needs to implement");
    println!("      browsingContext.captureScreenshot and element.captureScreenshot\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

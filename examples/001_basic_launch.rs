//! Basic Firefox launch and profile management.
//!
//! Demonstrates:
//! - Creating a Driver with Firefox binary and extension
//! - Spawning a window with temporary profile
//! - Spawning a window with persistent profile
//! - Profile directory verification
//!
//! Usage:
//!   cargo run --example 001_basic_launch
//!   cargo run --example 001_basic_launch -- --no-wait
//!   cargo run --example 001_basic_launch -- --debug
//!   cargo run --example 001_basic_launch -- --clean

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::path::Path;

use common::{Args, EXTENSION_PATH, FIREFOX_BINARY};
use firefox_webdriver::{Driver, Result};

// ============================================================================
// Constants
// ============================================================================

const PROFILE_PATH: &str = "./test_profile";

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
    println!("=== 001: Basic Launch ===\n");

    // Clean profile if requested
    if args.clean && Path::new(PROFILE_PATH).exists() {
        println!("[Setup] Cleaning existing profile...");
        std::fs::remove_dir_all(PROFILE_PATH).ok();
        println!("        ✓ Profile removed\n");
    }

    // ========================================================================
    // Create Driver
    // ========================================================================

    println!("[1] Creating driver...");
    println!("    Binary: {FIREFOX_BINARY}");
    println!("    Extension: {EXTENSION_PATH}");

    let driver = Driver::builder()
        .binary(FIREFOX_BINARY)
        .extension(EXTENSION_PATH)
        .build()?;

    println!("    ✓ Driver ready\n");

    // ========================================================================
    // Spawn Window (temp profile)
    // ========================================================================

    println!("[2] Spawning window (temp profile)...");

    let window = driver.window().window_size(1280, 720).spawn().await?;

    println!("    ✓ Window spawned");
    println!("    Session: {}", window.session_id());
    println!("    Port:    {}", window.port());
    println!("    PID:     {}", window.pid());

    let _tab = window.tab();
    println!("    ✓ Tab ready\n");

    window.close().await?;
    println!("    ✓ Temp window closed\n");

    // ========================================================================
    // Spawn Window (persistent profile)
    // ========================================================================

    println!("[3] Spawning window (persistent profile)...");
    println!("    Profile: {PROFILE_PATH}");

    let window2 = driver
        .window()
        .profile(PROFILE_PATH)
        .window_size(1280, 720)
        .spawn()
        .await?;

    println!("    ✓ Window spawned");
    println!("    Session: {}", window2.session_id());
    println!("    Port:    {}", window2.port());
    println!("    PID:     {}\n", window2.pid());

    // ========================================================================
    // Verify Profile
    // ========================================================================

    println!("[4] Verifying profile...");
    verify_profile()?;
    println!();

    let _tab2 = window2.tab();
    println!("    ✓ Tab ready\n");

    println!("=== Launch successful ===\n");

    common::wait_for_exit(args.no_wait).await;

    // ========================================================================
    // Cleanup
    // ========================================================================

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    println!("\n[Verify] Profile persistence...");
    if Path::new(PROFILE_PATH).exists() {
        println!("         ✓ Profile persists at {PROFILE_PATH}");
        println!("         (Use --clean to remove on next run)");
    } else {
        println!("         ✗ Profile was removed (unexpected)");
    }

    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

fn verify_profile() -> Result<()> {
    let profile_path = Path::new(PROFILE_PATH);

    if profile_path.exists() {
        println!("    ✓ Profile directory exists");
    } else {
        println!("    ✗ Profile directory missing");
        return Err(firefox_webdriver::Error::profile(
            "Profile directory not created",
        ));
    }

    let user_js = profile_path.join("user.js");
    if user_js.exists() {
        println!("    ✓ user.js exists");

        if let Ok(content) = std::fs::read_to_string(&user_js) {
            let pref_count = content
                .lines()
                .filter(|l| l.starts_with("user_pref"))
                .count();
            println!("    ✓ {pref_count} preferences written");
        }
    } else {
        println!("    - user.js not yet created");
    }

    let extensions_dir = profile_path.join("extensions");
    if extensions_dir.exists() {
        let ext_count = std::fs::read_dir(&extensions_dir)
            .map(|d| d.count())
            .unwrap_or(0);
        println!("    ✓ extensions/ exists ({ext_count} extension(s))");
    } else {
        println!("    - extensions/ not yet created");
    }

    Ok(())
}

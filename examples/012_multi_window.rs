//! Multi-window stress test.
//!
//! Demonstrates:
//! - Spawning 50 browser windows concurrently
//! - Continuous element manipulation for 1 minute
//! - Random text injection across all windows
//!
//! Usage:
//!   cargo run --example 012_multi_window
//!   cargo run --example 012_multi_window -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::{Duration, Instant};

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{By, Driver, Result, Window};

// ============================================================================
// Constants
// ============================================================================

const WINDOW_COUNT: usize = 50;
const TEST_DURATION: Duration = Duration::from_secs(15);
const URL: &str = "https://example.com";

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

async fn run(_args: Args) -> Result<()> {
    println!("=== 012: Multi-Window Stress Test ===\n");
    println!("Windows: {WINDOW_COUNT}");
    println!("Duration: {}s", TEST_DURATION.as_secs());
    println!("URL: {URL}\n");

    // ========================================================================
    // Create Driver
    // ========================================================================

    println!("[1] Creating driver...");
    let driver = Driver::builder()
        .binary(firefox_binary())
        .extension(extension_path())
        .build()
        .await?;
    println!("    ✓ Driver ready\n");

    // ========================================================================
    // Spawn Windows
    // ========================================================================

    println!("[2] Spawning {WINDOW_COUNT} windows...");
    let start = Instant::now();

    let spawn_futures: Vec<_> = (0..WINDOW_COUNT)
        .map(|_| driver.window().headless().spawn())
        .collect();

    let windows: Vec<Window> = futures_util::future::try_join_all(spawn_futures).await?;

    println!("    ✓ Spawned in {:.2?}\n", start.elapsed());

    // ========================================================================
    // Navigate All to example.com
    // ========================================================================

    println!("[3] Navigating all windows to {URL}...");
    let start = Instant::now();

    let nav_futures: Vec<_> = windows
        .iter()
        .map(|w| async {
            let tab = w.tab();
            tab.goto(URL).await
        })
        .collect();

    futures_util::future::try_join_all(nav_futures).await?;
    println!("    ✓ All navigated in {:.2?}\n", start.elapsed());

    // ========================================================================
    // Stress Test: Random Text Changes for 1 Minute
    // ========================================================================

    println!(
        "[4] Starting stress test for {}s...",
        TEST_DURATION.as_secs()
    );
    println!("    Continuously changing h1 text to random strings\n");

    let test_start = Instant::now();
    let mut iteration = 0u64;
    let mut errors = 0u64;

    while test_start.elapsed() < TEST_DURATION {
        iteration += 1;

        // Run all windows in parallel
        let tasks: Vec<_> = windows
            .iter()
            .enumerate()
            .map(|(i, w)| async move {
                let tab = w.tab();
                let random_text = generate_random_text(i as u64 + iteration);

                // Change h1 text via script execution
                let script =
                    format!(r#"document.querySelector('h1').textContent = '{random_text}'"#);
                tab.execute_script(&script).await?;

                // Also read it back to verify
                let h1 = tab.find_element(By::css("h1")).await?;
                let _text = h1.get_text().await?;

                Ok::<_, firefox_webdriver::Error>(())
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;

        // Count errors
        for r in results {
            if r.is_err() {
                errors += 1;
            }
        }

        // Progress every 10 iterations
        if iteration.is_multiple_of(10) {
            let elapsed = test_start.elapsed().as_secs();
            let ops = iteration * WINDOW_COUNT as u64;
            println!(
                "    [{:>2}s] Iteration {iteration}, Total ops: {ops}, Errors: {errors}",
                elapsed
            );
        }
    }

    let total_ops = iteration * WINDOW_COUNT as u64;
    let elapsed = test_start.elapsed();
    let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();

    println!();
    println!("=== Results ===");
    println!("    Duration:   {:.2?}", elapsed);
    println!("    Iterations: {iteration}");
    println!("    Total ops:  {total_ops}");
    println!("    Errors:     {errors}");
    println!("    Ops/sec:    {ops_per_sec:.1}");
    println!();

    // ========================================================================
    // Cleanup
    // ========================================================================

    println!("[5] Closing all windows...");
    let start = Instant::now();

    let close_futures: Vec<_> = windows.iter().map(|w| w.close()).collect();
    futures_util::future::try_join_all(close_futures).await?;

    println!("    ✓ Closed in {:.2?}", start.elapsed());

    driver.close().await?;
    println!("    ✓ Driver closed");

    Ok(())
}

/// Simple deterministic "random" text generator (no external deps).
fn generate_random_text(seed: u64) -> String {
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let len = 8 + (state % 12) as usize;

    (0..len)
        .map(|i| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(i as u64);
            (b'a' + (state % 26) as u8) as char
        })
        .collect()
}

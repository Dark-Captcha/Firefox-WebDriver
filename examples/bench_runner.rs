//! Benchmark runner for multi-window performance testing.
//!
//! Runs benchmarks and outputs CSV for plotting.
//!
//! Usage:
//!   cargo run --release --example bench_runner
//!   cargo run --release --example bench_runner -- --quick   # Quick mode (shorter tests)
//!
//! Output:
//!   benches/results/benchmark_results.csv

mod common;

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::time::{Duration, Instant};

use common::{extension_path, firefox_binary};
use firefox_webdriver::{By, Driver, Result, Window};

// ============================================================================
// Configuration
// ============================================================================

const WINDOW_COUNTS: &[usize] = &[50, 100];
const TEST_DURATIONS_SECS: &[u64] = &[15, 30, 60, 120];
const QUICK_WINDOW_COUNTS: &[usize] = &[10, 25, 50];
const QUICK_DURATIONS_SECS: &[u64] = &[5, 10, 15];
const URL: &str = "https://example.com";

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quick_mode = args.iter().any(|a| a == "--quick");

    if let Err(e) = run(quick_mode).await {
        eprintln!("\n[ERROR] {e}");
        std::process::exit(1);
    }
}

async fn run(quick_mode: bool) -> Result<()> {
    println!("=== Firefox WebDriver Benchmark Runner ===\n");

    let (window_counts, durations) = if quick_mode {
        println!("Mode: QUICK (shorter tests)\n");
        (QUICK_WINDOW_COUNTS, QUICK_DURATIONS_SECS)
    } else {
        println!("Mode: FULL (use --quick for shorter tests)\n");
        (WINDOW_COUNTS, TEST_DURATIONS_SECS)
    };

    // Create results directory and CSV file
    fs::create_dir_all("benches/results").ok();
    let csv_path = "benches/results/benchmark_results.csv";

    // Write CSV header
    {
        let mut file = File::create(csv_path).expect("Failed to create CSV file");
        writeln!(
            file,
            "windows,duration_secs,spawn_time_ms,total_ops,ops_per_sec,errors"
        )
        .unwrap();
    }

    for &window_count in window_counts {
        for &duration_secs in durations {
            println!(
                "Running: {} windows, {}s duration...",
                window_count, duration_secs
            );

            let result = match run_benchmark(window_count, duration_secs).await {
                Ok(result) => {
                    let error_rate = if result.total_ops > 0 {
                        (result.errors as f64 / result.total_ops as f64) * 100.0
                    } else {
                        0.0
                    };
                    println!(
                        "  ✓ Spawn: {}ms, Ops: {}, Ops/s: {:.1}, Errors: {} ({:.1}%)",
                        result.spawn_time_ms,
                        result.total_ops,
                        result.ops_per_sec,
                        result.errors,
                        error_rate
                    );
                    result
                }
                Err(e) => {
                    println!("  ✗ Failed: {e}");
                    BenchResult {
                        windows: window_count,
                        duration_secs,
                        spawn_time_ms: 0,
                        total_ops: 0,
                        ops_per_sec: 0.0,
                        errors: 1,
                    }
                }
            };

            // Append result to CSV immediately
            let mut file = OpenOptions::new()
                .append(true)
                .open(csv_path)
                .expect("Failed to open CSV file");
            writeln!(
                file,
                "{},{},{},{},{:.2},{}",
                result.windows,
                result.duration_secs,
                result.spawn_time_ms,
                result.total_ops,
                result.ops_per_sec,
                result.errors
            )
            .unwrap();

            println!();
        }
    }

    println!("Results saved to: {csv_path}");
    println!("\nTo generate plots:");
    println!("  python3 benches/plot_results.py");

    Ok(())
}

async fn run_benchmark(window_count: usize, duration_secs: u64) -> Result<BenchResult> {
    // Create driver
    let driver = Driver::builder()
        .binary(firefox_binary())
        .extension(extension_path())
        .build()
        .await?;

    // Spawn windows
    let spawn_start = Instant::now();
    let spawn_futures: Vec<_> = (0..window_count)
        .map(|_| driver.window().headless().spawn())
        .collect();
    let windows: Vec<Window> = futures_util::future::try_join_all(spawn_futures).await?;
    let spawn_time = spawn_start.elapsed();

    // Navigate all
    let nav_futures: Vec<_> = windows
        .iter()
        .map(|w| async {
            let tab = w.tab();
            tab.goto(URL).await
        })
        .collect();
    futures_util::future::try_join_all(nav_futures).await?;

    // Small delay to let connections stabilize
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Run sustained operations
    let test_duration = Duration::from_secs(duration_secs);
    let test_start = Instant::now();
    let mut iteration = 0u64;
    let mut errors = 0u64;
    let mut success_ops = 0u64;
    let mut first_error: Option<String> = None;

    while test_start.elapsed() < test_duration {
        iteration += 1;

        let tasks: Vec<_> = windows
            .iter()
            .enumerate()
            .map(|(i, w)| async move {
                let tab = w.tab();
                let random_text = generate_random_text(i as u64 + iteration);
                let h1 = tab.find_element(By::css("h1")).await?;
                h1.set_property("textContent", serde_json::Value::String(random_text))
                    .await?;
                let _text = h1.get_text().await?;
                Ok::<_, firefox_webdriver::Error>(())
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;
        for r in results {
            match r {
                Ok(_) => success_ops += 1,
                Err(e) => {
                    errors += 1;
                    if first_error.is_none() {
                        first_error = Some(format!("{e}"));
                    }
                }
            }
        }

        // Small yield to prevent overwhelming connections
        tokio::task::yield_now().await;
    }

    let total_ops = success_ops;
    let elapsed = test_start.elapsed();
    let ops_per_sec = if elapsed.as_secs_f64() > 0.0 {
        total_ops as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    // Print first error if any
    if let Some(err) = first_error {
        eprintln!("    First error: {}", err);
    }

    // Cleanup
    let close_futures: Vec<_> = windows.iter().map(|w| w.close()).collect();
    futures_util::future::try_join_all(close_futures).await?;
    driver.close().await?;

    Ok(BenchResult {
        windows: window_count,
        duration_secs,
        spawn_time_ms: spawn_time.as_millis() as u64,
        total_ops,
        ops_per_sec,
        errors,
    })
}

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

#[derive(Debug)]
struct BenchResult {
    windows: usize,
    duration_secs: u64,
    spawn_time_ms: u64,
    total_ops: u64,
    ops_per_sec: f64,
    errors: u64,
}

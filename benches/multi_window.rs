//! Multi-window benchmark suite.
//!
//! Benchmarks window spawning and operations at different scales:
//! - Window counts: 50, 100
//! - Durations: 15s, 30s, 60s, 120s
//!
//! Run with: cargo bench --bench multi_window
//! Results saved to: target/criterion/

use std::path::PathBuf;
use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tokio::runtime::Runtime;

// ============================================================================
// Configuration - Uses same paths as examples
// ============================================================================

fn firefox_binary() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join("Documents/Firefox-WebDriver-Patches/bin/firefox")
}

fn extension_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join("Documents/Firefox-WebDriver-Extension/firefox-webdriver-extension-0.1.0.xpi")
}

// ============================================================================
// Benchmark Parameters
// ============================================================================

const WINDOW_COUNTS: &[usize] = &[50, 100];
const TEST_DURATIONS_SECS: &[u64] = &[15, 30, 60, 120];

// ============================================================================
// Benchmark: Window Spawn Time
// ============================================================================

fn bench_window_spawn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("window_spawn");
    group.sample_size(10); // Reduce samples for expensive benchmarks
    group.measurement_time(Duration::from_secs(30));

    for &count in WINDOW_COUNTS {
        group.bench_with_input(
            BenchmarkId::new("spawn", count),
            &count,
            |b, &window_count| {
                b.to_async(&rt)
                    .iter(|| async { spawn_windows(window_count).await });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark: Sustained Operations
// ============================================================================

fn bench_sustained_ops(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sustained_ops");
    group.sample_size(10);

    for &count in WINDOW_COUNTS {
        for &duration_secs in TEST_DURATIONS_SECS {
            let id = format!("{}w_{}s", count, duration_secs);
            group.bench_with_input(
                BenchmarkId::new("ops", &id),
                &(count, duration_secs),
                |b, &(window_count, dur_secs)| {
                    b.to_async(&rt)
                        .iter(|| async { run_sustained_test(window_count, dur_secs).await });
                },
            );
        }
    }

    group.finish();
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn spawn_windows(count: usize) -> BenchResult {
    use firefox_webdriver::Driver;

    let start = Instant::now();

    let driver = match Driver::builder()
        .binary(firefox_binary())
        .extension(extension_path())
        .build()
        .await
    {
        Ok(d) => d,
        Err(e) => {
            return BenchResult {
                spawn_time_ms: 0,
                total_ops: 0,
                ops_per_sec: 0.0,
                errors: 1,
                error_msg: Some(format!("Driver creation failed: {e}")),
            };
        }
    };

    let spawn_futures: Vec<_> = (0..count)
        .map(|_| driver.window().headless().spawn())
        .collect();

    let windows = match futures_util::future::try_join_all(spawn_futures).await {
        Ok(w) => w,
        Err(e) => {
            let _ = driver.close().await;
            return BenchResult {
                spawn_time_ms: start.elapsed().as_millis() as u64,
                total_ops: 0,
                ops_per_sec: 0.0,
                errors: 1,
                error_msg: Some(format!("Window spawn failed: {e}")),
            };
        }
    };

    let spawn_time = start.elapsed();

    // Cleanup
    let close_futures: Vec<_> = windows.iter().map(|w| w.close()).collect();
    let _ = futures_util::future::try_join_all(close_futures).await;
    let _ = driver.close().await;

    BenchResult {
        spawn_time_ms: spawn_time.as_millis() as u64,
        total_ops: count as u64,
        ops_per_sec: count as f64 / spawn_time.as_secs_f64(),
        errors: 0,
        error_msg: None,
    }
}

async fn run_sustained_test(window_count: usize, duration_secs: u64) -> BenchResult {
    use firefox_webdriver::Driver;

    let driver = match Driver::builder()
        .binary(firefox_binary())
        .extension(extension_path())
        .build()
        .await
    {
        Ok(d) => d,
        Err(e) => {
            return BenchResult {
                spawn_time_ms: 0,
                total_ops: 0,
                ops_per_sec: 0.0,
                errors: 1,
                error_msg: Some(format!("Driver creation failed: {e}")),
            };
        }
    };

    // Spawn windows
    let spawn_start = Instant::now();
    let spawn_futures: Vec<_> = (0..window_count)
        .map(|_| driver.window().headless().spawn())
        .collect();

    let windows = match futures_util::future::try_join_all(spawn_futures).await {
        Ok(w) => w,
        Err(e) => {
            let _ = driver.close().await;
            return BenchResult {
                spawn_time_ms: spawn_start.elapsed().as_millis() as u64,
                total_ops: 0,
                ops_per_sec: 0.0,
                errors: 1,
                error_msg: Some(format!("Window spawn failed: {e}")),
            };
        }
    };
    let spawn_time = spawn_start.elapsed();

    // Navigate all to example.com
    let nav_futures: Vec<_> = windows
        .iter()
        .map(|w| async {
            let tab = w.tab();
            tab.goto("https://example.com").await
        })
        .collect();

    if let Err(e) = futures_util::future::try_join_all(nav_futures).await {
        let close_futures: Vec<_> = windows.iter().map(|w| w.close()).collect();
        let _ = futures_util::future::try_join_all(close_futures).await;
        let _ = driver.close().await;
        return BenchResult {
            spawn_time_ms: spawn_time.as_millis() as u64,
            total_ops: 0,
            ops_per_sec: 0.0,
            errors: 1,
            error_msg: Some(format!("Navigation failed: {e}")),
        };
    }

    // Run sustained operations
    let test_duration = Duration::from_secs(duration_secs);
    let test_start = Instant::now();
    let mut iteration = 0u64;
    let mut errors = 0u64;

    while test_start.elapsed() < test_duration {
        iteration += 1;

        let tasks: Vec<_> = windows
            .iter()
            .enumerate()
            .map(|(i, w)| async move {
                let tab = w.tab();
                let random_text = generate_random_text(i as u64 + iteration);
                let h1 = tab.find_element("h1").await?;
                h1.set_property("textContent", serde_json::Value::String(random_text))
                    .await?;
                let _text = h1.get_text().await?;
                Ok::<_, firefox_webdriver::Error>(())
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;
        for r in results {
            if r.is_err() {
                errors += 1;
            }
        }
    }

    let total_ops = iteration * window_count as u64;
    let elapsed = test_start.elapsed();
    let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();

    // Cleanup
    let close_futures: Vec<_> = windows.iter().map(|w| w.close()).collect();
    let _ = futures_util::future::try_join_all(close_futures).await;
    let _ = driver.close().await;

    BenchResult {
        spawn_time_ms: spawn_time.as_millis() as u64,
        total_ops,
        ops_per_sec,
        errors,
        error_msg: None,
    }
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

// ============================================================================
// Result Type
// ============================================================================

#[derive(Debug, Clone)]
struct BenchResult {
    spawn_time_ms: u64,
    total_ops: u64,
    ops_per_sec: f64,
    errors: u64,
    error_msg: Option<String>,
}

// ============================================================================
// Criterion Setup
// ============================================================================

criterion_group!(benches, bench_window_spawn, bench_sustained_ops);
criterion_main!(benches);

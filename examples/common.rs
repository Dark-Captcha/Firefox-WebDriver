//! Shared utilities for examples.
//!
//! Provides common functionality used across all examples:
//! - Command-line argument parsing
//! - Logging initialization
//! - Graceful exit handling

#![allow(dead_code)]

// ============================================================================
// Imports
// ============================================================================

use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

// ============================================================================
// Path Helpers
// ============================================================================

/// Get the Firefox binary path from $HOME.
pub fn firefox_binary() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join("Documents/Firefox-WebDriver-Patches/bin/firefox")
}

/// Get the extension path from $HOME.
pub fn extension_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join("Documents/Firefox-WebDriver-Extension/firefox-webdriver-extension-0.1.2.xpi")
}

// ============================================================================
// Types
// ============================================================================

/// Command-line arguments for examples.
#[derive(Debug, Clone)]
pub struct Args {
    pub debug: bool,
    pub no_wait: bool,
    pub clean: bool,
}

impl Args {
    /// Parse command-line arguments.
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self {
            debug: args.iter().any(|a| a == "--debug"),
            no_wait: args.iter().any(|a| a == "--no-wait"),
            clean: args.iter().any(|a| a == "--clean"),
        }
    }
}

// ============================================================================
// Functions
// ============================================================================

/// Initialize tracing/logging.
pub fn init_logging(debug: bool) {
    let filter = if debug {
        "firefox_webdriver=debug"
    } else {
        "firefox_webdriver=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .init();
}

/// Wait for Ctrl+C or skip if `--no-wait` flag is set.
pub async fn wait_for_exit(no_wait: bool) {
    if no_wait {
        println!("[--no-wait] Skipping wait");
        return;
    }

    println!("Press Ctrl+C to exit...");
    tokio::signal::ctrl_c().await.ok();
}

/// Print extension logs from window.
pub async fn print_logs(
    window: &firefox_webdriver::Window,
    count: usize,
) -> firefox_webdriver::Result<()> {
    println!("[Logs] Extension logs (last {count}):");
    let logs = window.steal_logs().await?;
    for log in logs.iter().rev().take(count).rev() {
        if let Some(msg) = log.get("message").and_then(|v| v.as_str()) {
            let level = log.get("level").and_then(|v| v.as_str()).unwrap_or("?");
            let module = log.get("module").and_then(|v| v.as_str()).unwrap_or("?");
            println!("        [{level}] [{module}] {msg}");
        }
    }
    Ok(())
}

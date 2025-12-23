//! Proxy configuration demonstration.
//!
//! Demonstrates:
//! - Window-level HTTP proxy
//! - Window-level SOCKS5 proxy
//! - Tab-level proxy override
//! - Proxy with authentication
//! - Proxy clear operations
//! - Multiple tabs with different proxies
//!
//! Note: Requires actual proxy servers to fully verify functionality.
//!
//! Usage:
//!   cargo run --example 009_proxy
//!   cargo run --example 009_proxy -- --no-wait
//!   cargo run --example 009_proxy -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, EXTENSION_PATH, FIREFOX_BINARY};
use firefox_webdriver::{Driver, ProxyConfig, Result};

// ============================================================================
// Constants
// ============================================================================

const TEST_URL: &str = "https://example.com";
const TEST_URL_2: &str = "https://httpbin.org";
const TEST_HTTP_PROXY_HOST: &str = "127.0.0.1";
const TEST_HTTP_PROXY_PORT: u16 = 8080;
const TEST_SOCKS_PROXY_HOST: &str = "127.0.0.1";
const TEST_SOCKS_PROXY_PORT: u16 = 1080;

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
    println!("=== 009: Proxy Configuration ===\n");

    // ========================================================================
    // Setup
    // ========================================================================

    println!("[Setup] Creating driver and window...");

    let driver = Driver::builder()
        .binary(FIREFOX_BINARY)
        .extension(EXTENSION_PATH)
        .build()?;

    let window = driver.window().window_size(1280, 720).spawn().await?;
    let tab = window.tab();
    println!(
        "        ✓ Window spawned (session={})\n",
        window.session_id()
    );

    println!("[Setup] Navigating to {TEST_URL}...");
    tab.goto(TEST_URL).await?;
    sleep(Duration::from_millis(1000)).await;
    println!("        ✓ Page loaded\n");

    test_window_proxy_http(&window, &tab).await?;
    test_window_proxy_socks5(&window, &tab).await?;
    test_tab_proxy_override(&window, &tab).await?;
    test_proxy_with_auth(&window, &tab).await?;
    test_proxy_clear(&window, &tab).await?;
    test_multiple_tabs_different_proxies(&window).await?;

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 20).await?;

    println!("\n=== All proxy tests complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

// ============================================================================
// Test: Window Proxy HTTP
// ============================================================================

async fn test_window_proxy_http(
    window: &firefox_webdriver::Window,
    tab: &firefox_webdriver::Tab,
) -> Result<()> {
    println!("[1] Window proxy - HTTP");

    let proxy = ProxyConfig::http(TEST_HTTP_PROXY_HOST, TEST_HTTP_PROXY_PORT);
    window.set_proxy(proxy).await?;
    println!("    ✓ Set HTTP proxy: {TEST_HTTP_PROXY_HOST}:{TEST_HTTP_PROXY_PORT}");

    let script = format!(
        r#"
        try {{
            const resp = await fetch('{TEST_URL_2}/ip', {{ signal: AbortSignal.timeout(3000) }});
            const data = await resp.json();
            return {{ success: true, origin: data.origin }};
        }} catch (e) {{
            return {{ success: false, error: e.message }};
        }}
    "#
    );
    let result = tab.execute_async_script(&script).await?;
    println!("    Request result: {result:?}");

    window.clear_proxy().await?;
    println!("    ✓ Window HTTP proxy test complete\n");
    Ok(())
}

// ============================================================================
// Test: Window Proxy SOCKS5
// ============================================================================

async fn test_window_proxy_socks5(
    window: &firefox_webdriver::Window,
    tab: &firefox_webdriver::Tab,
) -> Result<()> {
    println!("[2] Window proxy - SOCKS5");

    let proxy =
        ProxyConfig::socks5(TEST_SOCKS_PROXY_HOST, TEST_SOCKS_PROXY_PORT).with_proxy_dns(true);
    window.set_proxy(proxy).await?;
    println!(
        "    ✓ Set SOCKS5 proxy: {TEST_SOCKS_PROXY_HOST}:{TEST_SOCKS_PROXY_PORT} (proxyDNS=true)"
    );

    let script = format!(
        r#"
        try {{
            const resp = await fetch('{TEST_URL_2}/ip', {{ signal: AbortSignal.timeout(3000) }});
            const data = await resp.json();
            return {{ success: true, origin: data.origin }};
        }} catch (e) {{
            return {{ success: false, error: e.message }};
        }}
    "#
    );
    let result = tab.execute_async_script(&script).await?;
    println!("    Request result: {result:?}");

    window.clear_proxy().await?;
    println!("    ✓ Window SOCKS5 proxy test complete\n");
    Ok(())
}

// ============================================================================
// Test: Tab Proxy Override
// ============================================================================

async fn test_tab_proxy_override(
    window: &firefox_webdriver::Window,
    tab: &firefox_webdriver::Tab,
) -> Result<()> {
    println!("[3] Tab proxy - override window proxy");

    let window_proxy = ProxyConfig::http("window-proxy.example.com", 8080);
    window.set_proxy(window_proxy).await?;
    println!("    ✓ Set window proxy: window-proxy.example.com:8080");

    let tab_proxy = ProxyConfig::socks5("tab-proxy.example.com", 1080);
    tab.set_proxy(tab_proxy).await?;
    println!("    ✓ Set tab proxy: tab-proxy.example.com:1080 (overrides window)");

    tab.clear_proxy().await?;
    println!("    ✓ Cleared tab proxy (now using window proxy)");

    window.clear_proxy().await?;
    println!("    ✓ Tab proxy override test complete\n");
    Ok(())
}

// ============================================================================
// Test: Proxy with Authentication
// ============================================================================

async fn test_proxy_with_auth(
    window: &firefox_webdriver::Window,
    tab: &firefox_webdriver::Tab,
) -> Result<()> {
    println!("[4] Proxy with authentication");

    let http_proxy =
        ProxyConfig::http("auth-proxy.example.com", 8080).with_credentials("username", "password");
    window.set_proxy(http_proxy).await?;
    println!("    ✓ Set HTTP proxy with auth: auth-proxy.example.com:8080");

    window.clear_proxy().await?;

    let socks_proxy = ProxyConfig::socks5("socks-auth.example.com", 1080)
        .with_credentials("socks_user", "socks_pass")
        .with_proxy_dns(true);
    tab.set_proxy(socks_proxy).await?;
    println!("    ✓ Set SOCKS5 proxy with auth: socks-auth.example.com:1080");

    tab.clear_proxy().await?;
    println!("    ✓ Proxy authentication test complete\n");
    Ok(())
}

// ============================================================================
// Test: Proxy Clear
// ============================================================================

async fn test_proxy_clear(
    window: &firefox_webdriver::Window,
    tab: &firefox_webdriver::Tab,
) -> Result<()> {
    println!("[5] Proxy clear operations");

    window
        .set_proxy(ProxyConfig::http("window.example.com", 8080))
        .await?;
    tab.set_proxy(ProxyConfig::socks5("tab.example.com", 1080))
        .await?;
    println!("    ✓ Set window and tab proxies");

    tab.clear_proxy().await?;
    println!("    ✓ Cleared tab proxy (window proxy still active)");

    window.clear_proxy().await?;
    println!("    ✓ Cleared window proxy (direct connection)");

    let script = format!(
        r#"
        try {{
            const resp = await fetch('{TEST_URL}/', {{ signal: AbortSignal.timeout(5000) }});
            return {{ success: resp.ok }};
        }} catch (e) {{
            return {{ success: false, error: e.message }};
        }}
    "#
    );
    let result = tab.execute_async_script(&script).await?;
    println!("    Direct connection result: {result:?}");
    println!("    ✓ Proxy clear test complete\n");
    Ok(())
}

// ============================================================================
// Test: Multiple Tabs Different Proxies
// ============================================================================

async fn test_multiple_tabs_different_proxies(window: &firefox_webdriver::Window) -> Result<()> {
    println!("[6] Multiple tabs with different proxies");

    let tab1 = window.tab();

    let tab2 = window.new_tab().await?;
    println!("    ✓ Created second tab");

    tab1.set_proxy(ProxyConfig::http("proxy1.example.com", 8081))
        .await?;
    println!("    ✓ Tab 1 proxy: proxy1.example.com:8081");

    tab2.set_proxy(ProxyConfig::socks5("proxy2.example.com", 1082))
        .await?;
    println!("    ✓ Tab 2 proxy: proxy2.example.com:1082");

    tab1.clear_proxy().await?;
    tab2.clear_proxy().await?;

    tab2.close().await?;
    println!("    ✓ Multiple tabs proxy test complete\n");
    Ok(())
}

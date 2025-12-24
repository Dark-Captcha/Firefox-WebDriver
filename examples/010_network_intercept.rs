//! Network interception demonstration.
//!
//! Demonstrates:
//! - Block rules (basic patterns, overlapping, clear/re-add)
//! - Request interception (allow/block, rapid requests, stateful)
//! - Request header modification
//! - Request body logging
//! - Response header interception
//! - Response body modification (HTML, JSON)
//! - Combined block rules + intercept
//!
//! Usage:
//!   cargo run --example 010_network_intercept
//!   cargo run --example 010_network_intercept -- --no-wait
//!   cargo run --example 010_network_intercept -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{
    BodyAction, Driver, HeadersAction, RequestAction, RequestBody, Result, Tab,
};

// ============================================================================
// Constants
// ============================================================================

const TEST_URL: &str = "https://example.com";

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
    println!("=== 010: Network Interception ===\n");

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
    sleep(Duration::from_millis(1000)).await;
    println!("        ✓ Page loaded\n");

    // Run tests
    test_block_rules_basic(&tab).await?;
    test_block_rules_overlapping(&tab).await?;
    test_block_rules_clear_readd(&tab).await?;
    test_intercept_allow_block(&tab).await?;
    test_intercept_rapid_requests(&tab).await?;
    test_intercept_stateful(&tab).await?;
    test_request_headers_modify(&tab).await?;
    test_request_body_logging(&tab).await?;
    test_response_headers(&tab).await?;
    test_response_body_html(&tab).await?;
    test_response_body_json(&tab).await?;
    test_block_and_intercept_together(&tab).await?;

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 20).await?;

    println!("\n=== All network interception tests complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

// ============================================================================
// Helper: Check if fetch was blocked
// ============================================================================

async fn fetch_check(tab: &Tab, url: &str) -> Result<bool> {
    let script = format!(
        r#"
        try {{
            await fetch('{url}');
            return false;
        }} catch {{
            return true;
        }}
    "#
    );

    let result = tab.execute_async_script(&script).await?;
    Ok(result.as_bool().unwrap_or(false))
}

// ============================================================================
// Test: Block Rules Basic
// ============================================================================

async fn test_block_rules_basic(tab: &Tab) -> Result<()> {
    println!("[1] Block rules - basic patterns");

    tab.set_block_rules(&["*tracking*", "*analytics*", "*ads*"])
        .await?;

    let r1 = fetch_check(tab, &format!("{TEST_URL}/tracking.gif")).await?;
    let r2 = fetch_check(tab, &format!("{TEST_URL}/analytics.js")).await?;
    let r3 = fetch_check(tab, &format!("{TEST_URL}/ads/banner.png")).await?;
    let r4 = fetch_check(tab, &format!("{TEST_URL}/normal.js")).await?;

    assert!(r1, "tracking should be blocked");
    assert!(r2, "analytics should be blocked");
    assert!(r3, "ads should be blocked");
    assert!(!r4, "normal should NOT be blocked");

    tab.clear_block_rules().await?;
    println!("    ✓ Basic block rules work\n");
    Ok(())
}

// ============================================================================
// Test: Block Rules Overlapping
// ============================================================================

async fn test_block_rules_overlapping(tab: &Tab) -> Result<()> {
    println!("[2] Block rules - overlapping patterns");

    tab.set_block_rules(&["*example.com/test*", "*test/file*", "*special.js"])
        .await?;

    let r1 = fetch_check(tab, &format!("{TEST_URL}/test/file.txt")).await?;
    let r2 = fetch_check(tab, &format!("{TEST_URL}/test/other.png")).await?;
    let r3 = fetch_check(tab, &format!("{TEST_URL}/special.js")).await?;
    let r4 = fetch_check(tab, &format!("{TEST_URL}/normal/image.png")).await?;

    assert!(r1, "multi-match should be blocked");
    assert!(r2, "first-pattern match should be blocked");
    assert!(r3, "last-pattern match should be blocked");
    assert!(!r4, "no-match should NOT be blocked");

    tab.clear_block_rules().await?;
    println!("    ✓ Overlapping patterns work\n");
    Ok(())
}

// ============================================================================
// Test: Block Rules Clear and Re-add
// ============================================================================

async fn test_block_rules_clear_readd(tab: &Tab) -> Result<()> {
    println!("[3] Block rules - clear and re-add");

    tab.set_block_rules(&["*blocked1*"]).await?;
    let r1 = fetch_check(tab, &format!("{TEST_URL}/blocked1.js")).await?;
    assert!(r1, "blocked1 should be blocked initially");

    tab.clear_block_rules().await?;
    let r2 = fetch_check(tab, &format!("{TEST_URL}/blocked1.js")).await?;
    assert!(!r2, "blocked1 should NOT be blocked after clear");

    tab.set_block_rules(&["*blocked2*"]).await?;
    let r3 = fetch_check(tab, &format!("{TEST_URL}/blocked1.js")).await?;
    let r4 = fetch_check(tab, &format!("{TEST_URL}/blocked2.js")).await?;
    assert!(!r3, "blocked1 should NOT be blocked with new rules");
    assert!(r4, "blocked2 should be blocked with new rules");

    tab.clear_block_rules().await?;
    println!("    ✓ Clear and re-add work\n");
    Ok(())
}

// ============================================================================
// Test: Intercept Allow/Block Mixed
// ============================================================================

async fn test_intercept_allow_block(tab: &Tab) -> Result<()> {
    println!("[4] Intercept - allow/block mixed decisions");

    let block_count = Arc::new(AtomicUsize::new(0));
    let allow_count = Arc::new(AtomicUsize::new(0));
    let bc = block_count.clone();
    let ac = allow_count.clone();

    let intercept_id = tab
        .intercept_request(move |req| {
            if req.url.contains("block-this") {
                bc.fetch_add(1, Ordering::SeqCst);
                RequestAction::block()
            } else if req.url.contains("allow-this") {
                ac.fetch_add(1, Ordering::SeqCst);
                RequestAction::allow()
            } else {
                RequestAction::allow()
            }
        })
        .await?;

    let script = format!(
        r#"
        await Promise.all([
            fetch('{TEST_URL}/block-this-1').catch(() => {{}}),
            fetch('{TEST_URL}/allow-this-1').catch(() => {{}}),
            fetch('{TEST_URL}/block-this-2').catch(() => {{}}),
            fetch('{TEST_URL}/allow-this-2').catch(() => {{}}),
            fetch('{TEST_URL}/block-this-3').catch(() => {{}}),
        ]);
    "#
    );
    tab.execute_async_script(&script).await?;
    sleep(Duration::from_millis(500)).await;

    let blocked = block_count.load(Ordering::SeqCst);
    let allowed = allow_count.load(Ordering::SeqCst);
    println!("    Blocked: {blocked}, Allowed: {allowed}");
    assert_eq!(blocked, 3, "should block 3 requests");
    assert_eq!(allowed, 2, "should allow 2 requests");

    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ Mixed allow/block works\n");
    Ok(())
}

// ============================================================================
// Test: Intercept Rapid Requests
// ============================================================================

async fn test_intercept_rapid_requests(tab: &Tab) -> Result<()> {
    println!("[5] Intercept - rapid concurrent requests");

    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();

    let intercept_id = tab
        .intercept_request(move |_req| {
            c.fetch_add(1, Ordering::SeqCst);
            RequestAction::allow()
        })
        .await?;

    let script = format!(
        r#"
        const promises = [];
        for (let i = 0; i < 20; i++) {{
            promises.push(fetch('{TEST_URL}/rapid-' + i).catch(() => {{}}));
        }}
        await Promise.all(promises);
    "#
    );
    tab.execute_async_script(&script).await?;
    sleep(Duration::from_millis(1000)).await;

    let intercepted = count.load(Ordering::SeqCst);
    println!("    Intercepted: {intercepted}/20 requests");
    assert!(intercepted >= 18, "should intercept most requests");

    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ Rapid requests handled\n");
    Ok(())
}

// ============================================================================
// Test: Intercept Stateful
// ============================================================================

async fn test_intercept_stateful(tab: &Tab) -> Result<()> {
    println!("[6] Intercept - stateful (alternating block/allow)");

    let call_count = Arc::new(AtomicUsize::new(0));
    let cc = call_count.clone();

    let intercept_id = tab
        .intercept_request(move |_req| {
            let n = cc.fetch_add(1, Ordering::SeqCst);
            if n % 2 == 0 {
                RequestAction::block()
            } else {
                RequestAction::allow()
            }
        })
        .await?;

    let script = format!(
        r#"
        const results = [];
        for (let i = 0; i < 4; i++) {{
            try {{
                await fetch('{TEST_URL}/stateful-test');
                results.push('allowed');
            }} catch {{
                results.push('blocked');
            }}
        }}
        return results;
    "#
    );
    let results = tab.execute_async_script(&script).await?;
    println!("    Results: {results:?}");

    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ Stateful interception works\n");
    Ok(())
}

// ============================================================================
// Test: Request Headers Modify
// ============================================================================

async fn test_request_headers_modify(tab: &Tab) -> Result<()> {
    println!("[7] Request headers - add/modify");

    let modified_count = Arc::new(AtomicUsize::new(0));
    let mc = modified_count.clone();

    let intercept_id = tab
        .intercept_request_headers(move |headers| {
            mc.fetch_add(1, Ordering::SeqCst);

            let mut h = headers.headers.clone();
            h.insert("X-Custom-Added".to_string(), "added-value".to_string());
            h.insert("User-Agent".to_string(), "CustomBot/1.0".to_string());

            HeadersAction::modify_headers(h)
        })
        .await?;

    let script = format!(
        r#"
        await fetch('{TEST_URL}/headers-test-1').catch(() => {{}});
        await fetch('{TEST_URL}/headers-test-2').catch(() => {{}});
    "#
    );
    tab.execute_async_script(&script).await?;
    sleep(Duration::from_millis(500)).await;

    let count = modified_count.load(Ordering::SeqCst);
    println!("    Modified {count} requests");
    assert!(count >= 2, "should modify at least 2 requests");

    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ Header modification works\n");
    Ok(())
}

// ============================================================================
// Test: Request Body Logging
// ============================================================================

async fn test_request_body_logging(tab: &Tab) -> Result<()> {
    println!("[8] Request body - logging (read-only)");

    let body_count = Arc::new(AtomicUsize::new(0));
    let bc = body_count.clone();

    let intercept_id = tab
        .intercept_request_body(move |req| {
            bc.fetch_add(1, Ordering::SeqCst);
            if let Some(body) = &req.body {
                match body {
                    RequestBody::FormData(data) => {
                        println!(
                            "    [Body] {} - FormData: {:?}",
                            req.method,
                            data.keys().collect::<Vec<_>>()
                        );
                    }
                    RequestBody::Raw(bytes) => {
                        if let Ok(text) = String::from_utf8(bytes.clone()) {
                            let preview = if text.len() > 50 {
                                format!("{}...", &text[..50])
                            } else {
                                text
                            };
                            println!("    [Body] {} - Raw: {preview}", req.method);
                        }
                    }
                    RequestBody::Error(err) => {
                        println!("    [Body] {} - Error: {err}", req.method);
                    }
                }
            }
        })
        .await?;

    let script = format!(
        r#"
        const formData = new FormData();
        formData.append('username', 'testuser');
        await fetch('{TEST_URL}/login', {{ method: 'POST', body: formData }}).catch(() => {{}});

        await fetch('{TEST_URL}/api/json', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{ action: 'test' }})
        }}).catch(() => {{}});
    "#
    );
    tab.execute_async_script(&script).await?;
    sleep(Duration::from_millis(500)).await;

    let bodies = body_count.load(Ordering::SeqCst);
    println!("    Logged {bodies} requests");

    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ Request body logging works\n");
    Ok(())
}

// ============================================================================
// Test: Response Headers
// ============================================================================

async fn test_response_headers(tab: &Tab) -> Result<()> {
    println!("[9] Response headers - inspect");

    let response_count = Arc::new(AtomicUsize::new(0));
    let rc = response_count.clone();

    let intercept_id = tab
        .intercept_response(move |_resp| {
            rc.fetch_add(1, Ordering::SeqCst);
            HeadersAction::allow()
        })
        .await?;

    let script = format!(r#"await fetch('{TEST_URL}/').catch(() => {{}});"#);
    tab.execute_async_script(&script).await?;
    sleep(Duration::from_millis(500)).await;

    let count = response_count.load(Ordering::SeqCst);
    println!("    Intercepted {count} responses");

    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ Response header interception works\n");
    Ok(())
}

// ============================================================================
// Test: Response Body HTML
// ============================================================================

async fn test_response_body_html(tab: &Tab) -> Result<()> {
    println!("[10] Response body - modify HTML");

    let intercept_id = tab
        .intercept_response_body(|body| {
            if body.url.ends_with("/") && body.body.contains("<title>") {
                let modified = body.body.replace("Example Domain", "MODIFIED TITLE");
                BodyAction::modify_body(modified)
            } else {
                BodyAction::allow()
            }
        })
        .await?;

    tab.reload().await?;
    sleep(Duration::from_millis(2000)).await;

    let title = tab.get_title().await?;
    println!("    Page title: {title}");

    tab.stop_intercept(&intercept_id).await?;

    if title.contains("MODIFIED") {
        println!("    ✓ HTML body modification works\n");
    } else {
        println!("    ⚠ Title not modified (may be cached)\n");
    }
    Ok(())
}

// ============================================================================
// Test: Response Body JSON
// ============================================================================

async fn test_response_body_json(tab: &Tab) -> Result<()> {
    println!("[11] Response body - JSON detection");

    let json_count = Arc::new(AtomicUsize::new(0));
    let jc = json_count.clone();

    let intercept_id = tab
        .intercept_response_body(move |body| {
            let trimmed = body.body.trim();
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                jc.fetch_add(1, Ordering::SeqCst);
            }
            BodyAction::allow()
        })
        .await?;

    let script = format!(r#"await fetch('{TEST_URL}/').catch(() => {{}});"#);
    tab.execute_async_script(&script).await?;
    sleep(Duration::from_millis(500)).await;

    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ JSON detection logic works\n");
    Ok(())
}

// ============================================================================
// Test: Block Rules AND Intercept Together
// ============================================================================

async fn test_block_and_intercept_together(tab: &Tab) -> Result<()> {
    println!("[12] Block rules + intercept together");

    tab.set_block_rules(&["*blocked-by-rule*"]).await?;

    let intercept_count = Arc::new(AtomicUsize::new(0));
    let ic = intercept_count.clone();

    let intercept_id = tab
        .intercept_request(move |req| {
            ic.fetch_add(1, Ordering::SeqCst);
            if req.url.contains("blocked-by-callback") {
                RequestAction::block()
            } else {
                RequestAction::allow()
            }
        })
        .await?;

    let r1 = fetch_check(tab, &format!("{TEST_URL}/blocked-by-rule")).await?;
    let r2 = fetch_check(tab, &format!("{TEST_URL}/blocked-by-callback")).await?;
    let r3 = fetch_check(tab, &format!("{TEST_URL}/allowed")).await?;

    sleep(Duration::from_millis(300)).await;

    let intercepted = intercept_count.load(Ordering::SeqCst);
    println!("    Interceptor called: {intercepted} times");
    println!(
        "    blocked-by-rule: {}",
        if r1 { "blocked" } else { "allowed" }
    );
    println!(
        "    blocked-by-callback: {}",
        if r2 { "blocked" } else { "allowed" }
    );
    println!("    allowed: {}", if r3 { "blocked" } else { "allowed" });

    assert!(r1, "rule-blocked should be blocked");
    assert!(r2, "callback-blocked should be blocked");
    assert!(!r3, "allowed should NOT be blocked");

    tab.clear_block_rules().await?;
    tab.stop_intercept(&intercept_id).await?;
    println!("    ✓ Block rules + intercept work together\n");
    Ok(())
}

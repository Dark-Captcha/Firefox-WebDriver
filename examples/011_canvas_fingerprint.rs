//! Canvas fingerprint randomization test.
//!
//! Tests that Firefox canvas randomization produces different hashes
//! for each canvas extraction, defeating fingerprinting.
//!
//! Vectors tested:
//! - CMY Color Mixing (blending modes)
//! - Emoji & Text Metrics (font rendering)
//! - Text with Background (bounding box)
//! - Distorted ASCII (serif rendering)
//! - Winding Rule & Geometry (anti-aliasing)
//! - Complex Composite (multiple blending)
//! - Emoji Buffer (glyph buffering)
//! - Shadows & Primitives (GPU shadow blur)
//! - Unicode & Matrix (transformation matrix)
//!
//! Usage:
//!   cargo run --example 011_canvas_fingerprint
//!   cargo run --example 011_canvas_fingerprint -- --no-wait
//!   cargo run --example 011_canvas_fingerprint -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::collections::HashSet;
use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{Driver, Result, Tab};

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
    println!("=== 011: Canvas Fingerprint Randomization ===\n");

    // ========================================================================
    // Setup - Create 2 windows with 2 tabs each
    // ========================================================================

    println!("[Setup] Creating driver, 2 windows, 2 tabs each...");

    let driver = Driver::builder()
        .binary(firefox_binary())
        .extension(extension_path())
        .build()
        .await?;

    let window1 = driver.window().window_size(1280, 900).spawn().await?;
    let window2 = driver.window().window_size(1280, 900).spawn().await?;

    // Create second tab in each window
    let w1_tab2 = window1.new_tab().await?;
    let w2_tab2 = window2.new_tab().await?;

    println!(
        "        ✓ Window 1 spawned (session={}) with 2 tabs",
        window1.session_id()
    );
    println!(
        "        ✓ Window 2 spawned (session={}) with 2 tabs\n",
        window2.session_id()
    );

    // Collect all tabs: (window_idx, tab_idx, tab)
    let tabs: Vec<(usize, usize, Tab)> = vec![
        (1, 1, window1.tab()),
        (1, 2, w1_tab2),
        (2, 1, window2.tab()),
        (2, 2, w2_tab2),
    ];

    // Setup all tabs
    for (win_idx, tab_idx, tab) in &tabs {
        println!(
            "[Setup] W{}-T{} - Navigating to {TEST_URL}...",
            win_idx, tab_idx
        );
        tab.goto(TEST_URL).await?;
        sleep(Duration::from_millis(500)).await;

        println!(
            "[Setup] W{}-T{} - Loading canvas fingerprint test page...",
            win_idx, tab_idx
        );
        tab.load_html(FINGERPRINT_HTML).await?;
        sleep(Duration::from_millis(500)).await;

        println!(
            "[Setup] W{}-T{} - Injecting canvas fingerprint vectors...",
            win_idx, tab_idx
        );
        tab.execute_script(CANVAS_SCRIPT).await?;
        sleep(Duration::from_millis(500)).await;
        println!("        ✓ W{}-T{} ready\n", win_idx, tab_idx);
    }

    // ========================================================================
    // Test each canvas vector on all 4 tabs
    // ========================================================================

    let vectors = [
        ("CMY Color Mixing", "cmy_mixing"),
        ("Emoji & Text Metrics", "emoji_text"),
        ("Text with Background", "text_bg"),
        ("Distorted ASCII", "distorted_ascii"),
        ("Winding Rule & Geometry", "winding_rule"),
        ("Complex Composite", "complex_composite"),
        ("Emoji Buffer", "emoji_buffer"),
        ("Shadows & Primitives", "shadows_primitives"),
        ("Unicode & Matrix", "unicode_matrix"),
    ];

    let mut all_passed = true;
    let mut total_unique = 0;
    let mut total_tests = 0;

    for (i, (name, id)) in vectors.iter().enumerate() {
        println!("[Test {}] {} - hash uniqueness", i + 1, name);

        let mut all_hashes: Vec<String> = Vec::new();

        // Test each tab (2 extractions per tab = 8 total)
        for (win_idx, tab_idx, tab) in &tabs {
            let (_, hashes) = test_canvas_uniqueness(tab, id, 2).await?;

            // Display hashes on HTML
            display_hash_on_html(tab, id, &hashes).await?;

            println!("    W{}-T{} hashes:", win_idx, tab_idx);
            for (j, hash) in hashes.iter().enumerate() {
                let short_hash = &hash[..16.min(hash.len())];
                println!("      #{}: {}...", j + 1, short_hash);
            }

            all_hashes.extend(hashes);
        }

        // Check total uniqueness across all tabs
        let unique_set: HashSet<_> = all_hashes.iter().collect();
        let unique_count = unique_set.len();
        total_unique += unique_count;
        total_tests += 8;

        if unique_count == 8 {
            println!("    ✓ All 8 extractions unique (2 per tab × 4 tabs)\n");
        } else {
            println!(
                "    ✗ Only {}/8 unique hashes (randomization issue)\n",
                unique_count
            );
            all_passed = false;
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================

    println!("=== Summary ===");
    println!("    Configuration: 2 windows × 2 tabs = 4 tabs total");
    println!(
        "    Extractions: 2 per tab × 9 vectors = {} total",
        total_tests
    );
    println!("    Unique hashes: {}/{}", total_unique, total_tests);

    if all_passed {
        println!("    ✓ Canvas randomization is working correctly!");
        println!("    All tabs across both windows produce unique fingerprints.\n");
    } else {
        println!("    ⚠ Some canvas extractions produced duplicate hashes.");
        println!("    Check if canvas randomization patch is applied.\n");
    }

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window1, 10).await?;

    println!("\n=== Canvas fingerprint test complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

// ============================================================================
// Canvas Uniqueness Test
// ============================================================================

async fn test_canvas_uniqueness(
    tab: &Tab,
    canvas_id: &str,
    iterations: usize,
) -> Result<(usize, Vec<String>)> {
    let mut hashes = Vec::new();

    for _ in 0..iterations {
        let script = format!(
            r#"
            return (async () => {{
                const canvas = document.getElementById('{canvas_id}');
                if (!canvas) return 'canvas_not_found';
                const dataUrl = canvas.toDataURL();
                const msgBuffer = new TextEncoder().encode(dataUrl);
                const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);
                const hashArray = Array.from(new Uint8Array(hashBuffer));
                return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
            }})();
        "#
        );

        let result = tab.execute_async_script(&script).await?;
        if let Some(hash) = result.as_str() {
            hashes.push(hash.to_string());
        }

        sleep(Duration::from_millis(50)).await;
    }

    let unique: HashSet<_> = hashes.iter().collect();
    Ok((unique.len(), hashes))
}

// ============================================================================
// Display Hash on HTML
// ============================================================================

async fn display_hash_on_html(tab: &Tab, canvas_id: &str, hashes: &[String]) -> Result<()> {
    let hashes_json: Vec<String> = hashes
        .iter()
        .map(|h| format!("\"{}\"", &h[..16.min(h.len())]))
        .collect();
    let hashes_array = format!("[{}]", hashes_json.join(","));

    let script = format!(
        r#"
        (function() {{
            const canvas = document.getElementById('{canvas_id}');
            if (!canvas) return;
            const card = canvas.closest('.card');
            if (!card) return;

            // Remove existing hash display
            const existing = card.querySelector('.hash-display');
            if (existing) existing.remove();

            // Create hash display element
            const hashDiv = document.createElement('div');
            hashDiv.className = 'hash-display';
            hashDiv.style.cssText = 'font-size:9px;color:#666;margin-top:5px;word-break:break-all;max-width:240px;';

            const hashes = {hashes_array};
            hashDiv.innerHTML = hashes.map((h, i) => `<div>#${{i+1}}: ${{h}}...</div>`).join('');

            card.appendChild(hashDiv);
        }})();
        "#
    );

    tab.execute_script(&script).await?;
    Ok(())
}

// ============================================================================
// Constants: HTML Template
// ============================================================================

const FINGERPRINT_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Canvas Fingerprint Randomization Test</title>
    <style>
        body {
            font-family: monospace;
            background: #1a1a1a;
            color: #0f0;
            padding: 20px;
        }
        h2 { text-align: center; color: #fff; }
        .grid {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 15px;
            max-width: 1000px;
            margin: 0 auto;
        }
        .card {
            background: #fff;
            padding: 10px;
            border-radius: 8px;
            display: flex;
            flex-direction: column;
            align-items: center;
        }
        canvas {
            border: 1px solid #ccc;
            background: white;
        }
        .label {
            font-size: 11px;
            color: #333;
            margin-bottom: 5px;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <h2>Canvas Fingerprint Vectors (Randomization Test)</h2>
    <div class="grid" id="grid"></div>
</body>
</html>"#;

// ============================================================================
// Constants: Canvas Rendering Script
// ============================================================================

const CANVAS_SCRIPT: &str = r##"
(function() {
    const grid = document.getElementById('grid');

    function createCanvas(id, label, renderFn) {
        const card = document.createElement('div');
        card.className = 'card';

        const labelEl = document.createElement('div');
        labelEl.className = 'label';
        labelEl.textContent = label;

        const canvas = document.createElement('canvas');
        canvas.id = id;
        canvas.width = 240;
        canvas.height = 120;

        const ctx = canvas.getContext('2d');
        try {
            renderFn(ctx, canvas.width, canvas.height);
        } catch(e) {
            console.error('Canvas render error:', e);
        }

        card.appendChild(labelEl);
        card.appendChild(canvas);
        grid.appendChild(card);
    }

    // 1. CMY Color Mixing
    createCanvas('cmy_mixing', 'CMY Color Mixing', (ctx, w, h) => {
        ctx.globalCompositeOperation = "multiply";
        ctx.fillStyle = "#ff00ff";
        ctx.beginPath();
        ctx.arc(w/2 - 30, h/2, 40, 0, Math.PI * 2, true);
        ctx.fill();
        ctx.fillStyle = "#00ffff";
        ctx.beginPath();
        ctx.arc(w/2 + 30, h/2, 40, 0, Math.PI * 2, true);
        ctx.fill();
        ctx.fillStyle = "#ffff00";
        ctx.beginPath();
        ctx.arc(w/2, h/2 + 40, 40, 0, Math.PI * 2, true);
        ctx.fill();
    });

    // 2. Emoji & Text Metrics
    createCanvas('emoji_text', 'Emoji & Text Metrics', (ctx, w, h) => {
        ctx.textBaseline = "middle";
        ctx.textAlign = "center";
        ctx.font = "40px";
        ctx.fillStyle = "black";
        ctx.fillText("iO0A🤣💩", w/2, h/2);
    });

    // 3. Text with Background
    createCanvas('text_bg', 'Text with Background', (ctx, w, h) => {
        ctx.font = "18px 'Times New Roman'";
        ctx.fillStyle = "black";
        ctx.textBaseline = "alphabetic";
        ctx.fillStyle = "#069";
        ctx.fillText("Cwm fjordbank gly 😃", 20, 40);
        ctx.fillStyle = "rgba(255, 102, 0, 1)";
        ctx.fillRect(145, 28, 40, 18);
        ctx.font = "24px Arial";
        ctx.fillStyle = "rgba(0, 0, 0, 0.1)";
        ctx.fillText("Cwm fjordbank gly 😃", 20, 80);
    });

    // 4. Distorted ASCII
    createCanvas('distorted_ascii', 'Distorted ASCII', (ctx, w, h) => {
        ctx.font = "30px serif";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillStyle = "black";
        ctx.fillText("Hel478b2cf6-ba3c-44eb-bbcb-0fd8fc1a29cc?6%){mZ+#@", w/2, h/2);
    });

    // 5. Winding Rule & Geometry
    createCanvas('winding_rule', 'Winding Rule & Geometry', (ctx, w, h) => {
        ctx.fillStyle = "#6666ff";
        ctx.fillRect(20, 20, 80, 80);
        ctx.fillRect(110, 20, 40, 80);
        ctx.beginPath();
        ctx.strokeStyle = "black";
        ctx.lineWidth = 4;
        ctx.arc(80, 80, 30, 0, Math.PI*2);
        ctx.moveTo(140, 60);
        ctx.arc(120, 60, 20, 0, Math.PI*2);
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(110, 40, 4, 0, Math.PI*2);
        ctx.stroke();
    });

    // 6. Complex Composite
    createCanvas('complex_composite', 'Complex Composite', (ctx, w, h) => {
        ctx.font = "16px Arial";
        ctx.fillStyle = "black";
        ctx.fillText("Cwm fjordbank gly phs vext quiz", 5, 20);
        ctx.globalCompositeOperation = "multiply";
        ctx.fillStyle = "rgb(255,0,255)";
        ctx.beginPath();
        ctx.arc(50, 50, 40, 0, Math.PI*2, true);
        ctx.fill();
        ctx.fillStyle = "rgb(0,255,255)";
        ctx.beginPath();
        ctx.arc(100, 50, 40, 0, Math.PI*2, true);
        ctx.fill();
        ctx.fillStyle = "rgb(255,255,0)";
        ctx.beginPath();
        ctx.arc(75, 90, 40, 0, Math.PI*2, true);
        ctx.fill();
        ctx.fillStyle = "rgb(255,0,255)";
        ctx.globalCompositeOperation = "overlay";
        ctx.fillRect(20, 20, 150, 50);
        ctx.globalCompositeOperation = "source-over";
        ctx.fillStyle = "blue";
        ctx.font = "12px monospace";
        ctx.fillText("0.8178819...", 60, 60);
        ctx.fillStyle = "red";
        ctx.beginPath();
        ctx.arc(90, 90, 10, 0, Math.PI*2);
        ctx.fill();
    });

    // 7. Emoji Buffer
    createCanvas('emoji_buffer', 'Emoji Buffer', (ctx, w, h) => {
        ctx.font = "18px Arial";
        const emojis = "😀😁😂🤣😃😄😅😆😉😊😋😎😍😘🥰";
        ctx.fillText(emojis, 0, 30);
        ctx.fillText(emojis, 0, 60);
        ctx.fillText(emojis, 0, 90);
    });

    // 8. Shadows & Primitives
    createCanvas('shadows_primitives', 'Shadows & Primitives', (ctx, w, h) => {
        ctx.shadowBlur = 10;
        ctx.shadowColor = "red";
        ctx.fillStyle = "#369";
        ctx.beginPath();
        ctx.arc(50, 60, 25, 0, Math.PI*2);
        ctx.fill();
        ctx.shadowBlur = 0;
        ctx.fillStyle = "#9c6";
        ctx.beginPath();
        ctx.moveTo(100, 100);
        ctx.lineTo(125, 40);
        ctx.lineTo(150, 100);
        ctx.fill();
        ctx.strokeStyle = "black";
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(125, 40);
        ctx.lineTo(200, 90);
        ctx.stroke();
        ctx.fillStyle = "green";
        ctx.font = "10px Arial";
        ctx.fillText("SomeCanvasFingerPrint.65@34587", 30, 20);
        ctx.fillStyle = "orange";
        ctx.fillRect(120, 15, 50, 10);
    });

    // 9. Unicode & Matrix
    createCanvas('unicode_matrix', 'Unicode & Matrix', (ctx, w, h) => {
        ctx.font = "24px Arial";
        ctx.fillStyle = "#444";
        ctx.save();
        ctx.translate(10, 60);
        ctx.scale(1.0, 0.9);
        ctx.fillText("aBc#$efG~ \u2665", 0, 0);
        ctx.restore();
        ctx.fillStyle = "#888";
        ctx.font = "30px serif";
        ctx.fillText("\u2764 \u3020", 140, 60);
    });

    console.log('All canvas vectors rendered');
})();
"##;

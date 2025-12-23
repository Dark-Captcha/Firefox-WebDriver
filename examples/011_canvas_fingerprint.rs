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

use common::{Args, EXTENSION_PATH, FIREFOX_BINARY};
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
    // Setup
    // ========================================================================

    println!("[Setup] Creating driver and window...");

    let driver = Driver::builder()
        .binary(FIREFOX_BINARY)
        .extension(EXTENSION_PATH)
        .build()?;

    let window = driver.window().window_size(1280, 900).spawn().await?;
    let tab = window.tab();
    println!(
        "        ✓ Window spawned (session={})\n",
        window.session_id()
    );

    println!("[Setup] Navigating to {TEST_URL}...");
    tab.goto(TEST_URL).await?;
    sleep(Duration::from_millis(500)).await;
    println!("        ✓ Page loaded\n");

    println!("[Setup] Loading canvas fingerprint test page...");
    tab.load_html(FINGERPRINT_HTML).await?;
    sleep(Duration::from_millis(500)).await;
    println!("        ✓ Test page loaded\n");

    println!("[Setup] Injecting canvas fingerprint vectors...");
    tab.execute_script(CANVAS_SCRIPT).await?;
    sleep(Duration::from_millis(1000)).await;
    println!("        ✓ Canvas vectors rendered\n");

    // ========================================================================
    // Test each canvas vector
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

        let (unique_count, hashes) = test_canvas_uniqueness(&tab, id, 5).await?;
        total_unique += unique_count;
        total_tests += 5;

        println!("    Hashes collected:");
        for (j, hash) in hashes.iter().enumerate() {
            let short_hash = &hash[..16.min(hash.len())];
            println!("      #{}: {}...", j + 1, short_hash);
        }

        if unique_count == 5 {
            println!("    ✓ All 5 extractions produced unique hashes\n");
        } else {
            println!(
                "    ✗ Only {}/5 unique hashes (randomization may not be working)\n",
                unique_count
            );
            all_passed = false;
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================

    println!("=== Summary ===");
    println!("    Total unique hashes: {}/{}", total_unique, total_tests);

    if all_passed {
        println!("    ✓ Canvas randomization is working correctly!");
        println!("    Each toDataURL() call produces a different hash.\n");
    } else {
        println!("    ⚠ Some canvas extractions produced duplicate hashes.");
        println!("    Check if canvas randomization patch is applied.\n");
    }

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

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

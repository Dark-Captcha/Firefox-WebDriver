//! Microsoft/Outlook registration demonstration.
//!
//! Demonstrates:
//! - Complex form automation
//! - Custom dropdown handling
//! - Wait for elements
//! - Multi-step form flow
//!
//! NOTE: This example is for educational purposes only.
//! Registration may fail due to CAPTCHA or other anti-bot measures.
//!
//! Usage:
//!   cargo run --example 017_outlook_register
//!   cargo run --example 017_outlook_register -- --no-wait
//!   cargo run --example 017_outlook_register -- --debug

mod common;

// ============================================================================
// Imports
// ============================================================================

use std::time::Duration;

use tokio::time::sleep;

use common::{Args, extension_path, firefox_binary};
use firefox_webdriver::{By, Driver, Result, Tab};

// ============================================================================
// Constants
// ============================================================================

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

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
    println!("=== 017: Outlook Registration ===\n");

    // ========================================================================
    // Generate random data
    // ========================================================================

    let (first, last) = random_name();
    let password = random_password();
    let (year, month, day) = random_birthdate();
    let email = format!("{}.{}{}", first, last, year).to_lowercase();

    println!("[Info] Generated credentials:");
    println!("       Name: {first} {last}");
    println!("       Email: {email}@outlook.com");
    println!("       Birth: {year}-{month:02}-{day:02}");
    println!("       Password: {password}\n");

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

    // ========================================================================
    // Navigate to registration
    // ========================================================================

    println!("[1] Navigating to Outlook registration...");
    tab.goto("https://go.microsoft.com/fwlink/p/?linkid=2125440&clcid=0x409")
        .await?;
    println!("    ✓ Navigation started\n");

    // ========================================================================
    // Email step
    // ========================================================================

    println!("[2] Entering email...");
    let input = match tab.wait_for_element(By::name("New email")).await {
        Ok(el) => el,
        Err(e) => {
            println!("    [Error] {e}");
            println!("    [Debug] Stealing logs...");
            let logs = window.steal_logs().await?;
            for log in logs.iter().rev().take(50).rev() {
                if let Some(msg) = log.get("message").and_then(|v| v.as_str()) {
                    let level = log.get("level").and_then(|v| v.as_str()).unwrap_or("?");
                    let module = log.get("module").and_then(|v| v.as_str()).unwrap_or("?");
                    println!("        [{level}] [{module}] {msg}");
                }
            }
            return Err(e);
        }
    };
    input.type_text(&email).await?;
    println!("    Typed: {email}");
    click_submit(&tab).await?;
    println!("    ✓ Submitted\n");

    // ========================================================================
    // Password step
    // ========================================================================

    println!("[3] Entering password...");
    let input = tab
        .wait_for_element(By::css("input[type='password']"))
        .await?;
    input.type_text(&password).await?;
    println!("    Typed password");
    click_submit(&tab).await?;
    println!("    ✓ Submitted\n");

    // ========================================================================
    // Birthdate step
    // ========================================================================

    println!("[4] Entering birthdate...");

    // Month dropdown
    select_dropdown(
        &tab,
        By::css("button[name='BirthMonth']"),
        MONTHS[(month - 1) as usize],
    )
    .await?;
    println!("    Month: {}", MONTHS[(month - 1) as usize]);

    // Day dropdown
    select_dropdown(&tab, By::css("button[name='BirthDay']"), &day.to_string()).await?;
    println!("    Day: {day}");

    // Year input
    let year_input = tab
        .wait_for_element(By::css("input[name='BirthYear']"))
        .await?;
    year_input.type_text(&year.to_string()).await?;
    println!("    Year: {year}");

    click_submit(&tab).await?;
    println!("    ✓ Submitted\n");

    // ========================================================================
    // Name step
    // ========================================================================

    println!("[5] Entering name...");
    let first_input = tab
        .wait_for_element(By::css("input[name='firstNameInput']"))
        .await?;
    first_input.type_text(first).await?;
    println!("    First: {first}");

    let last_input = tab
        .wait_for_element(By::css("input[name='lastNameInput']"))
        .await?;
    last_input.type_text(last).await?;
    println!("    Last: {last}");

    click_submit(&tab).await?;
    println!("    ✓ Submitted\n");

    // ========================================================================
    // Check result
    // ========================================================================

    println!("[6] Checking result...");
    sleep(Duration::from_secs(2)).await;
    let url = tab.get_url().await?;
    println!("    URL: {url}");

    if url.contains("challenge") || url.contains("captcha") {
        println!("    ⚠ CAPTCHA detected - manual intervention required");
    } else if url.contains("proofs") {
        println!("    ⚠ Phone/email verification required");
    } else if url.contains("signup") {
        println!("    ⚠ Still on signup - check for errors");
    } else {
        println!("    ✓ Registration may have succeeded");
    }
    println!();

    // ========================================================================
    // Done
    // ========================================================================

    common::print_logs(&window, 10).await?;

    println!("\n=== Registration flow complete ===\n");

    common::wait_for_exit(args.no_wait).await;

    println!("\n[Cleanup] Closing driver...");
    driver.close().await?;
    println!("          ✓ Done");

    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

async fn click_submit(tab: &Tab) -> Result<()> {
    sleep(Duration::from_millis(500)).await;
    let btn = tab
        .wait_for_element(By::css("button[type='submit']"))
        .await?;
    btn.click().await?;
    sleep(Duration::from_secs(1)).await;
    Ok(())
}

async fn select_dropdown(tab: &Tab, selector: By, value: &str) -> Result<()> {
    let btn = tab.wait_for_element(selector).await?;
    btn.click().await?;
    sleep(Duration::from_millis(500)).await;

    for option in tab.find_elements(By::css("div[role='option']")).await? {
        if let Ok(text) = option.get_text().await
            && (text.trim() == value || text.contains(value))
        {
            option.click().await?;
            break;
        }
    }
    sleep(Duration::from_millis(500)).await;
    Ok(())
}

fn seed() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn random_password() -> String {
    let s = seed();
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%";
    (0..14)
        .map(|i| {
            let idx = ((s >> (i * 3)) % chars.len() as u128) as usize;
            chars.chars().nth(idx).unwrap()
        })
        .collect()
}

fn random_name() -> (&'static str, &'static str) {
    let s = seed() as usize;
    let first = ["Alex", "Jordan", "Taylor", "Casey", "Morgan", "Riley"];
    let last = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia"];
    (first[s % first.len()], last[(s / 7) % last.len()])
}

fn random_birthdate() -> (u32, u32, u32) {
    let s = seed() as u32;
    (
        2024 - 18 - (s % 47),
        1 + (s / 100) % 12,
        1 + (s / 1000) % 28,
    )
}

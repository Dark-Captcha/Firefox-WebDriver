//! Discord registration demonstration.
//!
//! Demonstrates:
//! - Complex form automation with custom components
//! - Role-based dropdown handling
//! - Multi-field form filling
//! - Error detection
//!
//! NOTE: This example is for educational purposes only.
//! Registration may fail due to CAPTCHA or other anti-bot measures.
//!
//! Usage:
//!   cargo run --example 018_discord_register
//!   cargo run --example 018_discord_register -- --no-wait
//!   cargo run --example 018_discord_register -- --debug

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
    println!("=== 018: Discord Registration ===\n");

    // ========================================================================
    // Generate random data
    // ========================================================================

    let email = random_email();
    let username = random_string(10);
    let display = format!("User{}", random_string(4));
    let password = random_password();
    let (year, month, day) = random_birthdate();

    println!("[Info] Generated credentials:");
    println!("       Email: {email}");
    println!("       Username: {username}");
    println!("       Display: {display}");
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

    println!("[1] Navigating to Discord registration...");
    tab.goto("https://discord.com/register").await?;
    sleep(Duration::from_secs(3)).await;
    println!("    ✓ Page loaded\n");

    // ========================================================================
    // Fill form fields
    // ========================================================================

    println!("[2] Filling form fields...");

    // Email
    fill_input(&tab, By::css("input[name='email']"), &email).await?;
    println!("    Email: {email}");

    // Display name (optional field)
    if fill_input(&tab, By::css("input[name='global_name']"), &display)
        .await
        .is_ok()
    {
        println!("    Display: {display}");
    }

    // Username
    fill_input(&tab, By::css("input[name='username']"), &username).await?;
    println!("    Username: {username}");

    // Password
    fill_input(&tab, By::css("input[name='password']"), &password).await?;
    println!("    Password: ****");
    println!();

    // ========================================================================
    // Date of birth dropdowns
    // ========================================================================

    println!("[3] Selecting date of birth...");

    // Month
    select_role_dropdown(
        &tab,
        By::css("div[role='button'][aria-label='Month']"),
        MONTHS[(month - 1) as usize],
    )
    .await?;
    println!("    Month: {}", MONTHS[(month - 1) as usize]);

    // Day
    select_role_dropdown(
        &tab,
        By::css("div[role='button'][aria-label='Day']"),
        &day.to_string(),
    )
    .await?;
    println!("    Day: {day}");

    // Year
    select_role_dropdown(
        &tab,
        By::css("div[role='button'][aria-label='Year']"),
        &year.to_string(),
    )
    .await?;
    println!("    Year: {year}");
    println!();

    // ========================================================================
    // Optional checkbox
    // ========================================================================

    println!("[4] Handling optional checkbox...");
    if let Ok(cb) = tab.find_element(By::css("input[type='checkbox']")).await {
        if seed().is_multiple_of(2) {
            cb.click().await?;
            println!("    Checkbox: checked");
        } else {
            println!("    Checkbox: skipped");
        }
    } else {
        println!("    Checkbox: not found");
    }
    println!();

    // ========================================================================
    // Submit
    // ========================================================================

    println!("[5] Submitting form...");
    sleep(Duration::from_millis(500)).await;
    let submit = tab
        .wait_for_element(By::css("button[type='submit']"))
        .await?;
    submit.click().await?;
    sleep(Duration::from_secs(3)).await;
    println!("    ✓ Submitted\n");

    // ========================================================================
    // Handle hCaptcha (if present)
    // ========================================================================

    println!("[6] Checking for hCaptcha...");

    // Wait a bit for captcha to appear
    sleep(Duration::from_secs(2)).await;

    // Try to find hcaptcha iframe
    if let Ok(iframe) = tab.find_element(By::css("iframe[src*='hcaptcha']")).await {
        println!("    Found hCaptcha iframe");

        // Switch to the hcaptcha iframe
        let frame_tab = tab.switch_to_frame(&iframe).await?;
        println!("    Switched to hCaptcha frame");

        // Wait for and click the checkbox
        match frame_tab
            .wait_for_element_timeout(By::css("#checkbox"), Duration::from_secs(5))
            .await
        {
            Ok(checkbox) => {
                println!("    Found checkbox, clicking...");
                checkbox.click().await?;
                sleep(Duration::from_secs(2)).await;
                println!("    ✓ Clicked hCaptcha checkbox");
            }
            Err(_) => {
                println!("    Checkbox not found in iframe");
            }
        }

        println!("    Switched back to main frame");
    } else {
        println!("    No hCaptcha iframe found");
    }
    println!();

    // ========================================================================
    // Check result
    // ========================================================================

    println!("[7] Checking result...");
    let url = tab.get_url().await?;
    println!("    URL: {url}");

    if url.contains("register") {
        println!("    ⚠ Still on registration - checking for errors");
        for err in tab.find_elements(By::css("[class*='error']")).await? {
            if let Ok(text) = err.get_text().await
                && !text.is_empty()
            {
                println!("    Error: {text}");
            }
        }
    } else if url.contains("verify") || url.contains("captcha") {
        println!("    ⚠ Verification/CAPTCHA required");
    } else if url.contains("channels") || url.contains("app") {
        println!("    ✓ Registration successful!");
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

async fn fill_input(tab: &Tab, selector: By, value: &str) -> Result<()> {
    let input = tab.wait_for_element(selector).await?;
    input.type_text(value).await?;
    sleep(Duration::from_millis(500)).await;
    Ok(())
}

async fn select_role_dropdown(tab: &Tab, selector: By, value: &str) -> Result<()> {
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

fn random_string(len: usize) -> String {
    let s = seed();
    let chars = "abcdefghijklmnopqrstuvwxyz0123456789";
    (0..len)
        .map(|i| {
            let idx = ((s >> (i * 4)) % chars.len() as u128) as usize;
            chars.chars().nth(idx).unwrap()
        })
        .collect()
}

fn random_password() -> String {
    let s = seed();
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%";
    (0..16)
        .map(|i| {
            let idx = ((s >> (i * 3)) % chars.len() as u128) as usize;
            chars.chars().nth(idx).unwrap()
        })
        .collect()
}

fn random_email() -> String {
    let domains = ["gmail.com", "yahoo.com", "hotmail.com", "outlook.com"];
    format!("{}@{}", random_string(8), domains[(seed() % 4) as usize])
}

fn random_birthdate() -> (u32, u32, u32) {
    let s = seed() as u32;
    (
        2024 - 18 - (s % 47),
        1 + (s / 100) % 12,
        1 + (s / 1000) % 28,
    )
}

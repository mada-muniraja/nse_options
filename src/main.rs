// This file is part of the BankNifty Options Filter project.
use chrono::{Datelike, Local, Months, Timelike};
use reqwest::Client;
use serde_json::{Value, json};
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_path = "NSE.json";
    let output_path = "banknifty.json";

    // Print current working directory and list files for debugging
    let cwd = std::env::current_dir().unwrap();
    println!("Current working directory: {}", cwd.display());
    let entries = std::fs::read_dir(&cwd).unwrap();
    println!("Files in current directory:");
    for entry in entries {
        if let Ok(entry) = entry {
            println!("- {}", entry.file_name().to_string_lossy());
        }
    }

    // Fetch current BankNifty price
    let banknifty_price = fetch_banknifty_price().await?;
    println!("Current BankNifty Price: {}", banknifty_price);

    // Check if input file exists
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' not found.", input_path);
        std::process::exit(1);
    }

    // Print absolute path and check file size
    let abs_path =
        std::fs::canonicalize(input_path).unwrap_or_else(|_| Path::new(input_path).to_path_buf());
    println!("Reading input file at: {}", abs_path.display());
    let metadata = std::fs::metadata(&abs_path).unwrap();
    println!("File size: {} bytes", metadata.len());
    if metadata.len() == 0 {
        eprintln!("Error: Input file '{}' is empty.", abs_path.display());
        std::process::exit(1);
    }
    // Print first 200 bytes for debugging
    let mut file_preview = File::open(&abs_path).unwrap();
    let mut buffer = [0; 200];
    let n = file_preview.read(&mut buffer).unwrap_or(0);
    println!(
        "First {} bytes: {}",
        n,
        String::from_utf8_lossy(&buffer[..n])
    );

    // Print first 32 bytes as hex to check for BOM or encoding issues
    let mut file_hex = File::open(&abs_path).unwrap();
    let mut hex_buffer = [0u8; 32];
    let n_hex = file_hex.read(&mut hex_buffer).unwrap_or(0);
    print!("First {} bytes in hex:", n_hex);
    for b in &hex_buffer[..n_hex] {
        print!(" {:02x}", b);
    }
    println!("");

    // Add a short delay to avoid race conditions (especially in CI)
    use std::{thread, time};
    let delay = time::Duration::from_millis(500);
    println!("Sleeping for 500ms before reading NSE.json to avoid race conditions...");
    thread::sleep(delay);

    // Re-check file size and contents before parsing
    let metadata = std::fs::metadata(&abs_path).unwrap();
    if metadata.len() == 0 {
        eprintln!("[ERROR] NSE.json is empty just before parsing. Aborting.");
        std::process::exit(1);
    }
    let mut file_check = File::open(&abs_path).unwrap();
    let mut preview = String::new();
    let _ = file_check.read_to_string(&mut preview);
    if preview.trim().is_empty() {
        eprintln!("[ERROR] NSE.json contains only whitespace just before parsing. Aborting.");
        std::process::exit(1);
    }
    // Optionally, print first 200 chars for debug
    println!(
        "First 200 chars before parsing: {}",
        &preview.chars().take(200).collect::<String>()
    );
    // Try parsing as JSON, with error context
    let data: Value = match serde_json::from_str(&preview) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("[ERROR] Failed to parse NSE.json as JSON: {}", e);
            eprintln!(
                "First 200 chars: {}",
                &preview.chars().take(200).collect::<String>()
            );
            std::process::exit(1);
        }
    };

    // Filter for Banknifty options within 3000 points of current price
    let filtered_options = filter_options(&data, banknifty_price);

    // Write the filtered data to a new JSON file
    let file = File::create(output_path)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &filtered_options)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    println!(
        "Successfully filtered {} NSE options and saved to '{}'.",
        filtered_options.as_array().unwrap_or(&vec![]).len(),
        output_path
    );
    Ok(())
}

async fn fetch_banknifty_price() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let response = client
        .get("https://www.nseindia.com/api/option-chain-indices?symbol=BANKNIFTY")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .header("Referer", "https://www.nseindia.com/option-chain")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let text = response
        .text()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let json_data: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    if let Some(records) = json_data.get("records") {
        if let Some(underlying_value) = records.get("underlyingValue") {
            if let Some(price) = underlying_value.as_f64() {
                return Ok(price);
            }
        }
    }

    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Could not find BankNifty price in the API response",
    )))
}

fn filter_options(data: &Value, banknifty_price: f64) -> Value {
    let mut initial_filtered_options = Vec::new();
    let mut final_filtered_options = Vec::new();

    // Get current date and calculate the end of the third month from now
    let now = Local::now();
    let start_of_month = now
        .with_day(1)
        .unwrap()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();
    let end_of_range = start_of_month + Months::new(3);
    let end_of_range_timestamp = end_of_range.timestamp_millis();

    // First filter: by name and expiry date
    if let Some(array) = data.as_array() {
        for item in array {
            if let Some(obj) = item.as_object() {
                let is_banknifty = obj
                    .get("name")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .eq_ignore_ascii_case("BANKNIFTY");

                let expiry_timestamp = obj.get("expiry").and_then(|e| e.as_i64()).unwrap_or(0);

                let is_within_three_months = expiry_timestamp > start_of_month.timestamp_millis()
                    && expiry_timestamp <= end_of_range_timestamp;

                if is_banknifty && is_within_three_months {
                    initial_filtered_options.push(item.clone());
                }
            }
        }
    } else if let Some(obj) = data.as_object() {
        if let Some(data_array) = obj.get("data").and_then(|d| d.as_array()) {
            for item in data_array {
                if let Some(obj) = item.as_object() {
                    let is_banknifty = obj
                        .get("name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .eq_ignore_ascii_case("BANKNIFTY");

                    let expiry_timestamp = obj.get("expiry").and_then(|e| e.as_i64()).unwrap_or(0);

                    let is_within_three_months = expiry_timestamp
                        > start_of_month.timestamp_millis()
                        && expiry_timestamp <= end_of_range_timestamp;

                    if is_banknifty && is_within_three_months {
                        initial_filtered_options.push(item.clone());
                    }
                }
            }
        }
    }

    println!(
        "Initially filtered {} BankNifty options based on name and expiry date.",
        initial_filtered_options.len()
    );

    // Second filter: by strike price range
    for item in initial_filtered_options {
        if let Some(obj) = item.as_object() {
            let strike_price = obj
                .get("strike_price")
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0);
            let is_within_price_range = strike_price >= banknifty_price - 3000.0
                && strike_price <= banknifty_price + 3000.0;

            if is_within_price_range {
                final_filtered_options.push(item.clone());
            }
        }
    }
    println!(
        "After applying strike price range filter, {} options remain.",
        final_filtered_options.len()
    );

    // Sort the final filtered options by expiry date
    final_filtered_options.sort_by(|a, b| {
        let expiry_a = a
            .as_object()
            .and_then(|obj| obj.get("expiry"))
            .and_then(|e| e.as_i64())
            .unwrap_or(0);
        let expiry_b = b
            .as_object()
            .and_then(|obj| obj.get("expiry"))
            .and_then(|e| e.as_i64())
            .unwrap_or(0);
        expiry_a.cmp(&expiry_b)
    });

    json!(final_filtered_options)
}

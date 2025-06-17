// This file is part of the BankNifty Options Filter project.
use chrono::{Datelike, Local, Months, Timelike};
use serde_json::{Value, json};
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::Path;
use std::{thread, time::Duration};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_path = "NSE.json";
    let output_path = "banknifty.json";
    let banknifty_previous_close = 55000.0; // Replace with actual previous close price as needed

    let cwd = std::env::current_dir().unwrap();
    println!("Current working directory: {}", cwd.display());
    let entries = std::fs::read_dir(&cwd).unwrap();
    println!("Files in current directory:");
    for entry in entries {
        if let Ok(entry) = entry {
            println!("- {}", entry.file_name().to_string_lossy());
        }
    }

    // Hardcoded BankNifty previous close price due to issues with API fetching

    println!(
        "Hardcoded BankNifty Previous Close: {}",
        banknifty_previous_close
    );

    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' not found.", input_path);
        std::process::exit(1);
    }

    let abs_path =
        std::fs::canonicalize(input_path).unwrap_or_else(|_| Path::new(input_path).to_path_buf());
    println!("Reading input file at: {}", abs_path.display());
    let metadata = std::fs::metadata(&abs_path).unwrap();
    println!("File size: {} bytes", metadata.len());

    let mut contents = String::new();
    let mut retries = 5;
    let mut success = false;

    while retries > 0 {
        contents.clear();
        if let Ok(mut file) = File::open(&abs_path) {
            if file.read_to_string(&mut contents).is_ok() && !contents.trim().is_empty() {
                success = true;
                break;
            }
        }
        eprintln!("Warning: NSE.json is empty or unreadable. Retrying in 2 seconds...");
        thread::sleep(Duration::from_secs(2));
        retries -= 1;
    }

    if !success {
        eprintln!("[ERROR] NSE.json could not be read after retries. Aborting.");
        std::process::exit(1);
    }

    println!(
        "First 200 chars before parsing: {}",
        &contents.chars().take(200).collect::<String>()
    );

    let data: Value = match serde_json::from_str(&contents) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("[ERROR] Failed to parse NSE.json as JSON: {}", e);
            std::process::exit(1);
        }
    };

    let filtered_options = filter_options(&data, banknifty_previous_close);

    let file = File::create(output_path)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &filtered_options)?;

    println!(
        "Successfully filtered {} NSE options and saved to '{}'.",
        filtered_options.as_array().unwrap_or(&vec![]).len(),
        output_path
    );
    Ok(())
}

fn filter_options(data: &Value, _banknifty_previous_close: f64) -> Value {
    let mut initial_filtered_options = Vec::new();
    let mut final_filtered_options = Vec::new();

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
    }

    println!(
        "Initially filtered {} BankNifty options based on name and expiry date.",
        initial_filtered_options.len()
    );

    for item in initial_filtered_options {
        if let Some(obj) = item.as_object() {
            let strike_price = obj
                .get("strike_price")
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0);
            // Use a fixed range since current price is not available
            let is_within_price_range = strike_price >= _banknifty_previous_close - 3000.0
                && strike_price <= _banknifty_previous_close + 3000.0;

            if is_within_price_range {
                final_filtered_options.push(item.clone());
            }
        }
    }

    println!(
        "After applying strike price range filter, {} options remain.",
        final_filtered_options.len()
    );

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

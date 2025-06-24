// This file is part of the BankNifty Options Filter project.

// Import necessary crates
use chrono::{Datelike, Local, Months, Timelike};
use serde::{Deserialize, Serialize}; // Import Deserialize and Serialize
use std::fs::File;
use std::io::{BufReader, BufWriter, Read}; // Use BufReader for potentially faster reads
use std::path::Path;
use std::{thread, time::Duration};
use tokio;

// Define a struct that represents the structure of an option in the JSON.
// This is much more efficient than using the generic serde_json::Value.
#[derive(Deserialize, Serialize, Debug, Clone)] // Derive Deserialize, Serialize, Debug and Clone
struct OptionData {
    name: String,
    expiry: Option<i64>,
    strike_price: Option<f64>,
    asset_key: Option<String>,
    asset_symbol: Option<String>,
    asset_type: Option<String>,
    exchange: Option<String>,
    exchange_token: Option<String>,
    freeze_quantity: Option<f64>,
    instrument_key: Option<String>,
    instrument_type: Option<String>,
    lot_size: Option<i64>,
    minimum_lot: Option<i64>,
    qty_multiplier: Option<f64>,
    segment: Option<String>,
    tick_size: Option<f64>,
    trading_symbol: Option<String>,
    underlying_key: Option<String>,
    underlying_symbol: Option<String>,
    underlying_type: Option<String>,
    weekly: Option<bool>,
    // Add other fields from the JSON here if you need them, otherwise they are ignored.
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_path = "NSE.json";
    let output_path = "banknifty.json";
    let strike_limit = 1000.0;
    let banknifty_previous_close = 56059.35; // Replace with actual previous close price as needed

    // --- File reading section is largely the same, but improved with BufReader ---
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' not found.", input_path);
        std::process::exit(1);
    }

    let file = File::open(input_path)?;
    let mut reader = BufReader::new(file); // Use a BufReader for efficiency
    let mut contents = String::new();

    // The retry logic remains useful for cases where the file is being written to
    let mut retries = 5;
    while retries > 0 {
        contents.clear();
        reader.read_to_string(&mut contents)?;
        if !contents.trim().is_empty() {
            break;
        }
        eprintln!("Warning: NSE.json is empty or unreadable. Retrying in 2 seconds...");
        thread::sleep(Duration::from_secs(2));
        // Reset reader to the beginning of the file for the next attempt
        use std::io::Seek;
        reader.seek(std::io::SeekFrom::Start(0))?;
        retries -= 1;
    }

    if contents.is_empty() {
        eprintln!("[ERROR] NSE.json could not be read after retries. Aborting.");
        std::process::exit(1);
    }

    // OPTIMIZATION: Deserialize directly into a Vec of our struct.
    // This is faster and uses less memory than parsing to serde_json::Value.
    let data: Vec<OptionData> = match serde_json::from_str(&contents) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("[ERROR] Failed to parse NSE.json: {}", e);
            std::process::exit(1);
        }
    };

    println!("Successfully parsed {} options from NSE.json.", data.len());

    // The filtering logic is now self-contained in the `filter_options` function.
    let mut filtered_options = filter_options(&data, banknifty_previous_close, strike_limit);

    // OPTIMIZATION: Sort using sort_by_key for clarity and efficiency.
    filtered_options.sort_by_key(|opt| opt.expiry);

    // --- Writing the output file ---
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    // Serialize the Vec<OptionData> back to JSON
    serde_json::to_writer_pretty(writer, &filtered_options)?;

    println!(
        "Successfully filtered {} options and saved to '{}'.",
        filtered_options.len(),
        output_path
    );
    Ok(())
}

/// Filters a slice of OptionData based on name, expiry date, and strike price.
fn filter_options(
    data: &[OptionData],
    banknifty_previous_close: f64,
    strike_limit: f64,
) -> Vec<OptionData> {
    // Calculate the date range once, before the loop.
    let now = Local::now();
    let start_of_month_timestamp = now
        .with_day(1)
        .unwrap()
        .with_hour(0)
        .unwrap()
        .timestamp_millis();
    let end_of_range_timestamp = (now.with_day(1).unwrap() + Months::new(3)).timestamp_millis();

    // Define the valid strike price range.
    let strike_price_range =
        (banknifty_previous_close - strike_limit)..=(banknifty_previous_close + strike_limit);

    // OPTIMIZATION: Use a single iterator chain to perform all filtering steps in one pass.
    // This avoids intermediate allocations and cloning.
    let filtered: Vec<OptionData> = data
        .iter()
        .filter(|opt| {
            // Condition 1: Name must be "BANKNIFTY"
            let is_banknifty = opt.name.eq_ignore_ascii_case("BANKNIFTY");

            // Condition 2: Expiry must be within the next 3 months
            let is_within_three_months = opt
                .expiry
                .map(|exp| exp > start_of_month_timestamp && exp <= end_of_range_timestamp)
                .unwrap_or(false);

            // Condition 3: Strike price must be within the specified range
            let is_within_price_range = opt
                .strike_price
                .map(|sp| strike_price_range.contains(&sp))
                .unwrap_or(false);

            // The option is kept only if all conditions are true
            is_banknifty && is_within_three_months && is_within_price_range
        })
        .cloned() // Clone the few items that pass the filter
        .collect(); // Collect the results into a new Vec

    println!("Filtered {} options based on all criteria.", filtered.len());

    filtered
}

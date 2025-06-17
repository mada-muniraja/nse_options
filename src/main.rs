// Description: This Rust program fetches the current BankNifty price from an API,
// reads a JSON file containing NSE options data, filters the options for BankNifty within a specified range of the current price, and writes the filtered options to a new JSON file.
//
// It uses the `tokio` runtime for asynchronous operations and handles errors gracefully.
// It also includes modules for API interaction, file I/O, and filtering logic.
// It is designed to be run in a Rust environment with the necessary dependencies included in the `Cargo.toml` file.
// The program is structured to be modular, with separate files for API calls, file operations, and filtering logic.
// This is the main entry point of the program.
use tokio;
mod api;
mod file_io;
mod filter;

use api::fetch_banknifty_price;
use file_io::{read_json_file, write_json_file};
use filter::filter_options;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_path = "NSE.json";
    let output_path = "banknifty.json";

    // Fetch current BankNifty price
    let banknifty_price = fetch_banknifty_price().await?;
    println!("Current BankNifty Price: {}", banknifty_price);

    // Read the JSON file
    let data = read_json_file(input_path)?;

    // Filter for Banknifty options within 3000 points of current price
    let filtered_options = filter_options(&data, banknifty_price);

    // Write the filtered data to a new JSON file
    write_json_file(output_path, &filtered_options)?;

    println!(
        "Successfully filtered {} NSE options and saved to '{}'.",
        filtered_options.as_array().unwrap_or(&vec![]).len(),
        output_path
    );
    Ok(())
}

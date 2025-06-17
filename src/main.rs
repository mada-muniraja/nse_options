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

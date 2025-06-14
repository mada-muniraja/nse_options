use serde_json::{Value, json};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "NSE.json";
    let output_path = "NSE_Options_Filtered.json";

    // Check if input file exists
    if !Path::new(input_path).exists() {
        eprintln!("Error: Input file '{}' not found.", input_path);
        std::process::exit(1);
    }

    // Read the JSON file
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)?;

    // Filter for Banknifty and Nifty options
    let filtered_options = filter_options(&data);

    // Write the filtered data to a new JSON file
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &filtered_options)?;

    println!(
        "Successfully filtered {} NSE options and saved to '{}'.",
        filtered_options.as_array().unwrap_or(&vec![]).len(),
        output_path
    );
    Ok(())
}

fn filter_options(data: &Value) -> Value {
    let mut filtered_options = Vec::new();

    if let Some(array) = data.as_array() {
        for item in array {
            if let Some(obj) = item.as_object() {
                if obj
                    .get("name")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .eq_ignore_ascii_case("BANKNIFTY")
                    || obj
                        .get("name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .eq_ignore_ascii_case("NIFTY")
                {
                    filtered_options.push(item.clone());
                }
            }
        }
    } else if let Some(obj) = data.as_object() {
        if let Some(data_array) = obj.get("data").and_then(|d| d.as_array()) {
            for item in data_array {
                if let Some(obj) = item.as_object() {
                    if obj
                        .get("name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .eq_ignore_ascii_case("BANKNIFTY")
                        || obj
                            .get("name")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .eq_ignore_ascii_case("NIFTY")
                    {
                        filtered_options.push(item.clone());
                    }
                }
            }
        }
    }

    json!(filtered_options)
}

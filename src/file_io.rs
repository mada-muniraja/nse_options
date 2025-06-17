use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn read_json_file(path: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    // Check if input file exists
    if !Path::new(path).exists() {
        eprintln!("Error: Input file '{}' not found.", path);
        std::process::exit(1);
    }

    // Read the JSON file
    let file =
        File::open(path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    let reader = BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    Ok(data)
}

pub fn write_json_file(
    path: &str,
    data: &Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Write the filtered data to a new JSON file
    let file =
        File::create(path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    Ok(())
}

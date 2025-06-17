# NSE Options Filter

A Rust application for filtering options instruments from the National Stock Exchange (NSE), specifically focusing on BANKNIFTY options. This tool reads options data from a JSON file, applies filtering criteria based on name, expiry date, and strike price, and outputs the filtered results to a new JSON file.

## Purpose

This project is designed to process and filter large datasets of options from the NSE, extracting only the relevant BANKNIFTY options within a specified strike price range and expiry period (next 3 months). It aims to optimize data handling with efficient JSON parsing and filtering techniques.

## Installation

Ensure you have Rust and Cargo installed on your system. You can install them from [rust-lang.org](https://www.rust-lang.org/tools/install).

Clone this repository or download the source code to your local machine.

```bash
git clone https://github.com/mada-muniraja/nse_options.git
cd nse_options
```

## Usage

1. **Prepare Input Data**: Place your NSE options data in a file named `NSE.json` in the project root directory. Ensure the JSON format matches the expected structure as defined in `src/main.rs`.

2. **Build the Project**: Compile the Rust application using Cargo.

   ```bash
   cargo build --release
   ```

3. **Run the Filter**: Execute the application to process `NSE.json` and generate `banknifty.json` with the filtered results.

   ```bash
   cargo run --release
   ```

   The filtered output will be saved to `banknifty.json` in the project root directory.

## Filtering Criteria

- **Name**: Only options with the name "BANKNIFTY" are included.
- **Expiry**: Options expiring within the next 3 months from the current date.
- **Strike Price**: Options with strike prices within ±2000 of the previous close price (default set to 55000.0, should update daily manually).

## Project Structure

- `src/main.rs`: Main source file containing the logic for reading, filtering, and writing options data.
- `Cargo.toml`: Configuration file for Rust dependencies and project metadata.
- `NSE.json`: Input file containing the raw NSE options data (not included in the repository).
- `banknifty.json`: Output file with filtered BANKNIFTY options data.

## Dependencies

- `serde` and `serde_json`: For JSON serialization and deserialization.
- `chrono`: For handling date and time calculations.
- `tokio`: For asynchronous file operations and main function execution.

## License

This project is licensed under the MIT License - see the LICENSE file for details (if applicable).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request or open an Issue for any bugs, feature requests, or improvements.

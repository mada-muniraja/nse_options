use chrono::{Datelike, Local, Months, Timelike};
use serde_json::{Value, json};

pub fn filter_options(data: &Value, banknifty_price: f64) -> Value {
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

use reqwest::Client;
use serde_json::Value;

pub async fn fetch_banknifty_price() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
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

    let json_data: Value = serde_json::from_str(&text)
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

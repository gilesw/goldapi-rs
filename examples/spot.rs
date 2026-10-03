use goldapi::{Client, Currency, Metal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::io::read_to_string(std::io::stdin())?;
    let client = Client::new(api_key.trim());
    let quote = client.spot_price(Metal::Xau, Currency::Usd).await?;
    println!("{}", serde_json::to_string_pretty(&quote)?);
    Ok(())
}

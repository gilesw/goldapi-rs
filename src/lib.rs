//! Typed async client for GoldAPI.io, generated from its published OpenAPI spec.
//!
//! ```no_run
//! use goldapi::{Client, Currency, Metal};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::new("your-api-key");
//! let quote = client.spot_price(Metal::Xau, Currency::Usd).await?;
//! println!("Price: {:?}, timestamp: {:?}", quote.price, quote.timestamp);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

mod client;
mod error;

// Keep generator output byte-identical for --check. The generated mod.rs is
// retained as an artifact but this declaration controls its lint/format scope.
#[rustfmt::skip]
#[allow(dead_code, clippy::all, clippy::pedantic, missing_docs, rustdoc::all)]
pub mod generated {
    pub mod client;
    pub mod types;
}

pub use chrono::NaiveDate;
pub use client::Client;
pub use error::Error;
pub use generated::types::{
    CurrencyCode as Currency, HistoricalPricePoint, HistoricalPriceResponse, LbmaResponse,
    LivePriceResponse, MetalHistoryResponse, MetalSymbol as Metal, RateResponse,
    RequestStatisticsResponse, StatusResponse,
};

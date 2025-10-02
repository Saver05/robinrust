//! Market data endpoints for crypto symbols.
//!
//! This module provides helpers to query best bid/ask and estimated prices
//! from the Robinhood crypto market data API.

use reqwest::Client;
use serde::{Serialize, Deserialize};
use crate::auth::Robinhood;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize)]
/// Best bid/ask snapshot for a symbol from Robinhood.
pub struct BestPriceResult {
    pub symbol: String,

    #[serde(with = "rust_decimal::serde::float")]
    pub price: Decimal,

    #[serde(with = "rust_decimal::serde::float")]
    pub bid_inclusive_of_sell_spread: Decimal,

    #[serde(with = "rust_decimal::serde::float")]
    pub sell_spread: Decimal,

    #[serde(with = "rust_decimal::serde::float")]
    pub ask_inclusive_of_buy_spread: Decimal,

    #[serde(with = "rust_decimal::serde::float")]
    pub buy_spread: Decimal,

    pub timestamp: String,
}


#[derive(Debug, Serialize, Deserialize)]
/// Response wrapper containing best price results.
pub struct BestPriceResponse {
    pub results: Vec<BestPriceResult>,
}

/// Fetch the best bid/ask for one or more symbols.
///
/// `symbols` should be Robinhood crypto pairs like "BTC-USD".
pub async fn get_best_price(rh: &Robinhood, symbols: Vec<&str>) -> Result<BestPriceResponse, reqwest::Error>{
    let mut path = String::from("/api/v1/crypto/marketdata/best_bid_ask/");
    if !symbols.is_empty() {
        path.push('?');
        for (i, sym) in symbols.iter().enumerate() {
            if i > 0 {
                path.push('&');
            }
            // Consider URL-escaping sym if needed
            path.push_str("symbol=");
            path.push_str(sym);
        }
    }
    let headers = rh.auth_headers(&path, "GET", "");
    let client = Client::new();
    let resp = client
        .get(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .send()
        .await?.json::<BestPriceResponse>().await?;
    Ok(resp)
}

#[derive(Debug, Serialize, Deserialize)]
/// Estimated execution price for a hypothetical trade request.
pub struct EstimatedPriceResult {
    pub symbol: String,

    pub side: String,

    #[serde(with = "rust_decimal::serde::float")]
    pub price: Decimal,

    #[serde(with = "rust_decimal::serde::float")]
    pub quantity: Decimal,

    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub bid_inclusive_of_sell_spread: Option<Decimal>,

    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub sell_spread: Option<Decimal>,

    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub ask_inclusive_of_buy_spread: Option<Decimal>,

    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub buy_spread: Option<Decimal>,

    pub timestamp: String,
}


#[derive(Debug, Serialize, Deserialize)]
/// Response wrapper containing estimated price results.
pub struct EstimatedPriceResponse {
    pub results: Vec<EstimatedPriceResult>,
}


/// Get an estimated execution price for a given symbol, side, and quantity.
///
/// `side` is either "bid" or "ask"; `quantity` is the trade size.
pub async fn get_estimated_price(rh: &Robinhood, symbol: &str, side: &str, quantity: Decimal) -> Result<EstimatedPriceResponse, reqwest::Error> {
    let  path = format!("/api/v1/crypto/marketdata/estimated_price/?symbol={symbol}&side={side}&quantity={quantity}");
    let headers = rh.auth_headers(&path, "GET", "");
    let client = Client::new();
    let resp = client
        .get(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .send()
        .await?.json::<EstimatedPriceResponse>().await?;
    Ok(resp)
}


#[tokio::test]
async fn test_best_price(){
    let rh = Robinhood::from_env();
    match get_best_price(&rh, vec!["BTC-USD"]).await{
        Ok(resp) =>{
            assert_eq!(resp.results.len(), 1);
            assert_eq!(resp.results[0].symbol, "BTC-USD");
        }
        Err(e) => {
            panic!("Error with best price: {}", e);
        }
    }
}

#[tokio::test]
async fn test_estimated_price(){
    let rh = Robinhood::from_env();
    match get_estimated_price(&rh, "BTC-USD", "bid",Decimal::from(1)).await{
        Ok(resp) =>{
            assert_eq!(resp.results.len(), 1);
            assert_eq!(resp.results[0].symbol, "BTC-USD");
            assert_eq!(resp.results[0].side, "bid");
            assert_eq!(resp.results[0].quantity, Decimal::from(1));
        }
        Err(e) => {
            panic!("Error with estimated price: {}", e);
        }
    }
}
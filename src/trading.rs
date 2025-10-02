use std::fmt::{format, Error};
use std::str::FromStr;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use crate::auth::Robinhood;
use rust_decimal::Decimal;
use typed_builder::TypedBuilder;
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoTradingPairsResponse{
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<TradingPairs>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingPairs{
    pub asset_code: String,
    pub quote_code: String,
    pub quote_increment: String,
    pub asset_increment: String,
    pub max_order_size: String,
    pub status: String,
    pub symbol: String,
}

impl TradingPairs{
    pub fn check_valid_trade(&self, quantity: Decimal) -> bool{
        let max_order_size = Decimal::from_str(&self.max_order_size).unwrap();
        let min_order_size = Decimal::from_str(&self.asset_increment).unwrap();
        quantity <= max_order_size && quantity >= min_order_size
    }
}

pub async fn get_crypto_trading_pairs(rh: &Robinhood, symbols: Vec<&str>) -> Result<CryptoTradingPairsResponse, reqwest::Error>{
    let mut path = String::from("/api/v1/crypto/trading/trading_pairs/");
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
        .await?.json::<CryptoTradingPairsResponse>().await?;
    Ok(resp)
}

#[tokio::test]
async fn test_get_trading_pairs(){
    let rh = Robinhood::from_env();
    match get_crypto_trading_pairs(&rh, vec!["BTC-USD"]).await{
        Ok(resp) => {
            assert_eq!(resp.results[0].asset_code, "BTC");
            assert_eq!(resp.results[0].quote_code, "USD");
        }
        Err(e) => {
            panic!("Error with trading pairs: {}", e);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoHoldingsResponse{
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<CryptoHoldings>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoHoldings{
    pub account_number: String,
    pub asset_code: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_quantity: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub quantity_available_for_trading: Decimal,
}

pub async fn get_crypto_holdings(rh: &Robinhood, symbols: Vec<&str>) -> Result<CryptoHoldingsResponse, reqwest::Error>{
    let mut path = String::from("/api/v1/crypto/trading/holdings/");
    if !symbols.is_empty() {
        path.push('?');
        for (i, sym) in symbols.iter().enumerate() {
            if i > 0 {
                path.push('&');
            }
            // Consider URL-escaping sym if needed
            path.push_str("asset_code=");
            path.push_str(sym);
        }
    }
    let headers = rh.auth_headers(&path, "GET", "");
    let client = Client::new();
    let resp = client
        .get(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .send()
        .await?.json::<CryptoHoldingsResponse>().await?;
    Ok(resp)
}

#[tokio::test]
async fn test_get_crypto_holdings(){
    let rh = Robinhood::from_env();
    match get_crypto_holdings(&rh, vec!["BTC"]).await{
        Ok(resp) => {
            assert_eq!(resp.next, None);
        }
        Err(e) => {
            panic!("Error with crypto holdings: {}", e);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoOrdersResponse {
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<CryptoOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoOrder {
    pub id: String,
    pub account_number: String,
    pub symbol: String,
    pub client_order_id: String,
    pub side: String,
    pub executions: Vec<Executions>,

    #[serde(rename = "type")]
    pub order_type: String,

    pub state: String,

    // May be absent or null
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub average_price: Option<Decimal>,

    // Always present (in your sample); string number
    #[serde(with = "rust_decimal::serde::str")]
    pub filled_asset_quantity: Decimal,

    pub created_at: String,
    pub updated_at: String,

    pub market_order_config: Option<MarketOrderConfig>,
    pub limit_order_config: Option<LimitOrderConfig>,
    pub stop_loss_order_config: Option<StopLossOrderConfig>,
    pub stop_limit_order_config: Option<StopLimitOrderConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Executions {
    pub effective_price: String,
    pub quantity: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct MarketOrderConfig {
    #[serde(with = "rust_decimal::serde::str")]
    pub asset_quantity: Decimal,
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct LimitOrderConfig {
    // Any of these may be omitted; they also arrive as strings
    #[serde(default, with = "rust_decimal::serde::str_option")]
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_amount: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_quantity: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub limit_price: Option<Decimal>,
    // Can be absent; plain Option<String> doesn't need `default`
    pub time_in_force: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct StopLossOrderConfig {
    #[serde(default, with = "rust_decimal::serde::str_option")]
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_amount: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_quantity: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub stop_price: Option<Decimal>,
    pub time_in_force: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct StopLimitOrderConfig {
    #[serde(default, with = "rust_decimal::serde::str_option")]
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_amount: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_quantity: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub limit_price: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub stop_price: Option<Decimal>,
    pub time_in_force: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct GetCryptoOrderParams{
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_start: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_end: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub type_: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_start: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_end: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}
pub async fn get_crypto_orders(rh: &Robinhood,params: GetCryptoOrderParams) -> Result<CryptoOrdersResponse, reqwest::Error>{
    let path = String::from("/api/v1/crypto/trading/orders/");
    let headers = rh.auth_headers(&path, "GET", "");
    let client = Client::new();
    let resp = client
        .get(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .query(&params)
        .send()
        .await?.json::<CryptoOrdersResponse>().await?;
    Ok(resp)
}

#[tokio::test]
async fn test_get_crypto_orders(){
    let rh = Robinhood::from_env();
    match get_crypto_orders(&rh, GetCryptoOrderParams::builder().build()).await{
        Ok(resp) => {
            assert_eq!(resp.previous, None);
        }
        Err(e) => {
            panic!("Error with crypto orders: {}", e);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct CreateCyptoOrderParams{
    pub symbol: String,
    pub client_order_id: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_order_config: Option<MarketOrderConfig>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_order_config: Option<LimitOrderConfig>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss_order_config: Option<StopLossOrderConfig>,
    #[builder(default, setter(strip_option, into))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_limit_order_config: Option<StopLimitOrderConfig>,
}

#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct CreateCryptoOrderResponse{
    pub id: String,
    pub account_number: String,
    pub symbol: String,
    pub client_order_id: String,
    pub side: String,
    pub executions: Vec<Executions>,
    #[serde(rename = "type")]
    pub order_type: String,
    pub state: String,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub average_price: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::float_option", default)]
    pub filled_asset_quantity: Option<Decimal>,
    pub created_at: String,
    pub updated_at: String,
    pub market_order_config: Option<MarketOrderConfig>,
    pub limit_order_config: Option<LimitOrderConfig>,
    pub stop_loss_order_config: Option<StopLossOrderConfig>,
    pub stop_limit_order_config: Option<StopLimitOrderConfig>,
}

pub async fn create_crypto_order(rh: &Robinhood, param: CreateCyptoOrderParams) -> Result<CreateCryptoOrderResponse, reqwest::Error>{
    let path = "/api/v1/crypto/trading/orders/";
    let headers = rh.auth_headers(&path, "POST", &serde_json::to_string(&param).unwrap());
    let client = Client::new();
    let resp = client
        .post(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .json(&param)
        .send()
        .await?.json::<CreateCryptoOrderResponse>().await?;
    Ok(resp)
}



pub async fn cancel_crypto_order(rh: &Robinhood, id: String) -> Result<String, reqwest::Error>{
    let path = format!("/api/v1/crypto/trading/orders/{}/cancel/", id);
    let headers = rh.auth_headers(&path, "POST", "");
    let client = Client::new();
    let resp = client
        .post(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .send()
        .await?;
    let body = resp.text().await?;
    let cleaned = body.trim_matches('"').to_string();
    Ok(cleaned)
}

#[tokio::test]
async fn test_create_cancel_crypto_order(){
    let rh = Robinhood::from_env();
    let resp = create_crypto_order(&rh, CreateCyptoOrderParams::builder()
        .symbol("XRP-USD".to_string())
        .client_order_id(Uuid::new_v4().to_string())
        .order_type("limit".to_string())
        .side("buy".to_string())
        .limit_order_config(LimitOrderConfig::builder()
            .asset_quantity(Decimal::from(1))
            .limit_price(Option::from(Decimal::from(1)))
            .time_in_force(Option::from("gfd".to_string())).build())
        .build()).await;

    let id = match resp{
        Ok(resp) => {
            assert_eq!(resp.side, "buy");
            assert_eq!(resp.symbol, "XRP-USD");
            resp.id
        }
        Err(e) => {
            panic!("Error with crypto orders: {}", e);
        }
    };

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    let cancel_resp = format!("Cancel request has been submitted for order {id}");
    match cancel_crypto_order(&rh, id).await{
        Ok(resp) => {
            assert_eq!(resp, cancel_resp);
        }
        Err(e) => {
            panic!("Error with crypto orders: {}", e);
        }
    }
}






# robinrust — A Rust client for the Robinhood Crypto API

A lightweight, async Rust library for interacting with Robinhood's Crypto trading endpoints. It includes helpers for request signing, market data (best bid/ask and estimated price), account holdings, and basic order management.

Note: This project is community-maintained and is not affiliated with, endorsed by, or supported by Robinhood Markets, Inc. Use at your own risk.

## Features
- Ed25519 request signing with API key headers
- Market data
  - Best bid/ask for one or more symbols
  - Estimated price quotes for bid/ask given a quantity
- Trading
  - Query trading pairs and min/max increments
  - View crypto holdings
  - List existing orders with flexible filters
  - Create and cancel crypto orders (market/limit/stop/stop-limit)
- Strong types with serde and rust_decimal
- Async HTTP via reqwest + tokio

## Getting started

### Prerequisites
- A Robinhood Crypto API key and signing key material read here https://docs.robinhood.com/crypto/trading/#section/Introduction for more detail
### Install
This is a library crate. You can consume it by cloning and adding it to your workspace, or by using a git dependency in Cargo.toml.

Example (git dependency):

```toml
[dependencies]
robinrust = { git = "https://github.com/your-org/robinrust" }
```

If you are developing locally inside the same workspace, you can use a path dependency instead:

```toml
[dependencies]
robinrust = { path = "../robinrust" }
```

### Environment variables
The library expects credentials in your environment. A `.env` file is supported via the `dotenv` crate.

Required variables:
- ROBINHOOD_API_KEY — your API key (starts with rh-api-...)
- ROBINHOOD_SIGNING_PRIVATE_B64 — base64-encoded 32-byte Ed25519 private key
- ROBINHOOD_PUBLIC_KEY — your Ed25519 public key (string Robinhood associates with the API key)

Example `.env`:

```
ROBINHOOD_API_KEY=rh-api-xxxxxxxxxxxxxxxx
ROBINHOOD_SIGNING_PRIVATE_B64=BASE64_OF_32_BYTE_ED25519_SECRET
ROBINHOOD_PUBLIC_KEY=ed25519-pub-key-string
```

## Usage
All calls are async. Use within a Tokio runtime.

### Initialize client and fetch best bid/ask
```rust
use robinrust::auth::Robinhood;
use robinrust::market_data::get_best_price;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rh = Robinhood::from_env();
    let resp = get_best_price(&rh, vec!["BTC-USD", "ETH-USD"]).await?;
    for r in resp.results {
        println!("{} best price: {}", r.symbol, r.price);
    }
    Ok(())
}
```

### Estimated price quote
```rust
use robinrust::auth::Robinhood;
use robinrust::market_data::get_estimated_price;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rh = Robinhood::from_env();
    let quote = get_estimated_price(&rh, "BTC-USD", "bid", Decimal::from(1)).await?;
    println!("{:?}", quote);
    Ok(())
}
```

### Trading pairs and validating order size
```rust
use robinrust::auth::Robinhood;
use robinrust::trading::{get_crypto_trading_pairs, TradingPairs};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rh = Robinhood::from_env();
    let pairs = get_crypto_trading_pairs(&rh, vec!["BTC-USD"]).await?;
    let btc_usd = &pairs.results[0];
    let ok = btc_usd.check_valid_trade(Decimal::from(1));
    println!("Size valid? {}", ok);
    Ok(())
}
```

### List orders
```rust
use robinrust::auth::Robinhood;
use robinrust::trading::{get_crypto_orders, GetCryptoOrderParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rh = Robinhood::from_env();
    let orders = get_crypto_orders(&rh, GetCryptoOrderParams::builder().symbol("BTC-USD").build()).await?;
    println!("Found {} orders", orders.results.len());
    Ok(())
}
```

### Place and cancel an order
```rust
use robinrust::auth::Robinhood;
use robinrust::trading::{create_crypto_order, cancel_crypto_order, CreateCyptoOrderParams, MarketOrderConfig};
use uuid::Uuid;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rh = Robinhood::from_env();

    // Create a market buy order for 0.001 BTC
    let params = CreateCyptoOrderParams::builder()
        .symbol("BTC-USD")
        .client_order_id(Uuid::new_v4().to_string())
        .side("buy")
        .order_type("market")
        .market_order_config(MarketOrderConfig { asset_quantity: Decimal::from_str_radix("0.001", 10).unwrap() })
        .build();

    let order = create_crypto_order(&rh, params).await?;
    println!("Created order {} state={}", order.id, order.state);

    // Optionally cancel
    let cancel_resp = cancel_crypto_order(&rh, order.id.clone()).await?;
    println!("Cancel response: {}", cancel_resp);

    Ok(())
}
```

### Holdings
```rust
use robinrust::auth::Robinhood;
use robinrust::trading::get_crypto_holdings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rh = Robinhood::from_env();
    let holdings = get_crypto_holdings(&rh, vec!["BTC"]).await?;
    println!("{:?}", holdings.results);
    Ok(())
}
```

## Notes and caveats
- This API and its requirements may change without notice; fields are modeled to the best of our knowledge.
- Some numeric fields arrive as strings in Robinhood responses; rust_decimal with serde helpers is used to preserve precision.
- Time fields are passed through as strings; consider parsing to DateTime if needed in your app.
- You are responsible for complying with Robinhood’s Terms of Service and applicable laws.
- You are responsible for any errors causing loss of funds, I am not held responsible for any losses.
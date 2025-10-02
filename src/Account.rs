use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::auth::Robinhood;


#[derive(Serialize, Deserialize)]
pub struct AccountInfo{
    pub account_number: String,
    pub status: String,
    pub buying_power: String,
    pub buying_power_currency: String,
}

pub async fn get_account_info(rh: &Robinhood) -> Result<AccountInfo, reqwest::Error>{
    let path = "/api/v1/crypto/trading/accounts/";
    let headers = rh.auth_headers(path, "GET", "");
    let client = Client::new();
    let resp = client
        .get(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .send()
        .await?.json::<AccountInfo>().await?;
    Ok(resp)
}

#[tokio::test]
async fn test_get_account_info(){
    let rh = Robinhood::from_env();
    match get_account_info(&rh).await {
        Ok(info) => {
            assert_eq!(info.status, "active");
            assert_eq!(info.buying_power_currency, "USD");
        }
        Err(e) => panic!("error: {e}")
    }
}

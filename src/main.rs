mod client;
use crate::client::Client;
use serde_json::json;
use serde_json::Value;
use std::time::Instant;

async fn latency_test(
    x: Value,
    client: &Client,
    public: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let res = match public {
        true => client.public_api(x).await?,
        false => client.private_api(x).await?,
    };
    let t = start.elapsed();

    println!(
        "================\n\nResponse ({} ms): {:?}\n\n================\n",
        t.as_millis(),
        res
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    latency_test(
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "public/get_index",
            "params": {
                "currency": "BTC"
            }
        }),
        &client,
        true,
    )
    .await?;

    latency_test(
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "private/buy",
            "params": {
                "instrument_name": "BTC-PERPETUAL",
                "amount": 10,
                "type": "limit",
                "reduce_only": false,
                "price": 30000
            }
        }),
        &client,
        false,
    )
    .await?;

    let sub_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "public/subscribe",
        "params": {
            "channels": [
                "deribit_price_index.btc_usd"
            ]
        }
    });
    client.public_subscribe(sub_request).await?;

    Ok(())
}

use dotenv::dotenv;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::{json, Value};
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

macro_rules! send_request {
    ($write:expr, $request:expr) => {{
        use tokio_tungstenite::tungstenite::Message;
        let message = Message::Text($request.to_string().into());
        $write.send(message).await?;
    }};
}

macro_rules! ev {
    ($key:expr) => {{
        env::var($key).expect(&format!("[error] env var {} not found", $key))
    }};
}

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub client_secret: String,
    pub client_url: String,
}

impl Client {
    pub fn new() -> Self {
        dotenv().ok();

        Client {
            client_id: ev!("CLIENT_ID"),
            client_secret: ev!("CLIENT_SECRET"),
            client_url: ev!("CLIENT_URL"),
        }
    }

    async fn get_stream(
        &self,
    ) -> Result<
        (
            SplitSink<
                WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
                Message,
            >,
            SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
        ),
        Box<dyn std::error::Error>,
    > {
        let (ws_stream, _) = connect_async(&self.client_url).await?;
        Ok(ws_stream.split())
    }

    pub async fn private_api(&self, request: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let (mut write, mut read) = self.get_stream().await?;

        let auth_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "public/auth",
            "params": {
                "grant_type": "client_credentials",
                "client_id": self.client_id,
                "client_secret": self.client_secret
            }
        });
        send_request!(write, auth_request);

        if (read.next().await).is_some() {
            send_request!(write, request);

            if let Some(msg) = read.next().await {
                let response = msg?.into_text()?;
                let json_response: Value = serde_json::from_str(&response)?;
                return Ok(json_response);
            }
        }

        Err("No response from server".into())
    }

    pub async fn public_api(&self, request: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let (mut write, mut read) = self.get_stream().await?;
        send_request!(write, request);

        if let Some(msg) = read.next().await {
            let response = msg?.into_text()?;
            let json_response: Value = serde_json::from_str(&response)?;
            return Ok(json_response);
        }

        Err("No response from server".into())
    }

    pub async fn public_subscribe(&self, request: Value) -> Result<(), Box<dyn std::error::Error>> {
        let (mut write, mut read) = self.get_stream().await?;
        send_request!(write, request);

        // Continuously read subscription responses
        while let Some(msg) = read.next().await {
            let response = msg?.into_text()?;
            println!("Subscription Response: {}", response);
        }

        Ok(())
    }
}

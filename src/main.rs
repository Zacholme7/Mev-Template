use alloy::providers::{ProviderBuilder, WsConnect};
use dotenv::dotenv;
use anyhow::Result;
use strategy::run_strategy;
use std::sync::Arc;
use crate::stream::*;
use tokio::sync::broadcast;
use log::info;

mod pools;
mod stream;
mod strategy;
mod util;

#[tokio::main]
async fn main() -> Result<()>{
    // init logging and config env
    dotenv().ok();

    // Constrcut the providers
    let http_url = std::env::var("HTTP_URL")?;
    let http_provider = Arc::new(ProviderBuilder::new().on_http(http_url.parse()?));

    let ws_url = std::env::var("WSS_URL")?;
    let ws = WsConnect::new(ws_url);
    let ws_provider = Arc::new(ProviderBuilder::new().on_ws(ws).await?);

    // channels for communication
    let (mempool_tx, mut mempool_rx) = broadcast::channel(100);
    let (block_tx, mut block_rx) = broadcast::channel(10);
    let (event_tx, mut event_rx) = broadcast::channel(100);

    // start a mempool and block stream
    // we dont care about the handle, they are meant to run indefinitely 
    tokio::task::spawn(
        stream_mempool(ws_provider.clone(), mempool_tx.clone())
    );
    tokio::task::spawn(
        stream_blocks(ws_provider.clone(), block_tx.clone())
    );
    tokio::task::spawn(
        run_strategy(ws_provider.clone(), http_provider.clone(), mempool_tx.clone(), block_tx.clone(), event_tx.clone())
    );

    while let Ok(event) = event_rx.recv().await {
        info!("Got a new event!!");
    }

    Ok(())
}
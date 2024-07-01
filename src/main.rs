use crate::strategy::StrategyActor;
use crate::stream::StreamActor;
use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, WsConnect};
use anyhow::Result;
use dashmap::DashMap;
use dotenv::dotenv;
use log::{info, LevelFilter};
use pool_sync::{Chain, Pool, PoolInfo, PoolSync, PoolType};
use std::sync::Arc;
use tokio::sync::broadcast;
use util::Event;

mod pools;
mod strategy;
mod stream;
mod util;
mod tracing;

#[tokio::main]
async fn main() -> Result<()> {
    // init logging and config env
    dotenv().ok();
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    // Construct the providers
    let http_url = std::env::var("HTTP_URL")?;
    let http_provider = Arc::new(ProviderBuilder::new().on_http(http_url.parse()?));

    let ws_url = std::env::var("WSS_URL")?;
    let ws = WsConnect::new(ws_url);
    let ws_provider = Arc::new(ProviderBuilder::new().on_ws(ws).await?);

    // sync all of the pools and convert into dashmap for easy querying
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV3)
        .chain(Chain::Ethereum)
        .build()?;
    let pools = pool_sync.sync_pools(http_provider.clone()).await?;
    let pool_map: Arc<DashMap<Address, Pool>> = Arc::new(DashMap::new());
    for pool in pools {
        pool_map.insert(pool.address(), pool);
    }

    // channels for communication
    let (mempool_tx, mempool_rx) = broadcast::channel(100);
    let (block_tx, block_rx) = broadcast::channel(10);
    let (event_tx, mut event_rx) = broadcast::channel::<Event>(100);

    // Start the stream actor
    StreamActor::new(block_tx.clone(), mempool_tx.clone())
        .start_streams(ws_provider.clone())
        .await;

    // Start the strategy actor, will start various processing workers
    StrategyActor::new(block_rx, mempool_rx, event_tx, pool_map)
        .start_strategy(http_provider.clone())
        .await;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    // event processing, just gives general information
    while let Ok(event) = event_rx.recv().await {
        info!("Got a new event: {:?}", event);
    }

    Ok(())
}

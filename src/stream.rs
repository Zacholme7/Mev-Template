use std::sync::Arc;
use alloy::providers::Provider;
use alloy::{providers::RootProvider, pubsub::PubSubFrontend};
use alloy::primitives::TxHash;
use futures::StreamExt;
use alloy::rpc::types::Block;
use log::warn;
use tokio::sync::broadcast::Sender;

/// Stream pending transactions from the mempool
pub async fn stream_mempool(ws_provider: Arc<RootProvider<PubSubFrontend>>, tx_sender: Sender<TxHash>) {
    // subscribe to all of the pending transactions
    let sub = ws_provider.subscribe_pending_transactions().await.unwrap();
    let mut stream = sub.into_stream();

    // wait for a new transaction
    while let Some(tx_hash) = stream.next().await {
            match tx_sender.send(tx_hash) {
                Err(e) => warn!("Failed to send transaciton hash: {} ", e),
                _ => {}
            }
    }
    panic!("The mempool stream has exited");
}
    
/// Streams new blocks
pub async fn stream_blocks(ws_provider: Arc<RootProvider<PubSubFrontend>>, block_sender: Sender<Block>) {
    // subscribe to the block stream
    let sub = ws_provider.subscribe_blocks().await.unwrap();
    let mut stream = sub.into_stream();

    // wait for a new block
    while let Some(block) = stream.next().await {
        match block_sender.send(block) {
            Err(e) => warn!("Failed to send block: {}", e),
            _ => ()
        }
    }
}
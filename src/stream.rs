use alloy::primitives::TxHash;
use alloy::providers::Provider;
use alloy::rpc::types::Block;
use alloy::{providers::RootProvider, pubsub::PubSubFrontend};
use futures::StreamExt;
use log::warn;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

/// Actor responsible for streaming in blocks and pending transactions
pub struct StreamActor {
    /// Sender for blocks
    block_sender: Sender<Block>,
    /// Sender for pending transactions
    tx_sender: Sender<TxHash>,
}

impl StreamActor {
    /// Construct the stream actor with the block and tx senders
    pub fn new(block_sender: Sender<Block>, tx_sender: Sender<TxHash>) -> Self {
        Self {
            block_sender,
            tx_sender,
        }
    }

    /// Start the block and tx streams
    pub async fn start_streams(&self, ws_provider: Arc<RootProvider<PubSubFrontend>>) {
        // start the block stream
        tokio::task::spawn(StreamActor::stream_blocks(
            ws_provider.clone(),
            self.block_sender.clone(),
        ));
        tokio::task::spawn(StreamActor::stream_mempool(
            ws_provider.clone(),
            self.tx_sender.clone(),
        ));
        // start the mempool stream
    }

    // Streams new blocks
    async fn stream_blocks(
        ws_provider: Arc<RootProvider<PubSubFrontend>>,
        block_sender: Sender<Block>,
    ) {
        // subscribe to the block stream
        let sub = ws_provider.subscribe_blocks().await.unwrap();
        let mut stream = sub.into_stream();

        // wait for a new block
        while let Some(block) = stream.next().await {
            if let Err(e) = block_sender.send(block) {
                warn!("Failed to send block: {}", e);
            }
        }
    }

    // Stream pending transactions from the mempool
    async fn stream_mempool(
        ws_provider: Arc<RootProvider<PubSubFrontend>>,
        tx_sender: Sender<TxHash>,
    ) {
        // subscribe to all of the pending transactions
        let sub = ws_provider.subscribe_pending_transactions().await.unwrap();
        let mut stream = sub.into_stream();

        // wait for a new transaction
        while let Some(tx_hash) = stream.next().await {
            if let Err(e) = tx_sender.send(tx_hash) {
                warn!("Failed to send transaciton hash: {} ", e)
            }
        }
    }
}

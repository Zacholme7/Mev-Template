use crate::util::Event;
use alloy::primitives::TxHash;
use alloy::providers::RootProvider;
use alloy::rpc::types::Block;
use alloy::transports::http::{Client, Http};
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

/// Core strategy actor, coordinates and manages the strategy
pub struct StrategyActor {
    /// Block receiver,
    block_rx: Receiver<Block>,
    /// Transaction receiver
    tx_rx: Receiver<TxHash>,
    /// Event sender
    event_tx: Sender<Event>,
}

impl StrategyActor {
    /// Construct a new StrategyActor
    pub fn new(
        block_rx: Receiver<Block>,
        tx_rx: Receiver<TxHash>,
        event_tx: Sender<Event>,
    ) -> Self {
        Self {
            block_rx,
            tx_rx,
            event_tx,
        }
    }

    pub async fn start_strategy(self, provider: Arc<RootProvider<Http<Client>>>) {
        // block worker
        tokio::task::spawn(StrategyActor::block_worker(self.block_rx));

        // mempool worker
        tokio::task::spawn(StrategyActor::tx_worker(self.tx_rx));
    }

    pub async fn block_worker(mut block_rx: Receiver<Block>) {
        while let Ok(block) = block_rx.recv().await {
            info!("Got a new block: {:?}", block);
        }
    }

    pub async fn tx_worker(mut tx_rx: Receiver<TxHash>) {
        while let Ok(tx_hash) = tx_rx.recv().await {
            info!("Got a new pending tx: {:?}", tx_hash);
        }
    }
}


use crate::util::Event;
use alloy::primitives::Address;
use alloy::providers::RootProvider;
use alloy::providers::ext::TraceApi;
use alloy::rpc::types::eth::TransactionRequest;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::rpc::types::trace::parity::TraceType;
use alloy::rpc::types::trace::tracerequest::TraceCallRequest;
use alloy::rpc::types::Block;
use alloy::rpc::types::Transaction;
use alloy::transports::http::{Client, Http};
use dashmap::DashMap;
use log::info;
use pool_sync::Pool;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

/// Core strategy actor, coordinates and manages the strategy
pub struct StrategyActor {
    /// Block receiver,
    block_rx: Receiver<Block>,
    /// Transaction receiver
    tx_rx: Receiver<Transaction>,
    /// Event sender
    event_tx: Sender<Event>,
    /// All Synced Pools
    pools: Arc<DashMap<Address, Pool>>,
}

impl StrategyActor {
    /// Construct a new StrategyActor
    pub fn new(
        block_rx: Receiver<Block>,
        tx_rx: Receiver<Transaction>,
        event_tx: Sender<Event>,
        pools: Arc<DashMap<Address, Pool>>,
    ) -> Self {
        Self {
            block_rx,
            tx_rx,
            event_tx,
            pools,
        }
    }

    pub async fn start_strategy(self, provider: Arc<RootProvider<Http<Client>>>) {
        // block worker
        tokio::task::spawn(StrategyActor::block_worker(self.block_rx));

        // mempool worker
        tokio::task::spawn(StrategyActor::tx_worker(
            self.tx_rx,
            provider,
            self.pools.clone(),
        ));
    }

    pub async fn block_worker(mut block_rx: Receiver<Block>) {
        while let Ok(block) = block_rx.recv().await {
            info!("Got a new block: {:?}", block);
        }
    }

    pub async fn tx_worker(
        mut tx_rx: Receiver<Transaction>,
        provider: Arc<RootProvider<Http<Client>>>,
        pools: Arc<DashMap<Address, Pool>>,
    ) {
        // wait for a new transaction
        while let Ok(tx) = tx_rx.recv().await {
            // trace the transaction
            let request: TransactionRequest = tx.into();
            let trace_types = [TraceType::StateDiff];
            match provider.trace_call(&request, &trace_types).await {
                Ok(trace_result) => {
                    // determine all pools that the transaction touched
                    let touched_pools: Vec<Pool> = trace_result
                        .state_diff
                        .unwrap()
                        .keys()
                        .filter_map(|address| pools.get(address).map(|pool_ref| pool_ref.clone()))
                        .collect();
                    println!("Trace result: {:#?}", touched_pools);

                    // send a mesasge that we have a new touched pool
                }
                Err(e) => println!("Tracing failed: {:?}", e),
            }
        }
    }
}

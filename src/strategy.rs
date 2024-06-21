use std::sync::Arc;
use alloy::providers::RootProvider;
use alloy::pubsub::PubSubFrontend;
use alloy::transports::http::{Http, Client};
use alloy::primitives::TxHash;
use alloy::rpc::types::Block;
use tokio::sync::broadcast::{Sender, Receiver};
use log::info;
use crate::util::Event;



pub async fn run_strategy(
        ws_provider: Arc<RootProvider<PubSubFrontend>>,
        http_provider: Arc<RootProvider<Http<Client>>>,
        mempool_sender: Sender<TxHash>,
        block_sender: Sender<Block>,
        event_sender: Sender<Event>)  
{

        info!("Staring the strategy!");
        todo!()
}
mod client_receipt2queue;
mod receipt_queue_server;
mod receipt2client;

use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::RwLock;

use crate::object::gram::receipt::ReceiptGram;

use crate::queue::{GramBufferPool, BitcommGramQueue};

pub async fn process_receipt_gram<'a>(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<ReceiptGram>,
    stm: &Arc<tokio::sync::Mutex<SendStream>>,
    rct_queue: &Arc<BitcommGramQueue>
) {
    client_receipt2queue::send_client_receipt_to_queue(
        reqmsgbuff,
        reqmsggram,
        stm,
        rct_queue
    ).await;
}

pub async fn start_receipt_queue_server(
    rct_queue: &Arc<BitcommGramQueue>,
    buffer: &Arc<RwLock<GramBufferPool>>
) -> Arc<tokio::task::JoinHandle<()>> {
    let handle = receipt_queue_server::start_receipt_queue_server(rct_queue,buffer).await;
    handle
}

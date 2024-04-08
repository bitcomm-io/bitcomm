mod server_receipt2queue;

use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::Mutex;

use crate::object::gram::receipt::ReceiptGram;

use crate::queue::BitcommGramQueue;

pub async fn process_receipt_gram<'a>(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<ReceiptGram>,
    stm: Arc<Mutex<SendStream>>,
    rct_queue: &Arc<BitcommGramQueue>
) {
    server_receipt2queue::send_server_receipt_to_queue(
        reqmsgbuff,
        reqmsggram,
        stm,
        rct_queue
    ).await;
}
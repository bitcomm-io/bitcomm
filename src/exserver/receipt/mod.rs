mod server_receipt2queue;

use std::sync::Arc;

use bytes::Bytes;

use crate::object::gram::receipt::ReceiptGram;

use crate::queue::BitcommGramQueue;

pub async fn process_receipt_gram<'a>(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<ReceiptGram>,
    rct_queue: &Arc<BitcommGramQueue>
) {
    server_receipt2queue::send_server_receipt_to_queue(
        reqmsgbuff,
        reqmsggram,
        rct_queue
    ).await;
}
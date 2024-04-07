use std::sync::Arc;

use bytes::Bytes;
use tracing::info;

use crate::{queue::{BitcommGramQueue, GramEvent}, object::gram::receipt::ReceiptGram};


// use tracing::error;
#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_server_receipt_to_queue (
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<ReceiptGram>,
    rct_queue:&Arc<BitcommGramQueue>,
) -> Option<Arc<ReceiptGram>> {
    //
    // eprintln!("client send message buf  to server {:?}", reqmsgbuff);
    info!("client send message gram to server {:?}", data_gram);
    //
    // let mut ccp = cpm.lock().await;
    //
    let clt: u64 = data_gram.sender().into();
    let dev: u32 = data_gram.device_id();

    send_receipt_to_queue(rct_queue,data_buff, data_gram).await;

    Option::Some(data_gram.clone())
}
//
async fn send_receipt_to_queue(rct_queue:&Arc<BitcommGramQueue>,data_buff: &Arc<Bytes>, data_gram: &Arc<ReceiptGram>) {
    let sender = rct_queue.get_sender();
    let reptevent = sender.lock().await;
    reptevent
        .send(GramEvent::ReceiptGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        })
        .await
        .expect("send event error!");
}

use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::Mutex;
// use s2n_quic::stream::SendStream;
// use tokio::{ io::AsyncWriteExt, sync::Mutex };
use tracing::info;

use crate::{ queue::{ BitcommGramQueue, GramEvent }, object::gram::message::MessageGram };

// use tracing::error;
#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_server_message_to_queue(
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>,
    stm: Arc<Mutex<SendStream>>,
    queue: &Arc<BitcommGramQueue>
) -> Option<Arc<MessageGram>> {
    //
    info!("client send message gram to server {:?}", data_gram);
    // let clt: u64 = data_gram.sender().into();
    // let dev: u32 = data_gram.device_id();

    send_message_to_queue(data_buff, data_gram, queue).await;
    Some(data_gram.clone())
}
//
async fn send_message_to_queue(
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>,
    queue: &Arc<BitcommGramQueue>
) {
    let sender = queue.get_sender();
    let msgevent = sender.lock().await;
    msgevent
        .send(GramEvent::MessagGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}
//

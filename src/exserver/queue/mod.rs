pub mod buffer;

use std::sync::Arc;

use bytes::Bytes;
// use lazy_static::lazy_static;
use tokio::{ io::AsyncWriteExt, sync::RwLock };

use crate::{
    object::gram::{ message::MessageGram, receipt::ReceiptGram },
    queue::{ BitcommGramQueue, GramBufferPool, GramEvent },
};

use super::get_s2s_msp;
//
// lazy_static! {
/// 全局消息通道,用于处理客户端发送来的所有消息
// pub static ref EXCHANGE_MESSAGE_CHANNEL : Arc<BitcommGramQueue>  =  {
//     Arc::new(BitcommGramQueue::new(EVENT_QUEUE_LEN * 5))
// };
// // 缓冲池
// pub static ref RESEND_BUFFER_2_SERVER : Arc<RwLock<GramBufferPool>> = {
//     Arc::new(RwLock::new(GramBufferPool::new(EVENT_QUEUE_LEN * 5)))
// };
// static ref resend_buffer: Arc::new(RwLock::new(GramBufferPool::new(queue_size))),
// }

//
pub async fn start_exchange_message_queue_server(
    message_queue: Arc<BitcommGramQueue>,
    message_buffer: Arc<RwLock<GramBufferPool>>
) -> Arc<tokio::task::JoinHandle<()>> {
    // 获取消息事件队列的接收端
    let handle = tokio::spawn(async move {
        let receiver = message_queue.get_receiver();
        let mut meqrece = receiver.lock().await;
        while let Some(event) = meqrece.recv().await {
            match event {
                // 处理消息接收事件
                GramEvent::MessagGramEvent { data_buff, data_gram } => {
                    buffer::send_message2buffer(&message_buffer, &data_buff, &data_gram).await;
                    send_message2server(&data_buff, &data_gram).await;
                }
                // 处理群组消息接收事件
                GramEvent::ReceiptGramEvent { data_buff, data_gram } => {
                    buffer::send_receipt2buffer(&message_buffer, &data_buff, &data_gram).await;
                    send_receipt2server(&data_buff, &data_gram).await;
                }
                _ => {} // 忽略其他类型的事件
            }
        }
    });
    Arc::new(handle)
}

//
async fn send_message2server(data_buff: &Arc<Bytes>, data_gram: &Arc<MessageGram>) {
    let client = data_gram.receiver();
    if let Some(server) = get_s2s_msp(client.planet()).await {
        let stm = server.read().await;
        let stm = stm.send_stream();
        let mut send_stream = stm.lock().await;
        send_stream.write(data_buff).await.expect("send to server error!");
        send_stream.flush().await.expect("flush error!");
    } else {
        // TODO: 增加找不到服务器的处理
    }
}
//
async fn send_receipt2server(data_buff: &Arc<Bytes>, data_gram: &Arc<ReceiptGram>) {
    let client = data_gram.receiver();
    if let Some(server) = get_s2s_msp(client.planet()).await {
        let stm = server.read().await;
        let stm = stm.send_stream();
        let mut send_stream = stm.lock().await;
        send_stream.write(data_buff).await.expect("send to server error!");
        send_stream.flush().await.expect("flush error!");
    } else {
        // TODO:增加找不到服务器的处理
    }
}

//
pub async fn put_data_buff_to_queue(message_queue: Arc<BitcommGramQueue>, data_buff: &Arc<Bytes>) {
    if MessageGram::is_message(data_buff) {
        let data_gram = Arc::new(*MessageGram::get_message_gram_by_u8(data_buff));
        return put_message_gram_to_queue(message_queue, data_buff, &data_gram).await;
    }
    if ReceiptGram::is_receipt(data_buff) {
        let data_gram = Arc::new(*ReceiptGram::get_receipt_gram_by_u8(data_buff));
        return put_receipt_gram_to_queue(message_queue, data_buff, &data_gram).await;
    }
}
//
pub async fn put_receipt_gram_to_queue(
    message_queue: Arc<BitcommGramQueue>,
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<ReceiptGram>
) {
    let sender = message_queue.get_sender();
    let msgevent = sender.lock().await;
    msgevent
        .send(GramEvent::ReceiptGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}

pub async fn put_message_gram_to_queue(
    message_queue: Arc<BitcommGramQueue>,
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>
) {
    let sender = message_queue.get_sender();
    let msgevent = sender.lock().await;
    msgevent
        .send(GramEvent::MessagGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}

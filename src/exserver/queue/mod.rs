pub mod buffer;


use std::{error::Error, sync::Arc};

use bytes::Bytes;
use lazy_static::lazy_static;
use tokio::{io::AsyncWriteExt, sync::RwLock};

use crate::{object::gram::{message::MessageGram, receipt::ReceiptGram}, queue::{BitcommGramQueue, GramBufferPool, GramEvent, EVENT_QUEUE_LEN}};

use super::get_s2s_msp;
//
lazy_static! {  
    /// 全局消息通道,用于处理客户端发送来的所有消息
    pub static ref EXCHANGE_MESSAGE_CHANNEL : Arc<BitcommGramQueue>  =  {
        Arc::new(BitcommGramQueue::new(EVENT_QUEUE_LEN * 5))
    };
    // 缓冲池
    pub static ref RESEND_BUFFER_2_SERVER : Arc<RwLock<GramBufferPool>> = {
        Arc::new(RwLock::new(GramBufferPool::new(EVENT_QUEUE_LEN * 5)))
    };
    // static ref resend_buffer: Arc::new(RwLock::new(GramBufferPool::new(queue_size))),
}
// 
pub async fn put_resend_data_queue(data_buff: &Arc<Bytes>,) {
    if MessageGram::is_message(data_buff) {
        let data_gram = Arc::new(*MessageGram::get_message_gram_by_u8(data_buff));
        return put_resend_message_queue(data_buff,&data_gram).await;
    }
    if ReceiptGram::is_receipt(data_buff) {
        let data_gram = Arc::new(*ReceiptGram::get_receipt_gram_by_u8(data_buff));
        return put_resend_receipt_queue(data_buff,&data_gram).await;
    }
}
// 
pub async fn put_resend_receipt_queue(
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<ReceiptGram>,
) {
    let sender = EXCHANGE_MESSAGE_CHANNEL.get_sender();
    let msgevent = sender.lock().await;
    msgevent
        .send(GramEvent::ReceiptGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}

pub async fn put_resend_message_queue(
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>,
) {
    let sender = EXCHANGE_MESSAGE_CHANNEL.get_sender();
    let msgevent = sender.lock().await;
    msgevent
        .send(GramEvent::MessagGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}

pub async fn start_exchange_message_queue_server() -> Result<(), Box<dyn Error>> {
    // 获取消息事件队列的接收端
    let receiver = EXCHANGE_MESSAGE_CHANNEL.get_receiver();
    let mut meqrece = receiver.lock().await;
    while let Some(event) = meqrece.recv().await {
        match event {
            // 处理消息接收事件
            GramEvent::MessagGramEvent { data_buff, data_gram } => {
                send_message2buffer(&data_buff,&data_gram).await;
                send_message2server(&data_buff,&data_gram).await;
            }
            // 处理群组消息接收事件
            GramEvent::ReceiptGramEvent { data_buff, data_gram } => {
                send_receipt2buffer(&data_buff,&data_gram).await;
                send_receipt2server(&data_buff,&data_gram).await;
            }
            _ => {} // 忽略其他类型的事件
        }
    }
    Ok(())
}
//
async fn send_message2buffer(data_buff: &Arc<Bytes>, data_gram: &Arc<MessageGram>) {
    let mut buffer = RESEND_BUFFER_2_SERVER.write().await;
    buffer.push(data_buff.clone(), data_gram.get_message_gram_key());
}
//
async fn send_message2server(data_buff:&Arc<Bytes>,data_gram:&Arc<MessageGram>) {
    let client = data_gram.receiver();
    if let Some(server) = get_s2s_msp(client.planet()).await {
        let mut send_stream = server.send_stream().lock().await;
        send_stream.write(data_buff).await.expect("send to server error!");
        send_stream.flush().await.expect("flush error!");
    } else {
        // TODO: 增加找不到服务器的处理
    }
}
//
async fn send_receipt2buffer(data_buff: &Arc<Bytes>, data_gram: &Arc<ReceiptGram>) {
    let mut buffer = RESEND_BUFFER_2_SERVER.write().await;
    buffer.push(data_buff.clone(), data_gram.get_receipt_gram_key());
}
//
async fn send_receipt2server(data_buff:&Arc<Bytes>,data_gram:&Arc<ReceiptGram>) {
    let client = data_gram.receiver();
    if let Some(server) = get_s2s_msp(client.planet()).await {
        let mut send_stream = server.send_stream().lock().await;
        send_stream.write(data_buff).await.expect("send to server error!");
        send_stream.flush().await.expect("flush error!");
    } else {
        // TODO:增加找不到服务器的处理
    }
}



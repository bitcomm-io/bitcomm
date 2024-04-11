pub mod buffer;

use std::sync::Arc;

use bytes::Bytes;
// use lazy_static::lazy_static;
use tokio::{ io::AsyncWriteExt, sync::RwLock };

use crate::{
    object::gram::{ message::MessageGram, receipt::ReceiptGram },
    queue::{ BitcommGramQueue, GramBufferPool, GramEvent },
};

use super::EXServer;

//
pub async fn start_exserver_send_server(
    exserver:&Arc<RwLock<EXServer>>,
) -> Arc<RwLock<EXServer>> {
    let exserver = exserver.clone();
    let message_queue = get_message_queue(&exserver).await;
    let message_buffer = get_message_buffer(&exserver).await;
    let mq = message_queue.clone();
    let mb = message_buffer.clone();
    let rtserver = exserver.clone();
    // 获取消息事件队列的接收端
    let queue_handle = tokio::spawn(async move {
        let receiver = message_queue.get_receiver();
        let mut meqrece = receiver.write().await;
        while let Some(event) = meqrece.recv().await {
            match event {
                // 处理消息接收事件
                GramEvent::MessageGramEvent { data_buff, data_gram } => {
                    buffer::send_message2buffer(&message_buffer, &data_buff, &data_gram).await;
                    send_message2server(&exserver,&data_buff, &data_gram).await;
                }
                // 处理群组消息接收事件
                GramEvent::ReceiptGramEvent { data_buff, data_gram } => {
                    buffer::send_receipt2buffer(&message_buffer, &data_buff, &data_gram).await;
                    send_receipt2server(&exserver,&data_buff, &data_gram).await;
                }
                _ => {} // 忽略其他类型的事件
            }
        }
    });
    let buffer_handle = buffer::start_resend_buffer_server(mq,mb).await;
    let mut set_server = rtserver.write().await;
    set_server.set_sng_queue_task(Some(Arc::new(queue_handle)));
    set_server.set_rsd_buffer_task(Some(buffer_handle));
    rtserver.clone()
}
async fn get_message_queue(exserver:&Arc<RwLock<EXServer>>,) -> Arc<BitcommGramQueue> {
    let exserver = exserver.read().await;
    exserver.ims_msg_queue().clone()
}
async fn get_message_buffer(exserver:&Arc<RwLock<EXServer>>,) -> Arc<RwLock<GramBufferPool>> {
    let exserver = exserver.read().await;
    exserver.resend_buffer().clone()
}
//
async fn send_message2server(exserver:&Arc<RwLock<EXServer>>,data_buff: &Arc<Bytes>, _data_gram: &Arc<MessageGram>) {
    // let client = data_gram.receiver();
    // if let Some(server) = get_s2s_msp(client.planet()).await {
        let exserver = exserver.read().await;
        let stm = exserver.send_stream();
        let mut send_stream = stm.lock().await;
        send_stream.write(data_buff).await.expect("send to server error!");
        send_stream.flush().await.expect("flush error!");
    // } else {
        // TODO: 增加找不到服务器的处理
    // }
}
//
async fn send_receipt2server(exserver:&Arc<RwLock<EXServer>>,data_buff: &Arc<Bytes>, _data_gram: &Arc<ReceiptGram>) {
    // let client = data_gram.receiver();
    // if let Some(server) = get_s2s_msp(client.planet()).await {
        let exserver = exserver.read().await;
        let send_stream = exserver.send_stream();
        let mut send_stream = send_stream.lock().await;
        send_stream.write(data_buff).await.expect("send to server error!");
        send_stream.flush().await.expect("flush error!");
    // } else {
        // TODO:增加找不到服务器的处理
    // }
}
// 
pub async fn put_data_buffer_to_exserver(exserver:&Arc<RwLock<EXServer>>,data_buff: &Arc<Bytes>,) {
    let message_queue = get_message_queue(exserver).await;
    put_data_buff_to_queue(&message_queue,data_buff).await
}
//
pub async fn put_data_buff_to_queue(message_queue: &Arc<BitcommGramQueue>, data_buff: &Arc<Bytes>) {
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
    message_queue: &Arc<BitcommGramQueue>,
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<ReceiptGram>
) {
    let sender = message_queue.get_sender().clone();
    // let msgevent = sender.lock().await;
    sender
        .send(GramEvent::ReceiptGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}

pub async fn put_message_gram_to_queue(
    message_queue: &Arc<BitcommGramQueue>,
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>
) {
    let sender = message_queue.get_sender().clone();
    // let msgevent = sender.lock().await;
    sender
        .send(GramEvent::MessageGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}

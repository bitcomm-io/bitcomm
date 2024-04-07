use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
// use tokio::{ io::AsyncWriteExt, sync::Mutex };
// use crate::{ connservice::ClientPoolManager, eventqueue::{ MessageEvent, MessageEventQueue } };
use tracing::info;

use crate::{client::ClientPoolManager, queue::{BitcommGramQueue, GramEvent}, object::gram::receipt::ReceiptGram};


// use tracing::error;
#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_client_receipt_to_queue (
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<ReceiptGram>,
    stm: Arc<tokio::sync::Mutex<SendStream>>,
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
    // 如果能够获取stream
    if let Some(ostm) = ClientPoolManager::get_client(clt, dev).await {
        // 并且pool里的和传递过来的是同一个stream
        if Arc::ptr_eq(&ostm, &stm) {
            // 更新client的超时信息
            ClientPoolManager::update_client(clt, dev).await;
            // 回复已发送信息
            // send_message_receipt(reqmsggram, stm).await;
            // 将消息发送到事件队列中
            send_receipt_to_queue(rct_queue,data_buff, data_gram).await;
        } //else {
          // 出错,返回出错的回执信息(两个连接不一致)
          // send_message_twosession_nosame(reqmsggram, stm).await;
          // }
    } //else {
      // 出错,返回出错的回执信息(client_pool中没有连接,说明没有登录或是已超时被删除)
      // send_message_notlogin(reqmsggram, stm).await;
      // }
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

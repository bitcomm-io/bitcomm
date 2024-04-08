use std::{ sync::Arc, time::Duration };

use bytes::Bytes;
use tokio::time::sleep;


use crate::object::gram::{message::MessageGram, receipt::ReceiptGram};

use super::{put_data_buff_to_queue, RESEND_BUFFER_2_SERVER};

pub static REMOVE_BUFFER_TIME: u64 = 5;



/// 启动重新发送缓冲服务器的异步函数。
///
/// 这个函数将会返回一个 `Arc<tokio::task::JoinHandle<()>>`，用于控制后台任务的生命周期。
///
/// # Returns
///
/// 返回一个 `Arc<tokio::task::JoinHandle<()>>`，用于控制后台任务的生命周期。
///
pub async fn start_resend_buffer_server() -> Arc<tokio::task::JoinHandle<()>> {
    let server_handle = tokio::spawn(async move {
        loop {
            if get_buffer_size().await > 0 {
                if let Some(data_buff) = pop_message_gram_from_buffer().await {
                    put_data_buff_to_queue(&data_buff).await;
                } else {
                    continue;
                }
            }
            // 先等一会
            sleep(Duration::from_secs(REMOVE_BUFFER_TIME)).await;
            // 取出
        }
    });
    Arc::new(server_handle)
}


//
pub async fn send_message2buffer(data_buff: &Arc<Bytes>, data_gram: &Arc<MessageGram>) {
    let mut buffer = RESEND_BUFFER_2_SERVER.write().await;
    buffer.push(data_buff.clone(), data_gram.get_message_gram_key());
}
//
pub async fn send_receipt2buffer(data_buff: &Arc<Bytes>, data_gram: &Arc<ReceiptGram>) {
    let mut buffer = RESEND_BUFFER_2_SERVER.write().await;
    buffer.push(data_buff.clone(), data_gram.get_receipt_gram_key());
}

/// 从发送消息缓冲区中获取消息对象的异步函数。
///
/// # Returns
///
/// 如果缓冲区中有数据，则返回 `Some(Arc<Bytes>)`，否则返回 `None`。
///
async fn pop_message_gram_from_buffer() -> Option<Arc<Bytes>> {
    let mut buffer = RESEND_BUFFER_2_SERVER.write().await;
    buffer.pop()
}

/// 获取发送消息缓冲区的大小的异步函数。
///
/// # Returns
///
/// 返回发送消息缓冲区的大小。
///
async fn get_buffer_size() -> usize {
    let buffer = RESEND_BUFFER_2_SERVER.read().await;
    buffer.get_buffer_size()
}
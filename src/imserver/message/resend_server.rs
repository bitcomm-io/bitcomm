use std::{ sync::Arc, time::Duration };

use bytes::Bytes;
use tokio::{ sync::RwLock, time::sleep };

use crate::{
    queue::{ GramBufferPool, BitcommGramQueue },
    object::gram::message::MessageGram,
};

use super::{ message_queue_server, REMOVE_BUFFER_TIME };

/// 启动重新发送缓冲服务器的异步函数。
///
/// 这个函数将会返回一个 `Arc<tokio::task::JoinHandle<()>>`，用于控制后台任务的生命周期。
///
/// # Returns
///
/// 返回一个 `Arc<tokio::task::JoinHandle<()>>`，用于控制后台任务的生命周期。
///
pub async fn start_resend_buffer_server(
    snd_queue: &Arc<BitcommGramQueue>,
    buffer: &Arc<RwLock<GramBufferPool>>
) -> Arc<tokio::task::JoinHandle<()>> {
    let snd_queue = snd_queue.clone();
    let buffer = buffer.clone();
    let server_handle = tokio::spawn(async move {
        loop {
            if get_buffer_size(&buffer).await > 0 {
                if let Some(data_buff) = get_message_gram_from_buffer(&buffer).await {
                    let mg = MessageGram::get_message_gram_by_u8(&data_buff);
                    let data_gram = Arc::new(*mg);
                    // 重新放入消息队列
                    message_queue_server::put_send_message_queue(
                        &data_buff,
                        &data_gram,
                        &snd_queue
                    ).await;
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

/// 从发送消息缓冲区中获取消息对象的异步函数。
///
/// # Returns
///
/// 如果缓冲区中有数据，则返回 `Some(Arc<Bytes>)`，否则返回 `None`。
///
async fn get_message_gram_from_buffer(buffer: &Arc<RwLock<GramBufferPool>>) -> Option<Arc<Bytes>> {
    let mut buffer = buffer.write().await;
    buffer.pop()
}

/// 获取发送消息缓冲区的大小的异步函数。
///
/// # Returns
///
/// 返回发送消息缓冲区的大小。
///
async fn get_buffer_size(buffer: &Arc<RwLock<GramBufferPool>>) -> usize {
    let buffer = buffer.read().await;
    buffer.get_buffer_size()
}

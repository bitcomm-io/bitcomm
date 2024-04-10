use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{ queue::{ GramBufferPool, BitcommGramQueue, GramEvent }, object::gram::receipt::ReceiptGram };

use super::receipt2client;

/// 启动消息事件队列服务器。
///
/// 该函数从消息事件队列中获取事件并处理，根据事件类型分发到相应的处理函数中。
///
/// # 返回
/// 如果启动成功，则返回 `Ok(())`，否则返回包含错误信息的 `Result`。
#[allow(unused_variables)]
pub async fn start_receipt_queue_server(
    rct_queue: &Arc<BitcommGramQueue>,
    buffer: &Arc<RwLock<GramBufferPool>>
) -> Arc<tokio::task::JoinHandle<()>> {
    let rct_queue = rct_queue.clone();
    let buffer = buffer.clone();
    // 获取消息事件队列的接收端
    let server_handle = tokio::spawn(async move {
        let receiver = rct_queue.get_receiver();
        let mut meqrece = receiver.write().await;
        while let Some(event) = meqrece.recv().await {
            match event {
                // 处理消息接收事件
                GramEvent::ReceiptGramEvent { data_buff, data_gram } => {
                    // 从正在发送缓冲池中删除
                    remove_buffer(&buffer,&data_gram).await;
                    // 发送到客户端
                    receipt2client::send_receipt_to_client(&data_buff, &data_gram).await;
                }
                // // 处理群组消息接收事件
                // MessageEvent::GroupReceive { reqmsgbuff, reqmsggram } => {
                //     grpmessage::send_group_message_to_all(&reqmsgbuff, &reqmsggram).await;
                // }
                // // 处理服务消息接收事件
                // MessageEvent::ServiceReceive { reqmsgbuff, reqmsggram } => {
                //     // 可根据需要添加处理逻辑
                // }
                _ => {} // 忽略其他类型的事件
            }
        }
    });
    Arc::new(server_handle)
}
// 
async fn remove_buffer(buffer: &Arc<RwLock<GramBufferPool>>,data_gram: &Arc<ReceiptGram>) {
    let key = data_gram.get_receipt_gram_key();
    let mut buffer = buffer.write().await;
    buffer.remove(&key);
}

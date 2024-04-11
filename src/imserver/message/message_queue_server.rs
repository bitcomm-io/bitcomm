use std::sync::Arc;

use bytes::Bytes;

use crate::{ queue::{ BitcommGramQueue, GramEvent }, object::gram::message::MessageGram };

/// 启动消息事件队列服务器。
///
/// 该函数从消息事件队列中获取事件并处理，根据事件类型分发到相应的处理函数中。
///
/// # 返回
/// 如果启动成功，则返回 `Ok(())`，否则返回包含错误信息的 `Result`。
#[allow(unused_variables)]
pub async fn start_message_queue_server(
    msg_queue: &Arc<BitcommGramQueue>,
    snd_queue: &Arc<BitcommGramQueue>
) -> Arc<tokio::task::JoinHandle<()>> {
    let msg_queue = msg_queue.clone();
    let snd_queue = snd_queue.clone();
    let server_handle = tokio::spawn(async move {
        let receiver = msg_queue.get_receiver();
        let mut meqrece = receiver.write().await;
        while let Some(event) = meqrece.recv().await {
            match event {
                // 处理消息接收事件
                GramEvent::MessageGramEvent { data_buff, data_gram } => {
                    // 存入收件箱
                    // save_to_inbox();
                    // 判断服务器类型，如果不是同一个服务器，则需要转发
                    // 接收者在线状态，如果离线，则存储离线收件箱

                    // sendtoclient::send_message_to_client(&reqmsgbuff, &reqmsggram).await;
                    // 如果是在线状态，则放入发送队列
                    put_send_message_queue(&data_buff, &data_gram, &snd_queue).await;
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
pub async fn put_send_message_queue(
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>,
    snd_queue: &Arc<BitcommGramQueue>
) {
    let sender = snd_queue.get_sender();
    let msgevent = sender.clone();
    msgevent
        .send(GramEvent::MessageGramEvent {
            data_buff: data_buff.clone(),
            data_gram: data_gram.clone(),
        }).await
        .expect("send event error!");
}

// use std:: error::Error ;

// use crate::{
//     eventqueue::{MessageEvent, MessageEventQueue},
//     group::grpmessage,
//     sendtoclient,
// };

// #[allow(unused_variables)]
// pub async fn start_message_event_queue_server() -> Result<(), Box<dyn Error>> {
//     let receiver = MessageEventQueue::get_receiver();
//     let mut meqrece = receiver.lock().await;//MESSAGE_CHANNEL.1.lock().await;
//     while let Some(event) = meqrece.recv().await {
//         match event {
//             MessageEvent::MessageReceive { reqmsgbuff, reqmsggram } => {
//                 sendtoclient::send_message_to_client( &reqmsgbuff, &reqmsggram).await;
//             }
//             MessageEvent::GroupReceive { reqmsgbuff, reqmsggram } => {
//                 // 处理 GroupReceive 事件
//                 grpmessage::send_group_message_to_all( &reqmsgbuff, &reqmsggram).await;
//             }
//             MessageEvent::ServiceReceive { reqmsgbuff, reqmsggram } => {
//                 // 处理 ServiceReceive 事件
//             }
//             _ => {}
//         }
//     }
//     Ok(())
// }

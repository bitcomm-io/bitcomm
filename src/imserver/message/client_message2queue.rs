use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::{ io::AsyncWriteExt, sync::Mutex };
use tracing::info;

use crate::{
    client::ClientPoolManager,
    queue::{ BitcommGramQueue, GramEvent },
    object::gram::{ command::CommandGram, message::MessageGram, ReturnCode },
};

// use tracing::error;
#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_client_message_to_queue(
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>,
    stm: Arc<Mutex<SendStream>>,
    queue: &Arc<BitcommGramQueue>
) -> Option<Arc<MessageGram>> {
    //
    info!("client send message gram to server {:?}", data_gram);
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
            send_message_to_queue(data_buff, data_gram, queue).await;
        } else {
            // 出错,返回出错的回执信息(两个连接不一致)
            send_message_twosession_nosame(data_gram, stm).await;
        }
    } else {
        // 出错,返回出错的回执信息(client_pool中没有连接,说明没有登录或是已超时被删除)
        send_message_notlogin(data_gram, stm).await;
    }
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
async fn send_message_notlogin(reqmsggram: &Arc<MessageGram>, stm: Arc<Mutex<SendStream>>) {
    let mut vecu8 = CommandGram::create_gram_buf(0);
    let cdg = CommandGram::create_command_gram_from_message_gram(
        vecu8.as_mut(),
        reqmsggram.as_ref()
    );
    cdg.set_return_code(ReturnCode::RETURN_NOT_LOGIN);

    let mut stream = stm.lock().await;
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
//
async fn send_message_twosession_nosame(
    reqmsggram: &Arc<MessageGram>,
    stm: Arc<Mutex<SendStream>>
) {
    let mut vecu8 = CommandGram::create_gram_buf(0);
    let cdg = CommandGram::create_command_gram_from_message_gram(
        vecu8.as_mut(),
        reqmsggram.as_ref()
    );
    cdg.set_return_code(ReturnCode::RETURN_TWO_SESSION_NO_SAME);

    let mut stream = stm.lock().await;
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}
//
async fn _send_message_receipt(reqmsggram: &Arc<MessageGram>, stm: Arc<Mutex<SendStream>>) {
    let mut vecu8 = CommandGram::create_gram_buf(0);
    let cdg = CommandGram::create_command_gram_from_message_gram(
        vecu8.as_mut(),
        reqmsggram.as_ref()
    );
    cdg.set_return_code(ReturnCode::RETURN_OK);

    let mut stream = stm.lock().await;
    stream.write_all(vecu8.as_mut()).await.expect("stream should be open");
    stream.flush().await.expect("stream should be open");
}

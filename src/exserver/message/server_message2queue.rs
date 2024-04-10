use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::{io::AsyncWriteExt, sync::Mutex};
// use s2n_quic::stream::SendStream;
// use tokio::{ io::AsyncWriteExt, sync::Mutex };
use tracing::info;

use crate::{ object::gram::{message::MessageGram, reply::ReplyGram, BitCommand, BitcommFlag}, queue::{ BitcommGramQueue, GramEvent } };

// use tracing::error;
#[allow(unused_variables)]
// #[allow(dead_code)]
pub async fn send_server_message_to_queue(
    data_buff: &Arc<Bytes>,
    data_gram: &Arc<MessageGram>,
    stm: &Arc<Mutex<SendStream>>,
    queue: &Arc<BitcommGramQueue>
) {
    //
    info!("client send message gram to server {:?}", data_gram);
    // 
    send_message_to_queue(data_buff, data_gram, queue).await;
    // send reply receipt
    send_server_message_reply(data_gram,stm).await;
    // Some(data_gram.clone())
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
async fn send_server_message_reply(data_gram: &Arc<MessageGram>, stm: &Arc<Mutex<SendStream>>) {
    // 如果当前接收是在ListenServer中，则需要发送反馈信息
    let rct_buff = get_reply_buff(data_gram);
    // 直接发送
    let mut stream = stm.lock().await;
    stream.write(&rct_buff).await.expect("send to error!");
    stream.flush().await.expect("flush error");
}

fn get_reply_buff(data_gram: &Arc<MessageGram>) -> Vec<u8> {
    let mut rct_buff:Vec<u8> = ReplyGram::create_gram_buf(0);
    let rct_gram = ReplyGram::create_reply_gram_by_mut_vec8(&mut rct_buff);
    rct_gram.set_bitcomm(BitcommFlag::BITCOMM_REPLY);
    rct_gram.set_version(data_gram.version());
    rct_gram.set_command(data_gram.command() | BitCommand::RESP_MASK);
    rct_gram.set_device_id(data_gram.device_id());
    rct_gram.set_sender(data_gram.sender());
    rct_gram.set_receiver(data_gram.receiver());
    rct_gram.set_message_type(data_gram.message_type());
    rct_gram.set_message_id(data_gram.message_id());
    // rct_gram.set_data_time(data_gram.data_time());
    // FIXME: 需要修改为合适的时间
    rct_buff
}
use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::{ ReceiveStream, SendStream };
use tokio::sync::Mutex;
use tracing::{ error, info };

use crate::queue::BitcommGramQueue;
use crate::imserver::{ command, message, receipt };
// use crate::net::proc;
use crate::object::gram::command::CommandGram;
use crate::object::gram::message::MessageGram;
use crate::object::gram::receipt::ReceiptGram;
use crate::object::gram::BitcommDataGram;

//
pub async fn receive_data_gram(
    mut receive_stream: ReceiveStream,
    stm0: &Arc<Mutex<SendStream>>,
    msg_queue: &Arc<BitcommGramQueue>,
    rct_queue: &Arc<BitcommGramQueue>
) {
    // 接收数据并处理
    while let Ok(Some(data_buff)) = receive_stream.receive().await {
        let rc_data_buff = Arc::new(data_buff);
        // 准备数据缓冲区
        if let Some(inner_data_gram) = prepare_data_buffer(rc_data_buff.clone()) {
            let rc_data_gram = Arc::new(inner_data_gram);
            // 发送回执，服务器已收到，正在处理
            send_receipt_2_client(&rc_data_gram, &stm0).await;
            // 处理命令
            process_bitcomm_command(&rc_data_gram, &stm0).await;
            // 处理消息
            process_bitcomm_gram(&rc_data_gram, &stm0, msg_queue, rct_queue).await;
        } else {
            // 获取发送流的互斥锁并发送数据
            let mut send_stream = stm0.lock().await;
            // 记录客户端主机信息和接收到的数据
            info!("client host from {:?}", send_stream.connection().remote_addr());
            info!("receive data is  {:?}", rc_data_buff.as_ref());
            // 发送数据并处理错误
            if let Err(err) = send_stream.send(Arc::try_unwrap(rc_data_buff).unwrap()).await {
                error!("send error: {}", err);
            }
        }
    }
}

pub fn prepare_data_buffer(data_buff: Arc<Bytes>) -> Option<BitcommDataGram> {
    let bts = data_buff.as_ref();

    if CommandGram::is_pingpong(bts) {
        let bitcomm_flag = CommandGram::get_bitcomm_flag_by_u8(bts);
        return Some(BitcommDataGram::Pingpong(Arc::new(*bitcomm_flag)));
    }

    if CommandGram::is_command(bts) {
        let command_gram = CommandGram::get_command_gram_by_u8(bts);
        return Some(BitcommDataGram::Command {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*command_gram),
        });
    }
    if ReceiptGram::is_receipt(bts) {
        let receipt_gram = ReceiptGram::get_receipt_gram_by_u8(bts);
        return Some(BitcommDataGram::Receipt {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*receipt_gram),
        });
    }
    if MessageGram::is_message(bts) {
        let message_gram = MessageGram::get_message_gram_by_u8(bts);
        return Some(BitcommDataGram::Message {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*message_gram),
        });
    }
    None
}

#[allow(unused_variables)]
pub async fn send_receipt_2_client(data: &Arc<BitcommDataGram>, _stm: &Arc<Mutex<SendStream>>) {
    match data.as_ref() {
        // 只有收到的是message类型的报文才发送
        BitcommDataGram::Message { data_buff, data_gram } => {}
        _ => {}
    }
}
#[allow(unused_variables)]
pub async fn process_bitcomm_command(data: &Arc<BitcommDataGram>, stm: &Arc<Mutex<SendStream>>) {
    match data.as_ref() {
        // 处理心跳消息
        BitcommDataGram::Pingpong(pingpong) => command::process_pingpong(pingpong, stm).await,
        // 处理命令
        BitcommDataGram::Command { data_buff, data_gram } =>
            command::process_command_data(data_buff, data_gram, stm).await,
        // 应答信息包
        BitcommDataGram::Reply { data_buff, data_gram } => {}
        _ => {}
    }
    // Ok(data)
}

#[allow(unused_variables)]
pub async fn process_bitcomm_gram(
    data: &Arc<BitcommDataGram>,
    stm: &Arc<Mutex<SendStream>>,
    msg_queue: &Arc<BitcommGramQueue>,
    rct_queue: &Arc<BitcommGramQueue>
) {
    match data.as_ref() {
        BitcommDataGram::Message { data_buff, data_gram } =>
            message::process_message_gram(data_buff, data_gram, stm, msg_queue).await,
        BitcommDataGram::Receipt { data_buff, data_gram } =>
            receipt::process_receipt_gram(data_buff, data_gram, stm, rct_queue).await,
        //
        _ => {}
    }
    // Ok(data)
}

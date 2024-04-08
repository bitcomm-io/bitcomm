use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::{ ReceiveStream, SendStream };
use tokio::sync::Mutex;
use tracing::error;

use crate::object::gram::hookup::S2SHookupGram;
use crate::object::gram::reply::ReplyGram;
use crate::queue::BitcommGramQueue;
use crate::exserver::{ hookup, message, receipt, reply, S2SMSPType };
use crate::object::gram::message::MessageGram;
use crate::object::gram::receipt::ReceiptGram;
use crate::object::gram::{ BitcommDataGram, DataGramError };

//
pub async fn receive_data_gram(
    mut receive_stream: ReceiveStream,
    stm0: Arc<Mutex<SendStream>>,
    msg_queue: Arc<BitcommGramQueue>,
    rct_queue: Arc<BitcommGramQueue>,
    s2smsp_type: S2SMSPType
) {
    // 接收数据并处理
    while let Ok(Some(data_buff)) = receive_stream.receive().await {
        let rc_data_buff = Arc::new(data_buff);
        // 准备数据缓冲区
        if let Some(inner_data_gram) = prepare_data_buffer(rc_data_buff.clone()) {
            let rc_data_gram = Arc::new(inner_data_gram);
            // 发送回执，服务器已收到，正在处理,如果是返回的信息，则无需再回执
            send_receipt_2_client(rc_data_gram.clone(), stm0.clone()).await;
            // 处理命令
            if
                let Err(err) = process_server_command(
                    rc_data_gram.clone(),
                    stm0.clone(),
                    s2smsp_type.clone()
                ).await
            {
                error!("process command error: {}", err);
            }
            // 处理消息
            if
                let Err(err) = process_bitcomm_gram(
                    rc_data_gram.clone(),
                    stm0.clone(),
                    msg_queue.clone(),
                    rct_queue.clone()
                ).await
            {
                error!("process message error: {}", err);
            }
        }
    }
}
//
pub fn prepare_data_buffer(data_buff: Arc<Bytes>) -> Option<BitcommDataGram> {
    let bts = data_buff.as_ref();

    if ReplyGram::is_reply(bts) {
        let reply_gram = ReplyGram::get_reply_gram_by_u8(bts);
        return Some(BitcommDataGram::Reply {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*reply_gram),
        });
    }
    if S2SHookupGram::is_hookup(bts) {
        let hookup_gram = S2SHookupGram::get_hookup_gram_by_u8(bts);
        return Some(BitcommDataGram::S2SHookup {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*hookup_gram),
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
//
#[allow(unused_variables)]
pub async fn send_receipt_2_client(data: Arc<BitcommDataGram>, _stm: Arc<Mutex<SendStream>>) {
    match data.as_ref() {
        // 只有收到的是message类型的报文才发送
        BitcommDataGram::Message { data_buff, data_gram } => {}
        _ => {}
    }
}
//
#[allow(unused_variables)]
pub async fn process_server_command(
    data: Arc<BitcommDataGram>,
    stm: Arc<Mutex<SendStream>>,
    s2smsp_type: S2SMSPType
) -> Result<Arc<BitcommDataGram>, DataGramError> {
    match data.as_ref() {
        // 处理命令
        BitcommDataGram::S2SHookup { data_buff, data_gram } =>
            hookup::process_hookup(data_buff, data_gram, stm, s2smsp_type).await,
        // 应答信息包
        BitcommDataGram::Reply { data_buff, data_gram } =>
            reply::process_reply(data_buff, data_gram, stm, s2smsp_type).await,
        _ => {}
    }
    Ok(data)
}
//
#[allow(unused_variables)]
pub async fn process_bitcomm_gram(
    data: Arc<BitcommDataGram>,
    stm: Arc<Mutex<SendStream>>,
    msg_queue: Arc<BitcommGramQueue>,
    rct_queue: Arc<BitcommGramQueue>
) -> Result<Arc<BitcommDataGram>, DataGramError> {
    match data.as_ref() {
        BitcommDataGram::Message { data_buff, data_gram } =>
            message::process_message_gram(data_buff, data_gram, stm, &msg_queue).await,
        BitcommDataGram::Receipt { data_buff, data_gram } =>
            receipt::process_receipt_gram(data_buff, data_gram, stm, &rct_queue).await,
        //
        _ => {}
    }
    Ok(data)
}

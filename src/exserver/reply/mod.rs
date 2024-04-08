use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::Mutex;
use tracing::info;

use std::sync::Arc;

use crate::{ exserver::S2SMSPType, object::gram::reply::ReplyGram };

use super::queue::RESEND_BUFFER_2_SERVER;

//
pub async fn process_reply<'a>(
    _data_buff: &Arc<Bytes>,
    data_gram: &Arc<ReplyGram>,
    _stm: Arc<Mutex<SendStream>>,
    _s2smsp_type:S2SMSPType
) {
    // 记录日志
    info!("server hookup server {:?}", data_gram);
    //
    remove_buffer(data_gram).await;
}
//
async fn remove_buffer(data_gram: &Arc<ReplyGram>) {
    let key = data_gram.get_reply_gram_key();
    let mut buffer = RESEND_BUFFER_2_SERVER.write().await;
    buffer.remove(&key);
}

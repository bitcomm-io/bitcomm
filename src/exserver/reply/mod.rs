use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::{Mutex, RwLock};
use tracing::info;

use std::sync::Arc;

use crate::{ exserver::EXServerType, object::gram::reply::ReplyGram, queue::GramBufferPool };

use super::EXServer;


//
pub async fn process_reply<'a>(
    _data_buff: &Arc<Bytes>,
    data_gram: &Arc<ReplyGram>,
    _stm: &Arc<Mutex<SendStream>>,
    _exserver_type:&EXServerType,
    exserver:&Arc<RwLock<EXServer>>,
) {
    // 记录日志
    info!("server hookup server {:?}", data_gram);
    let mb = exserver.write().await;
    let message_buffer = mb.resend_buffer();
    //
    remove_buffer(message_buffer,data_gram).await;
}
//
async fn remove_buffer(message_buffer: &Arc<RwLock<GramBufferPool>>,data_gram: &Arc<ReplyGram>) {
    let key = data_gram.get_reply_gram_key();
    let mut buffer = message_buffer.write().await;
    buffer.remove(&key);
}

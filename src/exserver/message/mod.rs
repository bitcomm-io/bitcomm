pub mod server_message2queue;


use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::Mutex;

use crate::object::gram::message::MessageGram;

use crate::queue::BitcommGramQueue;


pub async fn process_message_gram<'a>(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<MessageGram>,
    stm: Arc<Mutex<SendStream>>,
    queue: &Arc<BitcommGramQueue>
) {
    server_message2queue::send_server_message_to_queue(reqmsgbuff, reqmsggram, stm,queue).await;
}
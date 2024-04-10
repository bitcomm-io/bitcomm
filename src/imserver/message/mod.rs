mod client_message2queue;
mod message_queue_server;
mod send_queue_server;
mod resend_server;
mod message2client;

use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::sync::RwLock;

use crate::object::gram::message::MessageGram;

use crate::queue::{ GramBufferPool, BitcommGramQueue };

pub static REMOVE_BUFFER_TIME: u64 = 5;

pub async fn process_message_gram<'a>(
    reqmsgbuff: &Arc<Bytes>,
    reqmsggram: &Arc<MessageGram>,
    stm: &Arc<tokio::sync::Mutex<SendStream>>,
    queue: &Arc<BitcommGramQueue>
) {
    client_message2queue::send_client_message_to_queue(reqmsgbuff, reqmsggram, stm, queue).await;
}

pub async fn start_message_queue_server(
    msg_queue: &Arc<BitcommGramQueue>,
    snd_queue: &Arc<BitcommGramQueue>
) -> Arc<tokio::task::JoinHandle<()>> {
    let handle = message_queue_server::start_message_queue_server(msg_queue, snd_queue).await;
    handle
}

pub async fn start_send_message_queue_server(
    snd_queue: &Arc<BitcommGramQueue>,
    buffer: &Arc<RwLock<GramBufferPool>>
) -> Arc<tokio::task::JoinHandle<()>> {
    let handle = send_queue_server::start_send_message_queue_server(snd_queue, buffer).await;
    handle
}

pub async fn start_resend_buffer_server(
    snd_queue: &Arc<BitcommGramQueue>,
    buffer: &Arc<RwLock<GramBufferPool>>
) -> Arc<tokio::task::JoinHandle<()>> {
    let handle = resend_server::start_resend_buffer_server(snd_queue,buffer).await;
    handle
}

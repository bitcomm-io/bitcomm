mod listen_server;

use std::sync::Arc;

use crate::queue::BitcommGramQueue;

pub async fn start_client_message_listening_server(
    server: String,
    port: String,
    msg_queue: &Arc<BitcommGramQueue>,
    rct_queue: &Arc<BitcommGramQueue>
) -> Arc<tokio::task::JoinHandle<()>> {
    let handle = listen_server::client_message_listening_server(
        server,
        port,
        msg_queue,
        rct_queue
    ).await;
    handle
}

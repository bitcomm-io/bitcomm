use std::sync::Arc;

use crate::queue::BitcommGramQueue;

pub mod server_listen;



pub async fn server_message_listening_server(
    server_address: String,
    server_port: String,
    msg_queue: Arc<BitcommGramQueue>,
    rct_queue: Arc<BitcommGramQueue>
) -> Arc<tokio::task::JoinHandle<()>> {
    server_listen::server_message_listening_server( server_address, server_port, msg_queue, rct_queue).await
}

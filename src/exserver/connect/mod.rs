use std::sync::Arc;


use tokio::sync::RwLock;

use crate::queue::BitcommGramQueue;

use super::EXServer;

pub mod server_connect;

pub async fn connect_exchange_server(
    server: &str,
    port: &str,
    ims_msg_queue: Arc<BitcommGramQueue>,
    ims_rct_queue: Arc<BitcommGramQueue>
) -> Arc<RwLock<EXServer>> {
    server_connect::connect_exchange_server(
        server,
        port,
        ims_msg_queue,
        ims_rct_queue
    ).await
}

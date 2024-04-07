use std::sync::Arc;

use s2n_quic::Connection;
// use tokio::task::JoinHandle;

use crate::{object::BITServerID, queue::BitcommGramQueue};

pub mod conn2server;

pub async fn connect_exchange_server(
    local_server:BITServerID,
    remote_server:BITServerID,
    server: &str,
    port: &str,
    msg_queue: &Arc<BitcommGramQueue>,
    rct_queue: &Arc<BitcommGramQueue>
) -> Arc<Connection> {
    conn2server::connect_exchange_server(
        local_server,
        remote_server,
        server,
        port,
        msg_queue,
        rct_queue
    ).await
}

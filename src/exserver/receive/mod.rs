use std::sync::Arc;

use s2n_quic::{ stream::{ ReceiveStream, SendStream }, Connection };
use tokio::sync::{ Mutex, RwLock };

use crate::queue::BitcommGramQueue;

use super::{ EXServer, EXServerType };

pub mod server_data;

pub fn init_exserver(
    send_stream: &Arc<Mutex<SendStream>>,
    rece_stream: &Arc<Mutex<ReceiveStream>>,
    connection: &Arc<Connection>,
    exserver_type: &EXServerType,
    local_server_id: u32,
    ims_msg_queue: &Arc<BitcommGramQueue>,
    ims_rct_queue: &Arc<BitcommGramQueue>
) -> EXServer {
    let exserver = EXServer::new(
        exserver_type.clone(),
        local_server_id,
        ims_msg_queue.clone(),
        ims_rct_queue.clone(),
        connection.clone(),
        send_stream.clone(),
        rece_stream.clone()
    );
    exserver
}
//
pub async fn start_exserver_receive_server(
    exserver: &Arc<RwLock<EXServer>>
) -> Arc<RwLock<EXServer>> {
    let mut set_server = exserver.write().await;
    let rece_stream = set_server.rece_stream().clone();
    let send_stream = set_server.send_stream().clone();
    let msg_queue = set_server.ims_msg_queue().clone();
    let rct_queue = set_server.ims_rct_queue().clone();
    let exserver_type = set_server.exserver_type().clone();
    let rc_exserver = exserver.clone();
    // let conn = connection.clone();
    let handle = tokio::spawn(async move {
        server_data::receive_data_gram(
            rece_stream.clone(),
            send_stream.clone(),
            msg_queue.clone(),
            rct_queue.clone(),
            exserver_type,
            rc_exserver
        ).await;
    });
    set_server.set_rece_task(Some(Arc::new(handle)));

    exserver.clone()
}

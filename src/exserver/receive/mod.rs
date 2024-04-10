use std::sync::Arc;

use s2n_quic::{stream::{ReceiveStream, SendStream}, Connection};
use tokio::sync::{Mutex, RwLock};

use crate::queue::BitcommGramQueue;

use super::{EXServer, S2SMSPType};

pub mod server_data;


pub async fn start_exserver_receive_server(
    send_stream: SendStream,
    rece_stream: ReceiveStream,
    connection: Connection,
    local_server_id: u32,
    s2smsp_type: S2SMSPType,
    ims_msg_queue: Arc<BitcommGramQueue>,
    ims_rct_queue: Arc<BitcommGramQueue>
) -> (Arc<Mutex<SendStream>>, Arc<RwLock<EXServer>>) {
    let send_stream = Arc::new(Mutex::new(send_stream));
    let rece_stream = Arc::new(Mutex::new(rece_stream));
    let connection = Arc::new(connection);
    let sstm = send_stream.clone();

    let exserver = EXServer::new(
        s2smsp_type.clone(),
        local_server_id,
        ims_msg_queue.clone(),
        ims_rct_queue.clone(),
        connection.clone(),
        send_stream.clone(),
        rece_stream.clone()
    );

    let arc_exserver = Arc::new(RwLock::new(exserver));
    let set_exserver = arc_exserver.clone();
    let res_exserver = arc_exserver.clone();
    // let conn = connection.clone();
    let handle = tokio::spawn(async move {
        server_data::receive_data_gram(
            rece_stream,
            send_stream,
            ims_msg_queue.clone(),
            ims_rct_queue.clone(),
            s2smsp_type,
            arc_exserver.clone()
        ).await;
    });
    let mut exserver = set_exserver.write().await;
    exserver.set_rece_task(Some(Arc::new(handle)));
    (sstm, res_exserver)
}

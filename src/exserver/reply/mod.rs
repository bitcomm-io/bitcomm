use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::{io::AsyncWriteExt, sync::Mutex};
use tracing::info;

use std::sync::Arc;

use crate::{ exserver::{self, S2SMSPType}, object::gram::{hookup::S2SHookupGram, BitCommand} };

use super::S2SMessageProcess;

pub async fn process_reply<'a>(
    _data_buff: &Arc<Bytes>,
    data_gram: &Arc<S2SHookupGram>,
    stm: Arc<Mutex<SendStream>>,
    s2smsp_type:S2SMSPType
) {
    // 记录日志
    info!("server hookup server {:?}", data_gram);
    // let sm = stm.lock().await;
    // let conn = sm.connection();
    // let _sg = crate::SERVER_GUID.to_le();

    let s2smp = S2SMessageProcess::new(
        s2smsp_type.clone(),
        data_gram.recv_server_id(),
        data_gram.send_server_id(),
        // connection: Arc::new(Mutex::new(stm.lock().await.connection())),
        stm.clone()
    );
    exserver::put_s2s_msp(Arc::new(s2smp)).await;
    // 
    if let S2SMSPType::ListenServer = s2smsp_type {
        // send_hookup_reply(data_gram, stm).await;

    }
}
use bytes::Bytes;
use s2n_quic::stream::SendStream;
use tokio::{io::AsyncWriteExt, sync::Mutex};
use tracing::info;

use std::sync::Arc;

use crate::{ exserver::{self, S2SMSPType}, object::gram::{hookup::S2SHookupGram, BitCommand} };

use super::S2SMessageProcess;

// use super::S2SMessageProcess;

// use super::client::{get_cd_by_key, ClientPoolManager};

pub async fn process_hookup<'a>(
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
        send_hookup_reply(data_gram, stm).await;

    }
}

async fn send_hookup_reply(data_gram: &Arc<S2SHookupGram>, stm: Arc<Mutex<SendStream>>) {
    // 如果当前接收是在ListenServer中，则需要发送反馈信息
    let mut rct_buff:Vec<u8> = S2SHookupGram::create_gram_buf(0);
    let rct_gram = S2SHookupGram::create_hookup_gram_by_mut_vec8(&mut rct_buff);
    rct_gram.set_bitcomm(data_gram.bitcomm());
    rct_gram.set_version(data_gram.version());
    rct_gram.set_command(data_gram.command() | BitCommand::RESP_MASK);
    rct_gram.set_send_server_id(data_gram.recv_server_id());
    rct_gram.set_recv_server_id(data_gram.send_server_id());
    rct_gram.set_message_type(data_gram.message_type());
    rct_gram.set_message_id(data_gram.message_id());
    rct_gram.set_return_code(data_gram.return_code());
    rct_gram.set_data_time(data_gram.data_time());
    // FIXME: 需要修改为合适的时间
        
    // 直接发送
    let mut stream = stm.lock().await;
    stream.write(&rct_buff).await.expect("send to error!");
    stream.flush().await.expect("flush error");
}

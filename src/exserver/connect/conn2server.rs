use std::sync::Arc;
use s2n_quic::{stream::SendStream, Connection};
use tokio::{io::AsyncWriteExt, sync::Mutex};

use crate::{
    exserver::{ receive, S2SMSPType }, net::quic::qcutils, object::{gram::{hookup::S2SHookupGram, BitCommand, BitcommFlag, BitcommVersion}, BITServerID}, queue::BitcommGramQueue
};

pub async fn connect_exchange_server(
    local_server:BITServerID,
    remote_server:BITServerID,
    server: &str,
    port: &str,
    msg_queue: &Arc<BitcommGramQueue>,
    rct_queue: &Arc<BitcommGramQueue>
) -> Arc<Connection> {

    let mut connection = qcutils::get_client(server, port).await.unwrap(); //client.connect(connect).await?;

    // 打开一个新的双向流，并将其拆分为接收和发送两个部分
    let stream = connection.open_bidirectional_stream().await.unwrap();
    //
    let (receive_stream, send_stream) = stream.split();

    let stm = Arc::new(Mutex::new(send_stream));

    let msg_queue = msg_queue.clone();
    let rct_queue = rct_queue.clone();
    let connection = Arc::new(connection);
    let sstm = stm.clone();
    //
    let conn = connection.clone();
    let _handle = tokio::spawn(async move {
        receive::server_data::receive_data_gram(
            receive_stream,
            stm.clone(),
            msg_queue,
            rct_queue,
            S2SMSPType::ConnectServer
        ).await;
        conn.close(MY_ERROR_CODE.into())
        // rece_data(rs).await;
    });
    send_hookup(local_server,remote_server,sstm).await;
    // 需要发送Hookup消息
    connection
    //Arc::new(connection) // 成功返回 Ok(())
}
const MY_ERROR_CODE:u32 = 99;

async fn send_hookup(
    local_server:BITServerID,
    remote_server:BITServerID,
    stm: Arc<Mutex<SendStream>>) {
    // 如果当前接收是在ListenServer中，则需要发送反馈信息
    let mut rct_buff:Vec<u8> = S2SHookupGram::create_gram_buf(0);
    let rct_gram = S2SHookupGram::create_hookup_gram_by_mut_vec8(&mut rct_buff);
    rct_gram.set_bitcomm(BitcommFlag::BITCOMM_HOOKUP);
    rct_gram.set_version(BitcommVersion::BITCOMM_VERSION_0_1_0_1);
    rct_gram.set_command(BitCommand::HOOKUP_SERVER);
    rct_gram.set_send_server_id(local_server);
    rct_gram.set_recv_server_id(remote_server);
    // rct_gram.set_message_type();
    // rct_gram.set_message_id(0);
    // rct_gram.set_return_code(0);
    // rct_gram.set_data_time(0);
    // FIXME: 需要修改为合适的时间
        
    // 直接发送
    let mut stream = stm.lock().await;
    stream.write(&rct_buff).await.expect("send to error!");
    stream.flush().await.expect("flush error");
}

use std::sync::Arc;
use s2n_quic::stream::SendStream;
use tokio::{ io::AsyncWriteExt, sync::{ Mutex, RwLock } };

use crate::{
    exserver::{ queue, receive, EXServer, S2SMSPType },
    net::quic::qcutils,
    object::gram::{ hookup::S2SHookupGram, BitCommand, BitcommFlag, BitcommVersion },
    queue::BitcommGramQueue,
};

pub async fn connect_exchange_server(
    server: &str,
    port: &str,
    ims_msg_queue: Arc<BitcommGramQueue>,
    ims_rct_queue: Arc<BitcommGramQueue>
) -> Arc<RwLock<EXServer>> {
    let (_client, mut connection) = qcutils::get_client(server, port).await.unwrap(); //client.connect(connect).await?;
    // 打开一个新的双向流，并将其拆分为接收和发送两个部分
    let stream = connection.open_bidirectional_stream().await.unwrap();
    //
    let (rece_stream, send_stream) = stream.split();

    let send_stream = Arc::new(Mutex::new(send_stream));
    let rece_stream = Arc::new(Mutex::new(rece_stream));
    let connection = Arc::new(connection);
    let local_server_id = crate::SERVER_GUID.to_le();
    // 初始化EXServer
    let exserver = receive::init_exserver(
        &send_stream,
        &rece_stream,
        &connection,
        &S2SMSPType::Client,
        local_server_id,
        &ims_msg_queue,
        &ims_rct_queue
    );
    let exserver = Arc::new(RwLock::new(exserver));
    // 启动接收服务
    let exserver = receive::start_exserver_receive_server(&exserver).await;
    // 发送握手信息，以建立链接
    send_hookup(send_stream).await;
    // 启动发送服务
    queue::start_exserver_send_server(&exserver).await
}

// const MY_ERROR_CODE: u32 = 99;

async fn send_hookup(stm: Arc<Mutex<SendStream>>) {
    // 如果当前接收是在ListenServer中，则需要发送反馈信息
    let mut rct_buff: Vec<u8> = S2SHookupGram::create_gram_buf(0);
    let rct_gram = S2SHookupGram::create_hookup_gram_by_mut_vec8(&mut rct_buff);
    rct_gram.set_bitcomm(BitcommFlag::BITCOMM_HOOKUP);
    rct_gram.set_version(BitcommVersion::BITCOMM_VERSION_0_1_0_1);
    rct_gram.set_command(BitCommand::HOOKUP_SERVER);
    rct_gram.set_send_server_id(crate::SERVER_GUID.to_le());
    rct_gram.set_recv_server_id(0);
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

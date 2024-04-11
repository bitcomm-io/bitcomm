use std::sync::Arc;

use tokio::sync::{ Mutex, RwLock };

// use tracing::info;
use crate::{ exserver::{ queue, S2SMSPType }, queue::BitcommGramQueue };
#[allow(unused_imports)]
use crate::{ exserver::receive::{ self, server_data }, net::quic::qcutils };

//
pub async fn listening_exchange_server(
    server_address: String,
    server_port: String,
    ims_msg_queue: Arc<BitcommGramQueue>,
    ims_rct_queue: Arc<BitcommGramQueue>
) -> Arc<tokio::task::JoinHandle<()>> {
    let server_handle = tokio::spawn(async move {
        // 获取服务器实例
        let mut server = qcutils
            ::get_server(server_address.as_str(), server_port.as_str())
            .unwrap();

        // 接受客户端连接并处理
        while let Some(mut connection) = server.accept().await {
            // 设置连接不超时
            connection.keep_alive(true).unwrap();
            let ims_msg_queue = ims_msg_queue.clone();
            let ims_rct_queue = ims_rct_queue.clone();
            // 接受双向流
            if let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
                // 分割流为接收流和发送流
                let (rece_stream, send_stream) = stream.split();
                let send_stream = Arc::new(Mutex::new(send_stream));
                let rece_stream = Arc::new(Mutex::new(rece_stream));
                let connection = Arc::new(connection);
                let local_server_id = crate::server::SERVER_GUID.to_le();
                // 初始化EXServer
                let exserver = receive::init_exserver(
                    &send_stream,
                    &rece_stream,
                    &connection,
                    &S2SMSPType::Server,
                    local_server_id,
                    &ims_msg_queue,
                    &ims_rct_queue
                );
                let exserver = Arc::new(RwLock::new(exserver));
                // 启动接收服务
                let exserver = receive::start_exserver_receive_server(&exserver).await;
                // 启动发送服务
                queue::start_exserver_send_server(&exserver).await;
            }
            // });
        }
    });
    Arc::new(server_handle)
}

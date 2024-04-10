use std::sync::Arc;

// use tracing::info;
use crate::{ exserver::S2SMSPType, queue::BitcommGramQueue };
#[allow(unused_imports)]
use crate::{ exserver::receive::{ self, server_data }, net::quic::qcutils };

//
pub async fn server_message_listening_server(
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
                let (receive_stream, send_stream) = stream.split();
                let (_sstm, _res_exserver) = receive::start_exserver_receive_server(
                    send_stream,
                    receive_stream,
                    connection,
                    crate::SERVER_GUID.to_le(),
                    S2SMSPType::Server,
                    ims_msg_queue.clone(),
                    ims_rct_queue.clone()
                ).await;
            }
            // });
        }
    });
    Arc::new(server_handle)
}

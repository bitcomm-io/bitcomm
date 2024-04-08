use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::info;
use crate::{ exserver::S2SMSPType, queue::BitcommGramQueue };
#[allow(unused_imports)]
use crate::{ exserver::receive::{ self, server_data }, net::quic::qcutils };

//
pub async fn server_message_listening_server(
    server_address: String,
    server_port: String,
    msg_queue: &Arc<BitcommGramQueue>,
    rct_queue: &Arc<BitcommGramQueue>
) -> Arc<tokio::task::JoinHandle<()>> {
    let msg_queue = msg_queue.clone();
    let rct_queue = rct_queue.clone();
    let server_handle = tokio::spawn(async move {
        // 获取服务器实例
        let mut server = qcutils
            ::get_server(server_address.as_str(), server_port.as_str())
            .unwrap();
        // 接受客户端连接并处理
        while let Some(mut connection) = server.accept().await {
            // 设置连接不超时
            connection.keep_alive(true).unwrap();
            let msg_queue = msg_queue.clone();
            let rct_queue = rct_queue.clone();
            // 异步处理连接
            tokio::spawn(async move {
                // 记录连接接受日志
                info!("Connection accepted from {:?}", connection.remote_addr());
                // 接受双向流
                while let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
                    // 分割流为接收流和发送流
                    let (receive_stream, send_stream) = stream.split();
                    /*
                     *  S2SHookupMessage = receive_stream.read();
                     *  创建 ServerBufferPool
                     * send_stream.write(Return S2SHookup)
                     */
                    // 创建发送流的互斥锁
                    let send_stream = Arc::new(Mutex::new(send_stream));
                    // let rece_stream = Arc::new(Mutex::new(receive_stream));
                    // 异步处理数据流
                    let msg_queue = msg_queue.clone();
                    let rct_queue = rct_queue.clone();
                    let _handle = tokio::spawn(async move {
                        // 进入接收数据过程
                        receive::server_data::receive_data_gram(
                            receive_stream,
                            send_stream,
                            msg_queue,
                            rct_queue,
                            S2SMSPType::ListenServer
                        ).await;
                        // 在此需要清理IP->ClientID+DeviceID信息
                    });
                }
            });
        }
    });
    Arc::new(server_handle)
}

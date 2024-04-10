
use s2n_quic::{client::Connect, Client, Connection};
#[allow(unused_imports)]
use s2n_quic::{
    stream::{BidirectionalStream, SendStream},
    Server,
};
// use tokio::sync::{ mpsc::Sender, Mutex };

use std::{error::Error, net::SocketAddr, time::Duration};
// use s2n_quic::stream::ReceiveStream;
// use tokio::sync::Mutex;
use tracing::info;
// use tracing::error;

use crate::net::quic::slowloris;
///
fn get_im_server_port(server:&str,port:&str) -> String {
    format!("{}:{}", server, port)
}

/// 注意：此证书仅供演示目的使用！
static CERT_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../certs/cert.pem"));
/// 注意：此密钥仅供演示目的使用！
static KEY_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../certs/key.pem"));

/// 获取服务器实例
pub fn get_server(server:&str,port:&str) -> Result<Server, Box<dyn Error>> {
    let connection_limits = s2n_quic::provider::limits::Limits::new()
        .with_max_handshake_duration(Duration::from_secs(5))
        .expect("connection limits are valid");
    let endpoint_limits = s2n_quic::provider::endpoint_limits::Default::builder()
        .with_inflight_handshake_limit(100)?
        .build()?;
    let server_address = get_im_server_port(server,port);
    let server = Server::builder()
        // 提供上述定义的`connection_limits`
        .with_limits(connection_limits)?
        // 提供上述定义的`endpoint_limits`
        .with_endpoint_limits(endpoint_limits)?
        // 提供由`dos-mitigation/src/lib.rs`中定义的`slowloris::MyConnectionSupervisor`和默认事件跟踪订阅者组成的元组。
        // 此组合将允许利用`MyConnectionSupervisor`的slowloris缓解功能以及事件跟踪。
        .with_event((
            slowloris::MyConnectionSupervisor,
            s2n_quic::provider::event::tracing::Subscriber::default(),
        ))?
        .with_tls((CERT_PEM, KEY_PEM))?
        // .with_io("127.0.0.1:4433")?
        .with_io(server_address.as_str())?
        .start()?;
    info!("quic listening on {}", server.local_addr().unwrap());
    Ok(server)
}
// 
pub async fn get_client(server: &str, port: &str) -> Result<(Client,Connection), Box<dyn Error>> {
    let client = Client::builder()
        .with_tls(CERT_PEM)?
        // 使用指定的 PEM 证书配置 TLS
        .with_io("0.0.0.0:0")?
        .start()?; // 启动 Client

    let addr: SocketAddr = get_im_server_port(server, port).parse()?;
    let connect = Connect::new(addr); //.with_server_name("localhost");
    // 连接服务器，获取 Connection 实例
    let mut connection = client.connect(connect).await?;
    // 确保连接不会因不活动而超时
    connection.keep_alive(true)?;
    Result::Ok((client,connection))
}

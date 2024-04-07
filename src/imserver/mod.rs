pub mod listen;
pub mod receive;
pub mod command;
pub mod message;
pub mod receipt;
pub mod bridge;

use std::sync::Arc;

use getset::{ Getters, Setters };
use tokio::sync::RwLock;


use crate::queue::{ GramBufferPool, BitcommGramQueue };


// #[allow(dead_code)]
#[derive(Debug, Getters, Setters)]
pub struct IMServer {
    // 服务地址
    #[getset(set = "pub", get = "pub")]
    server_address: String,
    // 服务端口
    #[getset(set = "pub", get = "pub")]
    server_port: String,
    // 最大客户端连接数
    #[getset(set = "pub", get = "pub")]
    client_max_num: u32,
    // 监听服务
    #[getset(set = "pub", get = "pub")]
    listen_server: Option<Arc<tokio::task::JoinHandle<()>>>,
    // 消息队列及服务
    #[getset(set = "pub", get = "pub")]
    message_queue: Arc<BitcommGramQueue>,
    #[getset(set = "pub", get = "pub")]
    msg_queue_server: Option<Arc<tokio::task::JoinHandle<()>>>,
    // 回执队列及服务
    #[getset(set = "pub", get = "pub")]
    receipt_queue: Arc<BitcommGramQueue>,
    #[getset(set = "pub", get = "pub")]
    rct_queue_server: Option<Arc<tokio::task::JoinHandle<()>>>,
    // 发送队列及服务
    #[getset(set = "pub", get = "pub")]
    sendmsg_queue: Arc<BitcommGramQueue>,
    #[getset(set = "pub", get = "pub")]
    sng_queue_server: Option<Arc<tokio::task::JoinHandle<()>>>,
    // 发送缓冲及服务
    #[getset(set = "pub", get = "pub")]
    resend_buffer: Arc<RwLock<GramBufferPool>>,
    #[getset(set = "pub", get = "pub")]
    rsd_buffer_server: Option<Arc<tokio::task::JoinHandle<()>>>,
    // 发件箱，收件箱，离线收件箱
}

impl IMServer {
    //
    pub fn new(
        server_address: String,
        server_port: String,
        client_max_num: u32,
        queue_size: usize
    ) -> Self {
        //
        IMServer {
            server_address,
            server_port,
            client_max_num,
            listen_server: None,
            message_queue: Arc::new(BitcommGramQueue::new(queue_size)),
            msg_queue_server: None,
            receipt_queue: Arc::new(BitcommGramQueue::new(queue_size)),
            rct_queue_server: None,
            sendmsg_queue: Arc::new(BitcommGramQueue::new(queue_size)),
            sng_queue_server: None,
            resend_buffer: Arc::new(RwLock::new(GramBufferPool::new(queue_size))),
            rsd_buffer_server: None,
        }
    }
    //
    pub async fn start_imserver(&mut self) -> &mut Self {
        // 缓冲重发任务
        let handle = message::start_resend_buffer_server(
            &self.sendmsg_queue,
            &self.resend_buffer
        ).await;
        self.set_rsd_buffer_server(Some(handle.clone()));
        // 回执发送任务
        let handle = receipt::start_receipt_queue_server(
            &self.receipt_queue,
            &self.resend_buffer
        ).await;
        self.set_rct_queue_server(Some(handle));
        // 发送队列任务
        let handle = message::start_send_message_queue_server(
            &self.sendmsg_queue,
            &self.resend_buffer
        ).await;
        self.set_sng_queue_server(Some(handle.clone()));
        // 消息队列任务
        let handle = message::start_message_queue_server(
            &self.message_queue,
            &self.sendmsg_queue
        ).await;
        self.set_msg_queue_server(Some(handle.clone()));
        // 监听服务
        let handle = listen::start_client_message_listening_server(
            self.server_address.clone(),
            self.server_port.clone(),
            &self.message_queue,
            &self.receipt_queue
        ).await;
        self.set_listen_server(Some(handle.clone()));

        self
    }
}

// 定义模块，这些模块可能包含与服务器监听、连接、队列管理等相关的功能。
pub mod listen;
pub mod connect;
pub mod queue;
mod receive;
mod message;
mod receipt;
mod hookup;
mod reply;

use getset::CopyGetters;
// 使用getset库提供的Getters和Setters宏来生成字段的getter和setter方法。
use getset::Getters;
use getset::Setters;
// 使用lazy_static库创建全局静态变量，这里用于存储服务器间消息处理的状态。
use lazy_static::lazy_static;
use s2n_quic::stream::ReceiveStream;
// 使用s2n_quic库的SendStream类型来发送数据流。
use s2n_quic::stream::SendStream;
use s2n_quic::Connection;
// 使用标准库的HashMap来存储键值对。
use std::collections::HashMap;
// 使用Arc和Mutex来提供线程安全的引用计数和互斥锁。
use std::sync::Arc;
use tokio::sync::Mutex;
// 使用RwLock来提供读写锁。
use tokio::sync::RwLock;

// 使用crate内定义的BITServerID类型。
use crate::object::BITServerID;
use crate::queue::BitcommGramQueue;
use crate::queue::GramBufferPool;
use crate::queue::EVENT_QUEUE_LEN;

use self::connect::connect_server;
use self::listen::server_listen;

// use self::receive::server_data;

// 使用lazy_static创建一个全局静态的服务器到服务器池，用于存储服务器间的消息处理对象。
lazy_static! {
    // key: 目标服务器的ID，Value: 元组（链接，接收流，发送流）
    pub static ref SERVER_2_SERVER_POOL : Arc<RwLock<HashMap<BITServerID,Arc<RwLock<EXServer>>>>> = Arc::new(RwLock::new(HashMap::new()));
}

// 定义一个枚举类型，表示服务器到服务器消息处理的类型。
#[derive(Debug, Clone)]
pub enum EXServerType {
    Client, // 客户端连接模式
    Server, // 服务端监听模式
}

// 定义一个与C语言兼容的结构体，用于服务器间消息处理。
#[repr(C)]
#[derive(Debug, CopyGetters, Getters, Setters)]
pub struct EXServer {
    // 消息处理类型
    #[getset(get = "pub")]
    exserver_type: EXServerType,
    // 当前服务器ID
    #[getset(set = "pub", get = "pub")]
    local_server_id: BITServerID,
    // 目标服务器ID
    #[getset(set = "pub", get = "pub")]
    remote_server_id: BITServerID,

    // 发送队列及服务
    #[getset(get = "pub")]
    sendmsg_queue: Arc<BitcommGramQueue>,
    #[getset(set = "pub", get = "pub")]
    sng_queue_task: Option<Arc<tokio::task::JoinHandle<()>>>,
    // 发送缓冲及服务
    #[getset(get = "pub")]
    resend_buffer: Arc<RwLock<GramBufferPool>>,
    #[getset(set = "pub", get = "pub")]
    rsd_buffer_task: Option<Arc<tokio::task::JoinHandle<()>>>,
    // 长链接
    #[getset(get = "pub")]
    connection: Arc<Connection>,
    // 发送数据流
    #[getset(get = "pub")]
    send_stream: Arc<Mutex<SendStream>>,
    // 接收数据流
    #[getset(get = "pub")]
    rece_stream: Arc<Mutex<ReceiveStream>>,
    // 数据接收服务
    #[getset(set = "pub", get = "pub")]
    rece_task: Option<Arc<tokio::task::JoinHandle<()>>>,
    //TODO: 连接，接收流，队列，缓冲池 与IMServer的不同处：即可以监听，也可以连接，IMServer只能监听
    // 本机IMServer中的消息队列,通过Bridge传递过来
    #[getset(get = "pub")]
    ims_msg_queue: Arc<BitcommGramQueue>,
    // 本机IMServer中的回执队列,通过Bridge传递过来
    #[getset(get = "pub")]
    ims_rct_queue: Arc<BitcommGramQueue>,
}

// 实现S2SMessageProcess结构体的方法。
impl EXServer {
    // 构造函数，用于创建一个新的S2SMessageProcess实例。
    pub fn new(
        exserver_type: EXServerType,
        local_server_id: BITServerID,
        ims_msg_queue: Arc<BitcommGramQueue>,
        ims_rct_queue: Arc<BitcommGramQueue>,
        connection: Arc<Connection>,
        send_stream: Arc<Mutex<SendStream>>,
        rece_stream: Arc<Mutex<ReceiveStream>>
    ) -> Self {
        EXServer {
            exserver_type,
            local_server_id,
            remote_server_id:0,
            sendmsg_queue: Arc::new(BitcommGramQueue::new(EVENT_QUEUE_LEN)),
            sng_queue_task: None,
            resend_buffer: Arc::new(RwLock::new(GramBufferPool::new(EVENT_QUEUE_LEN))),
            rsd_buffer_task: None,
            connection,
            send_stream,
            rece_stream,
            rece_task: None,
            ims_msg_queue,
            ims_rct_queue,
        }
    }

}

// 异步函数，用于将一个S2SMessageProcess实例放入全局池中。
pub async fn put_s2s_msp(s2smp: &Arc<RwLock<EXServer>>) {
    let mut s2s_p = SERVER_2_SERVER_POOL.write().await;
    s2s_p.insert(s2smp.read().await.remote_server_id, s2smp.clone());
}

// 异步函数，用于从全局池中移除一个S2SMessageProcess实例。
pub async fn remove_s2s_msp(remote_server_id: BITServerID) -> Option<Arc<RwLock<EXServer>>> {
    let mut s2s_p = SERVER_2_SERVER_POOL.write().await;
    s2s_p.remove(&remote_server_id)
}

// 异步函数，用于从全局池中获取一个S2SMessageProcess实例。
pub async fn get_s2s_msp(remote_server_id: BITServerID) -> Option<Arc<RwLock<EXServer>>> {
    let s2s_p = SERVER_2_SERVER_POOL.read().await;
    s2s_p.get(&remote_server_id).map(|x| x.clone())
}






pub async fn connect_exchange_server(
    server: &str,
    port: &str,
    ims_msg_queue: Arc<BitcommGramQueue>,
    ims_rct_queue: Arc<BitcommGramQueue>
) -> Arc<RwLock<EXServer>> {
    connect_server::connect_exchange_server(
        server,
        port,
        ims_msg_queue,
        ims_rct_queue
    ).await
}

pub async fn listening_exchange_server(
    server_address: String,
    server_port: String,
    msg_queue: Arc<BitcommGramQueue>,
    rct_queue: Arc<BitcommGramQueue>
) -> Arc<tokio::task::JoinHandle<()>> {
    server_listen::listening_exchange_server( server_address, server_port, msg_queue, rct_queue).await
}

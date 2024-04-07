pub mod listen;
pub mod connect;
pub mod queue;
mod receive;
mod message;
mod receipt;
mod hookup;

// use getset::CopyGetters;
use getset::Getters;
use getset::Setters;
use lazy_static::lazy_static;
// use s2n_quic::stream::ReceiveStream;
use s2n_quic::stream::SendStream;
// use s2n_quic::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use crate::object::BITServerID;


lazy_static! {
    // key:目标服务器的ID，Value：元组（链接，接收流，发送流）
    pub static ref SERVER_2_SERVER_POOL : Arc<RwLock<HashMap<BITServerID,Arc<S2SMessageProcess>>>> = Arc::new(RwLock::new(HashMap::new()));
}
#[derive(Debug,Clone)]
pub enum S2SMSPType  {
    ListenServer,
    ConnectServer,
}

#[repr(C)] // 与C语言兼容
#[derive(Debug, Getters, Setters)]
pub struct S2SMessageProcess {
    #[getset(set = "pub", get = "pub")]
    s2smsp_type:S2SMSPType,
    // 当前服务器ID
    #[getset(set = "pub", get = "pub")]
    local_server_id: BITServerID,
    // 目标服务器ID
    #[getset(set = "pub", get = "pub")]
    remote_server_id: BITServerID,
    //
    #[getset(set = "pub", get = "pub")]
    send_stream: Arc<Mutex<SendStream>>,
}

impl S2SMessageProcess {
    //
    pub fn new(
        s2smsp_type:S2SMSPType,
        local_server_id: BITServerID,
        remote_server_id: BITServerID,
        // connection: Arc<Mutex<Connection>>,
        send_stream: Arc<Mutex<SendStream>>
    ) -> Self {
        S2SMessageProcess {
            s2smsp_type,
            local_server_id,
            remote_server_id,
            // connection,
            send_stream,
        }
    }
}
//
pub async fn put_s2s_msp(s2smp: Arc<S2SMessageProcess>) {
    let mut s2s_p = SERVER_2_SERVER_POOL.write().await;
    s2s_p.insert(s2smp.remote_server_id, s2smp);
}
//
pub async fn remove_s2s_msp(remote_server_id: BITServerID) -> Option<Arc<S2SMessageProcess>> {
    let mut s2s_p = SERVER_2_SERVER_POOL.write().await;
    s2s_p.remove(&remote_server_id)
}
//
pub async fn get_s2s_msp(remote_server_id: BITServerID) -> Option<Arc<S2SMessageProcess>> {
    let s2s_p = SERVER_2_SERVER_POOL.read().await;
    s2s_p.get(&remote_server_id).map(|x| x.clone())
}

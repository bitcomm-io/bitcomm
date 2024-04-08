mod client;
pub mod data;
pub mod net;
pub mod object;
pub mod utils;
pub mod imserver;
pub mod exserver;
pub mod config;
pub mod queue;
pub mod bridge;


use lazy_static::lazy_static;
use crate::utils::{ get_machine_guid, get_md5_u32id };
lazy_static! {
    //
    pub static ref MACHINE_GUID: String = get_machine_guid().unwrap();
    // 获取当前服务器的SERVER_GUID，全球唯一
    pub static ref SERVER_GUID: u32 = {
        if let Some(name) = get_machine_guid() { get_md5_u32id(name.as_str()) } else { 0 }
    };
}
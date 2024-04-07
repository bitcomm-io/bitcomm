use bitflags::bitflags;
use getset::{CopyGetters, Setters};
use serde::{Deserialize, Serialize};

use super::BITServerID;

bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq, Eq)]
    pub struct ServerType:u16 {
        const SERVER_INTERNET    = 0x0001;
        const SERVER_INTRANET    = 0x0002;
    }
}

pub static SERVER_INTERNET: u16 = 0x0001;
pub static SERVER_INTRANET: u16 = 0x0002;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitServer {
    #[serde(rename = "server_id")]
    pub server_id: u32,
    #[serde(rename = "server_type")]
    pub server_type: u16,
    #[serde(rename = "server_name")]
    pub server_name: String,
    #[serde(rename = "server_address")]
    pub server_address: String,
    #[serde(rename = "server_port")]
    pub server_port: String,
}
impl BitServer {
    // 这里可以添加结构体的构造函数或其他方法
}
#[repr(C)] // 与C语言兼容c
#[derive(Debug, Clone, Copy, CopyGetters, Setters, PartialEq, Eq)]
pub struct ClientID {
    #[getset(set = "pub", get_copy = "pub")]
    planet: BITServerID, //ClientPlanet,
    #[getset(set = "pub", get_copy = "pub")]
    object: u32,
}
impl Into<u64> for ClientID {
    fn into(self) -> u64 {
        ((self.planet as u64) << 32) | (self.object as u64)
    }
}
impl ClientID {
    //
    pub fn new(planet: u32, object: u32) -> Self {
        Self { planet, object }
    }
    //
    pub fn get_guid(&self) -> u64 {
        ((self.planet as u64) << 32) | (self.object as u64)
    }
    //
    pub fn get_hex(&self) -> String {
        format!("{:x}", self.get_guid())
    }
    //
    // pub fn get_key(&self, device_id: u32) -> u128 {
    //     let num_u64 = ((self.planet as u64) << 32) | (self.object as u64);
    //     ((num_u64 as u128) << 64) | (device_id as u128)
    // }
}

bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq, Eq)]
    pub struct ClientType:u32 {
        const CLIENT_PEOPLE    = 0x0001;
        const CLIENT_GROUP     = 0x0002;
        const CLIENT_DEVICE    = 0x0003;
        const CLIENT_ROBOT     = 0x0004;
        const CLIENT_SERVICE   = 0x0010;
    }
}
bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct ClientPlanet:u32 {
        const PLANET_EARTH    = 0x00000000;
        const PLANET_MAR      = 0x00000010;
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy, CopyGetters, Setters, PartialEq, Eq)]
pub struct DeviceConnInfo {
    #[getset(set = "pub", get_copy = "pub")]
    device_id: u32,
    #[getset(set = "pub", get_copy = "pub")]
    device_state: DeviceConnState,
}
impl DeviceConnInfo {
    pub fn new(device_id: u32, device_state: DeviceConnState) -> Self {
        Self {
            device_id,
            device_state,
        }
    }
}
bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct DeviceConnState:u32 {
        const STATE_ONLINE    = 0x00010001;
        const STATE_ONBACK    = 0x00010002;
        const STATE_OFFLINE   = 0x00010004;
    }
}

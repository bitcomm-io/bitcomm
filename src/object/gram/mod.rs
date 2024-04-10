pub mod command;
pub mod hookup;
pub mod message;
pub mod receipt;
pub mod reply;

use bitflags::bitflags;
// use bitflags::Flags;
// use bytes::BytesMut;
use getset::{ Getters, Setters };
// use std::cell::RefCell;
use std::error;
use std::fmt;
// use std::rc::Rc;
// use crate::client::ClientID;
// use crate::client::ClientType;
use self::command::CommandGram;
use self::hookup::S2SHookupGram;
use self::message::MessageGram;
use self::receipt::ReceiptGram;
use self::reply::ReplyGram;
use bytes::Bytes;
use std::sync::Arc;

bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct ReturnCode:u32 {
        const RETURN_OK                     =   0x00000000;
        const RETURN_NOT_LOGIN              =   0x00000001;
        const RETURN_TWO_SESSION_NO_SAME    =   0x00000002;
    }
}

bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct BitcommFlag:u32 {
        const BITCOMM_MESSAGE    =   0x4D435442; // BTCM
        const BITCOMM_RECEIPT    =   0x54525442; // BTRT
        const BITCOMM_REPLY      =   0x4C525442; // BTRL
        const BITCOMM_COMMAND    =   0x43435442; // BTCC
        const BITCOMM_PING       =   0x49505442; // BTPI
        const BITCOMM_PONG       =   0x4F505442; // BTPO
        const BITCOMM_HOOKUP     =   0x4B485442; // BTHK 两个服务器之前的握手协议
        const BITCOMM_NODEF      =   0xFFFFFFFF;
    }
}

bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct BitcommVersion:u32 {
        const BITCOMM_VERSION_0_1_0_1  =  0x00010001; // 0.1.0.1
    }
}

bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy,PartialEq,Eq)]
    pub struct BitCommand:u32 {
        const RESP_MASK      = 0x80000000;  // 回复的消息
        const LOGIN_COMMAND  = 0x00000001;  // 登录命令
        const LOGOUT_COMMAND = 0x00000002;  // 登出命令
        const SEND_MESS      = 0x00000004;  // 发送信息
        const SEND_PING      = 0x10000000;  // 返回PING
        const RESP_PONG      = Self::SEND_PING.bits() | Self::RESP_MASK.bits();  // 返回PONG
        const HOOKUP_SERVER  = 0x20000000; // 服务器Hookup

    }
}
bitflags! {
    #[repr(C)]   // 与C语言兼容
    #[derive(Debug,Clone,Copy)]
    pub struct MessageType:u32 {
        const MESS_TEXT    = 0x00000001;
        const MESS_IMAGES  = 0x00000002;
        const MESS_VIDEO   = 0x00000003;
        const MESS_FILES   = 0x00000004;
        const MESS_POSTION = 0x00000005;
    }
}

fn get_gram_by_u8<'a, T>(gram_buf: &[u8]) -> &'a T {
    // 将字节切片转换为对 T 类型的引用
    unsafe {
        &*(gram_buf[0..].as_ptr() as *const T)
    }
    // unsafe { & *(gram_buf.as_ptr() as *const T) }
}
fn get_mut_gram_by_u8<'a, T>(gram_buf: &mut [u8]) -> &'a mut T {
    // 将字节切片转换为对 T 类型的引用
    unsafe {
        &mut *(gram_buf[0..].as_mut_ptr() as *mut T)
    }
    // unsafe { & *(gram_buf.as_mut_ptr() as *mut T) }
}

fn size_of_align_data_gram<T>() -> usize {
    let size = std::mem::size_of::<T>();
    let align = std::mem::align_of::<T>();

    // 计算需要的补偿字节数
    let padding = (align - (size % align)) % align;

    // 计算最终大小
    size + padding
}

#[derive(Debug)]
pub enum BitcommDataGram {
    Command {
        data_buff: Arc<Bytes>,
        data_gram: Arc<CommandGram>,
    },
    Message {
        data_buff: Arc<Bytes>,
        data_gram: Arc<MessageGram>,
    },
    Receipt {
        data_buff: Arc<Bytes>,
        data_gram: Arc<ReceiptGram>,
    },
    Reply {
        data_buff: Arc<Bytes>,
        data_gram: Arc<ReplyGram>,
    },
    Pingpong(Arc<BitcommFlag>),
    S2SHookup {
        data_buff: Arc<Bytes>,
        data_gram: Arc<S2SHookupGram>,
    },
}

#[derive(Debug, Getters, Setters)]
pub struct DataGramError {
    #[getset(set = "pub", get = "pub")]
    error_code: u32,
    #[getset(set = "pub", get = "pub")]
    details: String,
}

impl DataGramError {
    pub fn new(code: u32, tails: &str) -> Self {
        DataGramError {
            error_code: code,
            details: String::from(tails),
        }
    }
}

// 为自定义错误类型实现 Display 和 Error trait
impl fmt::Display for DataGramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl error::Error for DataGramError {
    fn description(&self) -> &str {
        &self.details
    }
}

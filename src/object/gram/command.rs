use crate::object::client::ClientID;
use crate::object::gram::{
    get_gram_by_u8, get_mut_gram_by_u8, size_of_align_data_gram, BitCommand, BitcommFlag,
    BitcommVersion, MessageType, ReturnCode,
};
use crate::object::{DeviceID, MessageID};
use getset::{CopyGetters, Setters};

use super::message::MessageGram;

#[repr(C)] // 与C语言兼容
#[derive(Debug, Clone, Copy, CopyGetters, Setters)]
pub struct CommandGram {
    #[getset(set = "pub", get_copy = "pub")]
    bitcomm: BitcommFlag,
    #[getset(set = "pub", get_copy = "pub")]
    version: BitcommVersion,
    #[getset(set = "pub", get_copy = "pub")]
    command: BitCommand,
    #[getset(set = "pub", get_copy = "pub")]
    device_id: DeviceID,
    #[getset(set = "pub", get_copy = "pub")]
    sender: ClientID,
    #[getset(set = "pub", get_copy = "pub")]
    message_type: MessageType,
    #[getset(set = "pub", get_copy = "pub")]
    message_id: MessageID,
    #[getset(set = "pub", get_copy = "pub")]
    return_code: ReturnCode,
    #[getset(set = "pub", get_copy = "pub")]
    data_time: u32, // 时间戳
    #[getset(set = "pub", get_copy = "pub")]
    data_size: u32,
}

impl CommandGram {
    // pub const BITCOMM_COMMAND : u32  = 0x43435442; // BTCC // 命令报文

    ///
    ///
    /// # Arguments
    ///
    /// * `data_array`:
    ///
    /// returns: bool
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn is_pingpong(data_array: &[u8]) -> bool {
        if data_array.len() == std::mem::size_of::<BitcommFlag>() {
            let bitcomm_flag = get_gram_by_u8::<u32>(data_array);
            return *bitcomm_flag == BitcommFlag::BITCOMM_PING.bits()
                || *bitcomm_flag == BitcommFlag::BITCOMM_PONG.bits();
        }
        false
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `gram_buf`:
    ///
    /// returns: &BitcommFlag
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn get_bitcomm_flag_by_u8(gram_buf: &[u8]) -> &BitcommFlag {
        #[allow(unused_mut)]
        let bitcomm_flag_ref: &BitcommFlag = get_gram_by_u8::<BitcommFlag>(gram_buf);
        bitcomm_flag_ref
    }
    // 
    pub fn is_command(data_array: &[u8]) -> bool {
        if data_array.len() == Self::get_size() {
            let bytes: [u8; 4] = [data_array[0], data_array[1], data_array[2], data_array[3]]; 
            let value = u32::from_le_bytes(bytes);
            value == BitcommFlag::BITCOMM_COMMAND.bits()
        } else {
            false
        }
    }
    // 
    pub fn get_command_gram_by_u8(gram_buf: &[u8]) -> &CommandGram {
        #[allow(unused_mut)]
        let data_gram_head_ref: &CommandGram = get_gram_by_u8::<CommandGram>(gram_buf); //unsafe {& *(gram_buf[0..].as_ptr() as *const DataGramHead)};
                                                                                        // 设置结构体下,缓冲区的大小
                                                                                        // data_gram_ref.data_size = (gram_buf.len() - size_of_align_data_gram()) as u32;
        data_gram_head_ref
    }
    //
    pub fn create_command_gram_by_mut_vec8(byte_array: &mut Vec<u8>) -> &mut CommandGram {
        let gram_buf: &mut [u8] = byte_array.as_mut_slice();
        Self::create_command_gram_by_mut_u8(gram_buf)
    }
    //
    pub fn create_command_gram_by_mut_u8(gram_buf: &mut [u8]) -> &mut CommandGram {
        #[allow(unused_mut)]
        let mut data_gram_ref: &mut CommandGram = get_mut_gram_by_u8::<CommandGram>(gram_buf); //unsafe {&mut *(gram_buf[0..].as_mut_ptr() as *mut DataGram)};
                                                                                               // 设置结构体标志
        data_gram_ref.data_size =
            (gram_buf.len() - size_of_align_data_gram::<CommandGram>()) as u32;
        data_gram_ref.set_bitcomm(BitcommFlag::BITCOMM_COMMAND);
        data_gram_ref.set_version(BitcommVersion::BITCOMM_VERSION_0_1_0_1);
        data_gram_ref
    }
    //
    pub fn get_size() -> usize {
        size_of_align_data_gram::<Self>()
    }
    //
    pub fn create_gram_buf<'a>(data_size: usize) -> Vec<u8> {
        // 创建一个指定大小的 Vec<u8>
        #[allow(unused_mut)]
        let mut vec_u8: Vec<u8> = vec![0x00; data_size + size_of_align_data_gram::<CommandGram>()];
        vec_u8
    }
    //
    pub fn create_command_gram_from_gram<'a>(
        buf: &'a mut [u8],
        value: &'a CommandGram,
    ) -> &'a mut CommandGram {
        let command = Self::create_command_gram_by_mut_u8(buf);
        unsafe {
            std::ptr::copy_nonoverlapping(
                value, command, 1, // 1 表示复制一个元素
            );
        }
        command.set_bitcomm(value.bitcomm());
        command.set_command(value.command() | BitCommand::RESP_MASK);
        command
    }
    //
    pub fn create_command_gram_from_message_gram<'a>(
        buf: &'a mut [u8],
        value: &'a MessageGram,
    ) -> &'a mut CommandGram {
        let command = Self::create_command_gram_by_mut_u8(buf);
        command.set_bitcomm(BitcommFlag::BITCOMM_COMMAND);
        command.set_command(value.command() | BitCommand::RESP_MASK);
        command.set_sender(value.sender());
        // command.set_sender_type(value.sender_type());
        command.set_message_id(value.message_id());
        command.set_message_type(value.message_type());
        command.set_version(value.version());
        command.set_device_id(value.device_id());
        command
    }
}

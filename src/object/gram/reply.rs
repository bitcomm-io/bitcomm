use crate::object::{client::ClientID, gram::BitcommFlag, DeviceID, MessageID};
use bytes::Bytes;
use getset::{CopyGetters, Setters};

use super::{get_gram_by_u8, get_mut_gram_by_u8, message::MessageGram, size_of_align_data_gram, BitCommand, BitcommVersion, MessageType};


/*
    此结构体为服务端收到数据后的立即回馈，不管是Client发过来还是Server发过来，收到此信息的接收方无需再次反馈，
    在规定时间内，如果对方没有收到此信息，则需要重发原信息
*/
#[repr(C)] // 与C语言兼容
#[derive(Debug, Clone, Copy, CopyGetters, Setters)]
pub struct ReplyGram {
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
    receiver: ClientID,
    #[getset(set = "pub", get_copy = "pub")]
    data_time: u32, // 时间戳
}

impl ReplyGram {

    pub fn get_reply_gram_key(&self) -> u128 {
        let num_u64_1 = self.sender().get_guid(); //((self.sender().planet().bits() as u64) << 32) | (self.sender().object() as u64);
        let num_u64_2 = ((self.receiver().object() as u64) << 32) | (self.message_id() as u64);
        ((num_u64_1 as u128) << 64) | (num_u64_2 as u128)
    }
    //
    pub fn is_bitcomm_reply_flag(data_array: &[u8]) -> bool {
        if data_array.len() == std::mem::size_of::<BitcommFlag>() {
            let bitcomm_flag = get_gram_by_u8::<u32>(data_array);
            return *bitcomm_flag == BitcommFlag::BITCOMM_REPLY.bits();
        }
        false
    }

    //
    pub fn get_bitcomm_flag_by_u8(gram_buf: &[u8]) -> &BitcommFlag {
        #[allow(unused_mut)]
        let bitcomm_flag_ref: &BitcommFlag = get_gram_by_u8::<BitcommFlag>(gram_buf);                                                                            
         bitcomm_flag_ref
    }

    // 
    pub fn is_reply(data_array: &[u8]) -> bool {
        if data_array.len() == Self::get_size() {
            let bytes: [u8; 4] = [data_array[0], data_array[1], data_array[2], data_array[3]];
            let value = u32::from_le_bytes(bytes);
            value == BitcommFlag::BITCOMM_REPLY.bits()
        } else {
            false
        }
    }
    // 
    pub fn get_reply_gram_by_u8(gram_buf: &[u8]) -> &ReplyGram {
        let data_gram_head_ref: &ReplyGram = get_gram_by_u8::<ReplyGram>(gram_buf); 
        data_gram_head_ref
    }
    pub fn get_reply_gram_by_bytes(gram_buf:&Bytes) -> &ReplyGram {
        Self::get_reply_gram_by_u8(gram_buf)
    }
    
    //
    pub fn create_reply_gram_by_mut_vec8(byte_array: &mut Vec<u8>) -> &mut ReplyGram {
        let gram_buf: &mut [u8] = byte_array.as_mut_slice();
        Self::create_reply_gram_by_mut_u8(gram_buf)
    }
    //
    pub fn create_reply_gram_by_mut_u8(gram_buf: &mut [u8]) -> &mut ReplyGram {
        #[allow(unused_mut)]
        let mut data_gram_ref: &mut ReplyGram = get_mut_gram_by_u8::<ReplyGram>(gram_buf);
        data_gram_ref.set_bitcomm(BitcommFlag::BITCOMM_REPLY);
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
        let mut vec_u8: Vec<u8> = vec![0x00; data_size + size_of_align_data_gram::<ReplyGram>()];
        vec_u8
    }
    // 
    pub fn create_reply_from_message<'a>(gram_buf: &'a mut [u8],
                                     message : &'a MessageGram) -> &'a mut ReplyGram {
        let reply = Self::create_reply_gram_by_mut_u8(gram_buf);
        reply.set_bitcomm(BitcommFlag::BITCOMM_REPLY);
        reply.set_command(message.command() | BitCommand::RESP_MASK);

        reply.set_version(BitcommVersion::BITCOMM_VERSION_0_1_0_1);
        reply.set_device_id(message.device_id());
        reply.set_sender(message.sender());
        reply.set_message_type(message.message_type());
        reply.set_message_id(message.message_id());
        reply.set_receiver(message.receiver());
        //data_time: u32, // 时间戳
        reply
    }
}

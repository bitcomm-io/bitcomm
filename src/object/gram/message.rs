use crate::object::client::ClientID;
use crate::object::gram::{
    get_gram_by_u8, get_mut_gram_by_u8, size_of_align_data_gram, BitCommand, BitcommFlag,
    BitcommVersion, MessageType,
};
use crate::object::{DeviceID, MessageID};
use getset::{CopyGetters, Setters};

#[repr(C)] // 与C语言兼容
#[derive(Debug, Clone, Copy, CopyGetters, Setters)]
pub struct MessageGram {
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
    ref_message_id: MessageID,
    #[getset(set = "pub", get_copy = "pub")]
    receiver: ClientID,

    reserve1: u64,
    reserve2: u64,
    reserve3: u32,
    reserve4: u32,
    #[getset(set = "pub", get_copy = "pub")]
    data_time: u32, // 时间戳

    #[getset(set = "pub", get_copy = "pub")]
    data_size: u32,
}

impl MessageGram {
    /// sender.planet | sender.object | receiver.object | + message_id
    pub fn get_message_gram_key(&self) -> u128 {
        let num_u64_1 = self.sender().get_guid(); //((self.sender().planet().bits() as u64) << 32) | (self.sender().object() as u64);
        let num_u64_2 = ((self.receiver().object() as u64) << 32) | (self.message_id() as u64);
        ((num_u64_1 as u128) << 64) | (num_u64_2 as u128)
    }
    //
    pub fn copy_data2tail(&self, src_data: &[u8], des_data: &mut [u8]) {
        let begin_index = self.data_size as usize;
        let end_index = des_data.len();
        des_data[begin_index..end_index].copy_from_slice(src_data);
    }
    //
    pub fn get_size() -> usize {
        size_of_align_data_gram::<Self>()
    }
    //
    pub fn is_message(data_array: &[u8]) -> bool {
        if data_array.len() >= Self::get_size() {
            let bytes: [u8; 4] = [data_array[0], data_array[1], data_array[2], data_array[3]];
            let value = u32::from_le_bytes(bytes);
            value == BitcommFlag::BITCOMM_MESSAGE.bits()
        } else {
            false
        }
    }
    //
    pub fn create_gram_data_buf<'a>(source: &[u8]) -> Vec<u8> {
        // 创建一个指定大小的 Vec<u8>
        // #[allow(unused_mut)]
        let md_len = size_of_align_data_gram::<MessageGram>();
        let mut vec_u8: Vec<u8> = vec![0x00; source.len() + md_len];
        vec_u8[md_len..md_len + source.len()].copy_from_slice(source);
        vec_u8
    }
    //
    pub fn create_gram_buf<'a>(data_size: usize) -> Vec<u8> {
        // 创建一个指定大小的 Vec<u8>
        // #[allow(unused_mut)]
        let vec_u8: Vec<u8> = vec![0x00; data_size + size_of_align_data_gram::<MessageGram>()];
        vec_u8
    }
    //
    pub fn create_message_data_gram_by_mut_vec8(byte_array: &mut Vec<u8>) -> &mut MessageGram {
        let gram_buf: &mut [u8] = byte_array.as_mut_slice();
        MessageGram::create_message_data_gram_by_mut_u8(gram_buf)
    }
    //
    pub fn create_message_data_gram_by_mut_u8(gram_buf: &mut [u8]) -> &mut MessageGram {
        let data_gram_ref: &mut MessageGram = get_mut_gram_by_u8::<MessageGram>(gram_buf); //unsafe {&mut *(gram_buf[0..].as_mut_ptr() as *mut DataGram)};
                                                                                           // 设置结构体下,缓冲区的大小
        data_gram_ref.set_bitcomm(BitcommFlag::BITCOMM_MESSAGE);
        data_gram_ref.set_version(BitcommVersion::BITCOMM_VERSION_0_1_0_1);
        data_gram_ref.data_size =
            (gram_buf.len() - size_of_align_data_gram::<MessageGram>()) as u32;
        data_gram_ref
    }
    //
    pub fn get_message_gram_by_u8(gram_buf: &[u8]) -> &MessageGram {
        let data_gram_ref: &MessageGram = get_gram_by_u8::<MessageGram>(gram_buf); //unsafe {& *(gram_buf[0..].as_ptr() as *const DataGram)};
        data_gram_ref
    }
    //
    pub fn create_message_gram_from_msg_u8<'a>(
        gram_buf: &'a mut [u8],
        value: &'a MessageGram,
    ) -> &'a mut MessageGram {
        #[allow(unused_mut)]
        let mut message: &mut MessageGram = get_mut_gram_by_u8::<MessageGram>(gram_buf); //unsafe {&mut *(gram_buf[0..].as_mut_ptr() as *mut DataGram)};
        unsafe {
            std::ptr::copy_nonoverlapping(
                value, message, 1, // 1 表示复制一个元素
            );
        }
        message.set_bitcomm(value.bitcomm());
        message.set_command(value.command() | BitCommand::RESP_MASK);
        message.set_data_size(gram_buf.len() as u32);

        message
    }
}

use crate::object::client::ClientID;
use crate::object::gram::{BitCommand, BitcommFlag, BitcommVersion, MessageType};
use crate::object::{DeviceID, MessageID};
use getset::{CopyGetters, Setters};

use super::{get_gram_by_u8, get_mut_gram_by_u8, size_of_align_data_gram};

#[repr(C)] // 与C语言兼容
#[derive(Debug, Clone, Copy, CopyGetters, Setters)]
pub struct ReceiptGram {
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
    #[getset(set = "pub", get_copy = "pub")]
    data_time: u32, // 时间戳
}

impl ReceiptGram {

    pub fn get_receipt_gram_key(&self) -> u128 {
        let num_u64_1 = self.sender().get_guid(); //((self.sender().planet().bits() as u64) << 32) | (self.sender().object() as u64);
        let num_u64_2 = ((self.receiver().object() as u64) << 32) | (self.message_id() as u64);
        ((num_u64_1 as u128) << 64) | (num_u64_2 as u128)
    }

    //
    pub fn get_bitcomm_flag_by_u8(gram_buf: &[u8]) -> &BitcommFlag {
        #[allow(unused_mut)]
        let bitcomm_flag_ref: &BitcommFlag = get_gram_by_u8::<BitcommFlag>(gram_buf); 
        bitcomm_flag_ref
    }
    // 
    pub fn is_receipt(data_array: &[u8]) -> bool {
        if data_array.len() == Self::get_size() {
            let bytes: [u8; 4] = [data_array[0], data_array[1], data_array[2], data_array[3]]; //[b0, b1, b2, b3];
            // 将字节数组转换为u32
            let value = u32::from_le_bytes(bytes);
            value == BitcommFlag::BITCOMM_RECEIPT.bits()
        } else {
            false
        }
    }
    // 
    pub fn get_receipt_gram_by_u8(gram_buf: &[u8]) -> &ReceiptGram {
        #[allow(unused_mut)]
        let data_gram_head_ref: &ReceiptGram = get_gram_by_u8::<ReceiptGram>(gram_buf);
        data_gram_head_ref
    }
    // 
    pub fn create_receipt_gram_by_mut_vec8(byte_array: &mut Vec<u8>) -> &mut ReceiptGram {
        let gram_buf: &mut [u8] = byte_array.as_mut_slice();
        Self::create_receipt_gram_by_mut_u8(gram_buf)
    }
    // 
    pub fn create_receipt_gram_by_mut_u8(gram_buf: &mut [u8]) -> &mut ReceiptGram {
        #[allow(unused_mut)]
        let mut data_gram_ref: &mut ReceiptGram = get_mut_gram_by_u8::<ReceiptGram>(gram_buf);
        data_gram_ref.set_bitcomm(BitcommFlag::BITCOMM_RECEIPT);
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
        let mut vec_u8: Vec<u8> = vec![0x00; data_size + size_of_align_data_gram::<ReceiptGram>()];
        vec_u8
    }
    // 
    pub fn create_receipt_gram_from_gram<'a>(
        buf: &'a mut [u8],
        value: &'a ReceiptGram,
    ) -> &'a mut ReceiptGram {
        let receipt = Self::create_receipt_gram_by_mut_u8(buf);
        unsafe {
            std::ptr::copy_nonoverlapping(
                value, receipt, 1, // 1 表示复制一个元素
            );
        }
        receipt.set_bitcomm(value.bitcomm());
        receipt.set_command(value.command() | BitCommand::RESP_MASK);
        receipt
    }
}

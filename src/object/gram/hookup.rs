// use crate::object::client::ClientID;
use crate::object::{gram::{
    get_gram_by_u8, get_mut_gram_by_u8, size_of_align_data_gram, BitCommand, BitcommFlag,
    BitcommVersion, MessageType, ReturnCode,
}, MessageID};
use getset::{CopyGetters, Setters};

#[repr(C)] // 与C语言兼容
#[derive(Debug, Clone, Copy, CopyGetters, Setters)]
pub struct S2SHookupGram {
    #[getset(set = "pub", get_copy = "pub")]
    bitcomm: BitcommFlag,
    #[getset(set = "pub", get_copy = "pub")]
    version: BitcommVersion,
    #[getset(set = "pub", get_copy = "pub")]
    command: BitCommand,
    #[getset(set = "pub", get_copy = "pub")]
    send_server_id: u32,
    #[getset(set = "pub", get_copy = "pub")]
    recv_server_id: u32,
    #[getset(set = "pub", get_copy = "pub")]
    message_type: MessageType,
    #[getset(set = "pub", get_copy = "pub")]
    message_id: MessageID,
    #[getset(set = "pub", get_copy = "pub")]
    return_code: ReturnCode,
    #[getset(set = "pub", get_copy = "pub")]
    data_time: u32, // 时间戳
}

impl S2SHookupGram {
    //
    pub fn is_hookup(data_array: &[u8]) -> bool {
        if data_array.len() == Self::get_size() {
            let bytes: [u8; 4] = [data_array[0], data_array[1], data_array[2], data_array[3]];
            let value = u32::from_le_bytes(bytes);
            value == BitcommFlag::BITCOMM_HOOKUP.bits()
        } else {
            false
        }
    }
    //
    pub fn get_hookup_gram_by_u8(gram_buf: &[u8]) -> &S2SHookupGram {
        #[allow(unused_mut)]
        let data_gram_head_ref: &S2SHookupGram = get_gram_by_u8::<S2SHookupGram>(gram_buf);
        data_gram_head_ref
    }
    //
    pub fn create_hookup_gram_by_mut_vec8(byte_array: &mut Vec<u8>) -> &mut S2SHookupGram {
        let gram_buf: &mut [u8] = byte_array.as_mut_slice();
        Self::create_hookup_gram_by_mut_u8(gram_buf)
    }
    //
    pub fn create_hookup_gram_by_mut_u8(gram_buf: &mut [u8]) -> &mut S2SHookupGram {
        #[allow(unused_mut)]
        let mut data_gram_ref: &mut S2SHookupGram = get_mut_gram_by_u8::<S2SHookupGram>(gram_buf); //unsafe {&mut *(gram_buf[0..].as_mut_ptr() as *mut DataGram)};
                                                                                                   // 设置结构体标志
        // data_gram_ref.data_size =
            // (gram_buf.len() - size_of_align_data_gram::<S2SHookupGram>()) as u32;
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
        let mut vec_u8: Vec<u8> =
            vec![0x00; data_size + size_of_align_data_gram::<S2SHookupGram>()];
        vec_u8
    }
    //
    pub fn create_hookup_gram_from_gram<'a>(
        buf: &'a mut [u8],
        value: &'a S2SHookupGram,
    ) -> &'a mut S2SHookupGram {
        let command = Self::create_hookup_gram_by_mut_u8(buf);
        unsafe {
            std::ptr::copy_nonoverlapping(
                value, command, 1, // 1 表示复制一个元素
            );
        }
        command.set_bitcomm(value.bitcomm());
        command.set_command(value.command() | BitCommand::RESP_MASK);
        command
    }
}

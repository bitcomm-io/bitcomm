
// 获取设备的GUID
pub fn get_machine_guid() -> Option<String> {
    //
    if let Ok(name) = machine_uid::get() {
        Some(name)
    } else {
        None
    }
}
// 有可能会产生碰撞，概率比较小
pub fn get_md5_u32id(long_string:&str) -> u32 {
    // 使用MD5哈希函数
    let digest = md5::compute(long_string);
    // 将MD5哈希值转换为一个32位无符号整数
    u32::from_le_bytes(digest[0..4].try_into().unwrap())
}
    

    
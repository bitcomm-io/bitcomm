use getset::{Getters, Setters};

use crate::object::client::ClientID;

#[repr(C)] // 与C语言兼容
#[derive(Debug, Clone, Getters, Setters)]
pub struct ClientGroup {
    #[getset(set = "pub", get = "pub")]
    group_id: ClientID, // 组ID
    #[getset(set = "pub", get = "pub")]
    client_list: Vec<ClientID>, // 用户列表
}

impl ClientGroup {
    pub fn new(group_id: ClientID) -> Self {
        ClientGroup {
            group_id,
            client_list: Vec::new(),
        }
    }
}

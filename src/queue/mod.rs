// pub mod grambuffer;

use std::sync::Arc;

// use btcmbase::datagram::MessageDataGram;
use bytes::Bytes;
use getset::{CopyGetters, Setters};
use tokio::sync::{mpsc::{self, Receiver, Sender}, RwLock};
// use tokio::sync::{mpsc::{self, Receiver, Sender}, Mutex};

use crate::object::gram::{hookup::S2SHookupGram, message::MessageGram, receipt::ReceiptGram};

use std::collections::{HashMap, VecDeque};


/// 默认事件队列长度。
#[warn(dead_code)]
pub static EVENT_QUEUE_LEN: usize = 1024;

#[allow(dead_code)]
#[derive(Debug, CopyGetters, Setters)]
pub struct BitcommGramQueue {
    // 队列大小
    #[getset(set = "pub", get = "pub")]
    queue_size  :   usize,
    // 队列发送端
    // #[getset(set = "pub", get = "pub")]
    sender      :   Sender<GramEvent>,
    // 队列接收端。
    // #[getset(set = "pub", get = "pub")]
    receiver    :   Arc<RwLock<Receiver<GramEvent>>>,
}

impl BitcommGramQueue {
    //
    pub fn new(queue_size:usize) -> Self {
        let (s, r) = mpsc::channel::<GramEvent>(queue_size);
        BitcommGramQueue {
            queue_size,
            sender      : s,
            receiver    : Arc::new(RwLock::new(r)),
        }
    }
    //
    pub fn get_sender(&self) -> &Sender<GramEvent> {
        &self.sender
    }
    /// 获取消息接收端。
    pub fn get_receiver(&self) -> &Arc<RwLock<Receiver<GramEvent>>> {
        &self.receiver
    }
}

/// 事件类型枚举。
#[allow(dead_code)]
pub enum GramEvent {
    MessagGramEvent {
        data_buff: Arc<Bytes>,
        data_gram: Arc<MessageGram>,
    },
    ReceiptGramEvent {
        data_buff: Arc<Bytes>,
        data_gram: Arc<ReceiptGram>,
    },
    S2SHookupGramEvent {
        data_buff: Arc<Bytes>,
        data_gram: Arc<S2SHookupGram>,
    },
}

// 定义一个结构体来表示缓冲池
#[derive(Debug, CopyGetters, Setters)]
pub struct GramBufferPool {
    #[getset(set = "pub", get = "pub")]
    buffer: VecDeque<u128>,
    #[getset(set = "pub", get = "pub")]
    hashsto: HashMap<u128, Arc<Bytes>>,
    #[getset(set = "pub", get = "pub")]
    capacity: usize,
}

impl GramBufferPool {
    // 创建一个新的缓冲池实例
    pub fn new(capacity: usize) -> Self {
        GramBufferPool {
            buffer: VecDeque::with_capacity(capacity),
            hashsto: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    // 向缓冲池中添加数据
    pub fn push(&mut self, data: Arc<Bytes>, key: u128) {
        if self.buffer.len() >= self.capacity {
            // 如果缓冲池已满，先删除最早添加的数据
            self.buffer.pop_front();
        }
        // let key = gram.get_message_gram_key();
        self.buffer.push_back(key);
        self.hashsto.insert(key, data);
    }

    // 从缓冲池中获取数据（按照先进先出的顺序）
    pub fn pop(&mut self) -> Option<Arc<Bytes>> {
        if let Some(key) = self.buffer.pop_front() {
            self.hashsto.remove(&key)
            // self.hashsto.get(&key).map(|x| x.clone())
        } else {
            None
        }
    }
    pub fn remove(&mut self, key: &u128) -> Option<Arc<Bytes>> {
        self.hashsto.remove(key)
    }
    pub fn get_buffer_size(&self) -> usize {
        self.buffer.len()
    }
}



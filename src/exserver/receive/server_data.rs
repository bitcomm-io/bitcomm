use std::sync::Arc;

use bytes::Bytes;
use s2n_quic::stream::{ ReceiveStream, SendStream };
use tokio::sync::{Mutex, RwLock};
// use tracing::error;

use crate::object::gram::hookup::S2SHookupGram;
use crate::object::gram::reply::ReplyGram;
use crate::queue::BitcommGramQueue;
use crate::exserver::{ hookup, message, receipt, reply, EXServer, S2SMSPType };
use crate::object::gram::message::MessageGram;
use crate::object::gram::receipt::ReceiptGram;
use crate::object::gram::BitcommDataGram;

/// 接收数据报文并进行处理。
///
/// # 参数
/// * `receive_stream` - 用于接收数据的流。
/// * `stm0` - 发送数据的流的互斥锁包装的`Arc`。
/// * `msg_queue` - 消息队列的`Arc`。
/// * `rct_queue` - 回执队列的`Arc`。
/// * `s2smsp_type` - 服务器到服务器消息处理类型。
///
/// # 示例
///
/// ```
/// // 示例代码展示如何调用此函数。
/// ```
pub async fn receive_data_gram(
    rece_stream: Arc<Mutex<ReceiveStream>>,
    send_stream: Arc<Mutex<SendStream>>,
    msg_queue: Arc<BitcommGramQueue>,
    rct_queue: Arc<BitcommGramQueue>,
    s2smsp_type: S2SMSPType,
    exserver:Arc<RwLock<EXServer>>,
) {
    // 接收数据并处理
    while let Ok(Some(data_buff)) = rece_stream.lock().await.receive().await {
        let rc_data_buff = Arc::new(data_buff);
        // 准备数据缓冲区
        if let Some(inner_data_gram) = prepare_data_buffer(rc_data_buff.clone()) {
            let rc_data_gram = Arc::new(inner_data_gram);
            // 发送回执，服务器已收到，正在处理,如果是返回的信息，则无需再回执
            send_receipt_2_client(rc_data_gram.clone(), send_stream.clone()).await;
            // 处理命令
            process_server_command(
                    &rc_data_gram,
                    &send_stream,
                    &s2smsp_type,
                    &exserver
                ).await;
            // 处理消息
            process_bitcomm_gram(
                    &rc_data_gram,
                    &send_stream,
                    &msg_queue,
                    &rct_queue
                ).await;
        }
    }
}

/// 准备数据缓冲区以解析为数据报文。
///
/// 此函数接收一个`Arc<Bytes>`类型的数据缓冲区，并尝试将其解析为不同类型的`BitcommDataGram`。
/// 它首先检查数据是否符合特定的报文格式，然后根据格式创建相应类型的`BitcommDataGram`实例。
///
/// # 参数
/// * `data_buff` - 包含待解析数据的`Arc<Bytes>`。
///
/// # 返回
/// 如果数据缓冲区成功解析为已知的数据报文格式，则返回`Some(BitcommDataGram)`。
/// 如果数据不符合任何已知格式，则返回`None`。
///
/// # 示例
/// ```rust
/// let data = Arc::new(Bytes::from_static(b"some data"));
/// let data_gram = prepare_data_buffer(data);
/// match data_gram {
///     Some(gram) => println!("Data gram parsed: {:?}", gram),
///     None => println!("Data buffer did not match any known data gram format"),
/// }
/// ```
pub fn prepare_data_buffer(data_buff: Arc<Bytes>) -> Option<BitcommDataGram> {
    // 获取数据缓冲区的引用
    let bts = data_buff.as_ref();

    // 检查数据是否为回复报文格式
    if ReplyGram::is_reply(bts) {
        // 解析回复报文并创建`BitcommDataGram::Reply`
        let reply_gram = ReplyGram::get_reply_gram_by_u8(bts);
        return Some(BitcommDataGram::Reply {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*reply_gram),
        });
    }
    // 检查数据是否为挂钩报文格式
    if S2SHookupGram::is_hookup(bts) {
        // 解析挂钩报文并创建`BitcommDataGram::S2SHookup`
        let hookup_gram = S2SHookupGram::get_hookup_gram_by_u8(bts);
        return Some(BitcommDataGram::S2SHookup {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*hookup_gram),
        });
    }
    // 检查数据是否为回执报文格式
    if ReceiptGram::is_receipt(bts) {
        // 解析回执报文并创建`BitcommDataGram::Receipt`
        let receipt_gram = ReceiptGram::get_receipt_gram_by_u8(bts);
        return Some(BitcommDataGram::Receipt {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*receipt_gram),
        });
    }
    // 检查数据是否为消息报文格式
    if MessageGram::is_message(bts) {
        // 解析消息报文并创建`BitcommDataGram::Message`
        let message_gram = MessageGram::get_message_gram_by_u8(bts);
        return Some(BitcommDataGram::Message {
            data_buff: data_buff.clone(),
            data_gram: Arc::new(*message_gram),
        });
    }
    // 如果数据不符合任何已知格式，返回`None`
    None
}

/// 向客户端发送回执。
///
/// # 参数
/// * `data` - 要发送的数据报文的`Arc`。
/// * `_stm` - 发送数据的流的互斥锁包装的`Arc`。
///
/// # 示例
///
/// ```
/// // 示例代码展示如何调用此函数。
/// ```
#[allow(unused_variables)]
pub async fn send_receipt_2_client(data: Arc<BitcommDataGram>, _stm: Arc<Mutex<SendStream>>) {
    match data.as_ref() {
        // 只有收到的是message类型的报文才发送
        BitcommDataGram::Message { data_buff, data_gram } => {}
        _ => {}
    }
}
/// 处理服务器命令。
///
/// # 参数
/// * `data` - 要处理的数据报文的`Arc`。
/// * `stm` - 发送数据的流的互斥锁包装的`Arc`。
/// * `s2smsp_type` - 服务器到服务器消息处理类型。
///
/// # 返回
/// 返回`Result<Arc<BitcommDataGram>, DataGramError>`，表示处理结果。
///
/// # 示例
///
/// ```
/// // 示例代码展示如何调用此函数。
/// ```
#[allow(unused_variables)]
pub async fn process_server_command(
    data: &Arc<BitcommDataGram>,
    stm: &Arc<Mutex<SendStream>>,
    s2smsp_type: &S2SMSPType,
    exserver:&Arc<RwLock<EXServer>>,
) {
    match data.as_ref() {
        // 处理命令
        BitcommDataGram::S2SHookup { data_buff, data_gram } =>
            hookup::process_hookup(data_buff, data_gram, stm, s2smsp_type,exserver).await,
        // 应答信息包
        BitcommDataGram::Reply { data_buff, data_gram } =>
            reply::process_reply(data_buff, data_gram, stm, s2smsp_type,exserver).await,
        _ => {}
    }
}
/// 处理接收到的Bitcomm数据报文。
///
/// 根据数据报文的类型，调用相应的处理函数。对于消息类型的数据报文，调用`process_message_gram`函数；
/// 对于回执类型的数据报文，调用`process_receipt_gram`函数。
///
/// # 参数
/// * `data` - 接收到的数据报文的`Arc`。
/// * `stm` - 发送数据流的互斥锁包装的`Arc`。
/// * `msg_queue` - 消息队列的`Arc`。
/// * `rct_queue` - 回执队列的`Arc`。
///
/// # 返回
/// 返回`Result<Arc<BitcommDataGram>, DataGramError>`，表示处理结果。
///
/// # 示例
///
/// ```
/// // 示例代码展示如何调用此函数。
/// ```
#[allow(unused_variables)]
pub async fn process_bitcomm_gram(
    data: &Arc<BitcommDataGram>,
    stm: &Arc<Mutex<SendStream>>,
    msg_queue: &Arc<BitcommGramQueue>,
    rct_queue: &Arc<BitcommGramQueue>
) {
    match data.as_ref() {
        BitcommDataGram::Message { data_buff, data_gram } =>
            message::process_message_gram(data_buff, data_gram, stm, &msg_queue).await,
        BitcommDataGram::Receipt { data_buff, data_gram } =>
            receipt::process_receipt_gram(data_buff, data_gram, stm, &rct_queue).await,
        //
        _ => {}
    }
    // Ok(data)
}

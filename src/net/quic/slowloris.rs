
use s2n_quic::provider::{
    event,
    event::{events, supervisor, ConnectionMeta, Timestamp},
};
use std::time::Duration;

/// 在低吞吐量连接关闭之前，同时活动的最大连接数。
const CONNECTION_COUNT_THRESHOLD: usize = 1000;
/// 连接必须维持的最小吞吐量，以字节每秒计
const MIN_THROUGHPUT: usize = 500;

/// 定义包含要跟踪的每个连接的任何连接状态的Connection Context。
/// 对于此示例，我们需要跟踪传输的字节数和传输字节数上次总计的时间。
#[derive(Debug, Clone)]
pub struct MyConnectionContext {
    transferred_bytes: usize,
    last_update: Timestamp,
}

/// 定义一个包含要在所有连接上跟踪的任何状态的结构。
/// 对于此示例，没有额外的状态需要跟踪，因此该结构为空。
#[derive(Default)]
pub struct MyConnectionSupervisor;

/// 为结构实现`event::Subscriber`特性。必须实现`create_connection_context`方法，
/// 以初始化每个连接的Connection Context。其他方法可以根据需要实现。
impl event::Subscriber for MyConnectionSupervisor {
    type ConnectionContext = MyConnectionContext;

    /// 初始化Connection Context，该上下文传递给`supervisor_timeout`和
    /// `on_supervisor_timeout`方法，以及每个与连接相关的事件。
    fn create_connection_context(
        &mut self,
        meta: &events::ConnectionMeta,
        _info: &events::ConnectionInfo,
    ) -> Self::ConnectionContext {
        MyConnectionContext {
            transferred_bytes: 0,
            last_update: meta.timestamp,
        }
    }

    /// 实现`supervisor_timeout`以定义`on_supervisor_timeout`将在何时调用。
    /// 对于此示例，使用常量1秒，但此值可以根据连接随时间变化或基于连接调整。
    fn supervisor_timeout(
        &mut self,
        _conn_context: &mut Self::ConnectionContext,
        _meta: &ConnectionMeta,
        _context: &supervisor::Context,
    ) -> Option<Duration> {
        Some(Duration::from_secs(1))
    }

    /// 实现`on_supervisor_timeout`以定义`supervisor_timeout`过期时应采取的连接操作。
    /// 对于此示例，如果打开的连接数大于`CONNECTION_COUNT_THRESHOLD`且连接的吞吐量
    /// 自上次`supervisor_timeout`以来下降到`MIN_THROUGHPUT`以下，则立即关闭连接
    /// (`supervisor::Outcome::ImmediateClose`)。
    fn on_supervisor_timeout(
        &mut self,
        conn_context: &mut Self::ConnectionContext,
        meta: &ConnectionMeta,
        context: &supervisor::Context,
    ) -> supervisor::Outcome {
        if !context.is_handshaking && context.connection_count > CONNECTION_COUNT_THRESHOLD {
            let elapsed_time = meta.timestamp.duration_since_start()
                - conn_context.last_update.duration_since_start();

            // 计算吞吐量，单位为每秒字节数
            let throughput =
                (conn_context.transferred_bytes as f32 / elapsed_time.as_secs_f32()) as usize;

            if throughput < MIN_THROUGHPUT {
                // 立即关闭连接，而不通知对等方
                return supervisor::Outcome::ImmediateClose {
                    reason: "Connection throughput was below MIN_THROUGHPUT",
                };
            }
        }

        // 更新`last_update`时间戳并重置传输字节
        conn_context.last_update = meta.timestamp;
        conn_context.transferred_bytes = 0;

        // 允许连接继续
        supervisor::Outcome::Continue
    }

    /// 实现`on_tx_stream_progress`，在传出流上取得进展时通知每次。
    fn on_tx_stream_progress(
        &mut self,
        context: &mut Self::ConnectionContext,
        _meta: &events::ConnectionMeta,
        event: &events::TxStreamProgress,
    ) {
        context.transferred_bytes += event.bytes;
    }

    /// 实现`on_rx_stream_progress`，在传入流上取得进展时通知每次。
    fn on_rx_stream_progress(
        &mut self,
        context: &mut Self::ConnectionContext,
        _meta: &events::ConnectionMeta,
        event: &events::RxStreamProgress,
    ) {
        context.transferred_bytes += event.bytes;
    }
}

use crate::errors::errors::{AuthenticationResult, RtspError};
use crate::rtsp::client::RtspClient;
use lazy_static::lazy_static;
use log::{debug, error, info, trace};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, mpsc};

// 定义消息类型
pub enum RtspMessage {
    // 认证请求
    AuthRequest {
        username: String,
        password: String,
        rtsp_url: String,
        response_tx: mpsc::Sender<Result<AuthenticationResult, RtspError>>,
    },
    // 停止工作线程
    Stop,
}

// RTSP工作线程
pub struct RtspWorker {
    // 消息接收通道
    receiver: Arc<Mutex<mpsc::Receiver<RtspMessage>>>,
    // 工作线程句柄
    handle: Option<tokio::task::JoinHandle<()>>,
    // 工作线程ID
    id: u32,
}

impl RtspWorker {
    // 创建新的工作线程
    pub fn new(id: u32) -> (Self, mpsc::Sender<RtspMessage>) {
        let (sender, receiver) = mpsc::channel(100);
        let worker = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            handle: None,
            id,
        };
        (worker, sender)
    }

    // 启动工作线程
    pub fn start(&mut self) {
        let receiver = self.receiver.clone();
        let id = self.id;
        self.handle = Some(tokio::spawn(async move {
            debug!("RTSP worker {} started", id);
            while let Some(msg) = receiver.lock().await.recv().await {
                match msg {
                    RtspMessage::AuthRequest {
                        username,
                        password,
                        rtsp_url,
                        response_tx,
                    } => {
                        let start_time = Instant::now();
                        debug!(
                            "Worker {} processing auth request for {}@{}:123",
                            id, username, rtsp_url
                        );
                        // 执行认证
                        let result = async {
                            let client = RtspClient::new(&username, &password);
                            client.describe(&rtsp_url).await
                        }
                        .await;
                        let duration = start_time.elapsed();
                        trace!("Worker {} completed auth request in {:?}", id, duration);

                        // 发送结果
                        if let Err(e) = response_tx.send(result).await {
                            error!("Failed to send authentication result: {:?}", e);
                        }
                    }
                    RtspMessage::Stop => {
                        debug!("RTSP worker {} stopping", id);
                        break;
                    }
                }
            }
        }));
    }

    // 停止工作线程
    pub async fn stop(mut self) {
        if let Some(handle) = self.handle.take() {
            // 等待工作线程结束
            if let Err(e) = handle.await {
                error!("RTSP worker {} panicked: {:?}", self.id, e);
            }
        }
    }
}

// 工作线程管理器 - 多线程工作池模式
pub struct RtspWorkerManager {
    // 所有工作线程的发送器
    senders: Arc<Mutex<Vec<mpsc::Sender<RtspMessage>>>>,
    // 工作线程池
    workers: Arc<Mutex<Vec<Option<RtspWorker>>>>,
    // 下一个要使用的工作线程索引
    next_worker_index: Arc<Mutex<usize>>,
    // 是否正在运行
    is_running: Arc<Mutex<bool>>,
    // 工作线程数量
    worker_count: u32,
}

impl RtspWorkerManager {
    // 创建新的管理器
    pub fn new(worker_count: Option<u32>) -> Self {
        // 默认使用CPU核心数作为工作线程数
        let count = worker_count.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(4)
        });
        info!("RTSP worker pool with {} workers created", count);

        let mut senders = Vec::new();
        let mut workers = Vec::new();

        // 创建指定数量的工作线程
        for i in 0..count {
            let (worker, sender) = RtspWorker::new(i);
            senders.push(sender);
            workers.push(Some(worker));
        }

        Self {
            senders: Arc::new(Mutex::new(senders)),
            workers: Arc::new(Mutex::new(workers)),
            next_worker_index: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(false)),
            worker_count: count,
        }
    }

    // 启动所有工作线程
    pub async fn start(&self) {
        let mut running = self.is_running.lock().await;
        if *running {
            debug!("RTSP worker pool is already running");
            return;
        }

        let mut workers = self.workers.lock().await;
        for worker in workers.iter_mut() {
            if let Some(w) = worker {
                w.start();
            }
        }

        *running = true;
        info!(
            "RTSP worker pool with {} workers started",
            self.worker_count
        );
    }

    // 轮询选择下一个工作线程
    async fn get_next_worker_index(&self) -> usize {
        let senders_len = self.senders.lock().await.len();
        let mut index = self.next_worker_index.lock().await;
        let current = *index;
        *index = (current + 1) % senders_len;
        trace!("Selected worker index: {}", current);
        current
    }

    // 发送认证请求 - 使用轮询方式分发到工作线程
    pub async fn auth_request(
        &self,
        username: &str,
        password: &str,
        rtsp_url: &str,
    ) -> Result<Option<(String, String)>, RtspError> {
        let (response_tx, mut response_rx) = mpsc::channel(10);

        let sender = {
            // 注意，以下代码在异步上下文中运行，不能直接使用阻塞的锁操作
            // 也不能使用mutex的lock方法，因为它会阻塞当前线程
            // 所以这里使用了senders的生命周期，确保在{}中完成对sender的获取。

            // 获取下一个工作线程的索引
            let index = self.get_next_worker_index().await;
            // 获取对应的发送器
            let senders = self.senders.lock().await;
            let sender = senders
                .get(index)
                .ok_or_else(|| RtspError::ProtocolError("No available workers".to_string()))?;
            debug!(
                "RTSP worker {} selected for auth request to {}",
                index, rtsp_url
            );
            sender.clone()
        };
        // 发送认证请求
        // 发送给后台的worker进行处理
        sender.send(RtspMessage::AuthRequest {
                username: username.to_string(),
                password: password.to_string(),
                rtsp_url: rtsp_url.to_string(),
                response_tx, //用于将验证结果传回
            })
            .await
            .map_err(|e| {
                RtspError::ProtocolError(format!("Failed to send auth request: {:?}", e))
            })?;

        // 等待响应
        match response_rx.recv().await {
            Some(result) => match result {
                Ok(AuthenticationResult::Success) => {
                    Ok(Some((username.to_string(), password.to_string())))
                }
                Ok(AuthenticationResult::NoAuthenticationRequired) => {
                    Ok(Some(("".to_string(), "".to_string())))
                }
                Ok(AuthenticationResult::Failed) => Ok(None),
                Err(e) => Err(RtspError::AuthenticationError(format!(
                    "Authentication failed: {:?} for {}",
                    e, rtsp_url
                ))),
            },
            None => Err(RtspError::ProtocolError(
                "No response from RTSP worker".to_string(),
            )),
        }
    }

    // 停止所有工作线程
    pub async fn stop(&self) {
        // 向所有工作线程发送停止消息
        let senders = self.senders.lock().await;
        for sender in senders.iter() {
            if let Err(e) = sender.send(RtspMessage::Stop).await {
                error!("Failed to send stop message: {:?}", e);
            }
        }

        // 清理所有工作线程
        let mut workers = self.workers.lock().await;
        for worker in workers.iter_mut() {
            if let Some(w) = worker.take() {
                w.stop().await;
            }
        }

        *self.is_running.lock().await = false;
        debug!("RTSP worker pool stopped");
    }
}

// 创建全局静态实例
lazy_static! {
    pub static ref RTSP_WORKER_MANAGER: RtspWorkerManager = RtspWorkerManager::new(None);
}

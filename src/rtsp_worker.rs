use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use log::{info, error, debug};
use crate::client::RtspClient;
use crate::error::{AuthenticationResult, RtspError};

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
}

impl RtspWorker {
    // 创建新的工作线程
    pub fn new() -> (Self, mpsc::Sender<RtspMessage>) {
        let (sender, receiver) = mpsc::channel(100);
        let worker = Self {
            receiver: Arc::new(Mutex::new(receiver)),
            handle: None,
        };
        (worker, sender)
    }

    // 启动工作线程
    pub fn start(&mut self) {
        let receiver = self.receiver.clone();
        self.handle = Some(tokio::spawn(async move {
            while let Some(msg) = receiver.lock().await.recv().await {
                match msg {
                    RtspMessage::AuthRequest {
                        username,
                        password,
                        rtsp_url,
                        response_tx,
                    } => {
                        // 执行认证
                        let result = async {
                            let client = RtspClient::new(&username, &password);
                            client.describe(&rtsp_url).await
                        }.await;

                        // 发送结果
                        if let Err(e) = response_tx.send(result).await {
                            error!("Failed to send authentication result: {:?}", e);
                        }
                    },
                    RtspMessage::Stop => {
                        debug!("RTSP worker stopping");
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
                error!("RTSP worker panicked: {:?}", e);
            }
        }
    }
}

// 工作线程管理器 - 单例模式
pub struct RtspWorkerManager {
    sender: mpsc::Sender<RtspMessage>,
    worker: Arc<Mutex<Option<RtspWorker>>>,
    is_running: Arc<Mutex<bool>>,
}

impl RtspWorkerManager {
    // 创建新的管理器
    pub fn new() -> Self {
        let (worker, sender) = RtspWorker::new();
        Self {
            sender,
            worker: Arc::new(Mutex::new(Some(worker))),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    // 启动工作线程
    pub async fn start(&self) {
        let mut running = self.is_running.lock().await;
        if *running {
            debug!("RTSP worker is already running");
            return;
        }

        let mut worker = self.worker.lock().await;
        if let Some(ref mut w) = *worker {
            w.start();
            *running = true;
            debug!("RTSP worker started");
        }
    }

    // 发送认证请求
    pub async fn auth_request(
        &self,
        username: &str,
        password: &str,
        rtsp_url: &str,
    ) -> Result<Option<(String, String)>, RtspError> {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        // 发送认证请求
        self.sender
            .send(RtspMessage::AuthRequest {
                username: username.to_string(),
                password: password.to_string(),
                rtsp_url: rtsp_url.to_string(),
                response_tx,
            })
            .await
            .map_err(|e| RtspError::ProtocolError(format!("Failed to send auth request: {:?}", e)))?;

        // 等待响应
        match response_rx.recv().await {
            Some(result) => {
                match result {
                    Ok(AuthenticationResult::Success) => {
                        Ok(Some((username.to_string(), password.to_string())))
                    },
                    Ok(AuthenticationResult::NoAuthenticationRequired) => {
                        Ok(None)
                    },
                    Ok(AuthenticationResult::Failed) => {
                        Err(RtspError::AuthenticationError("Authentication failed".to_string()))
                    },
                    Err(e) => Err(e),
                }
            },
            None => Err(RtspError::ProtocolError("No response from RTSP worker".to_string())),
        }
    }

    // 停止工作线程
    pub async fn stop(&self) {
        // 发送停止消息
        if let Err(e) = self.sender.send(RtspMessage::Stop).await {
            error!("Failed to send stop message: {:?}", e);
        }

        // 清理工作线程
        let mut worker = self.worker.lock().await;
        if let Some(w) = worker.take() {
            w.stop().await;
            *self.is_running.lock().await = false;
            debug!("RTSP worker stopped");
        }
    }
}

// 全局静态工作线程管理器
lazy_static! {
    pub static ref RTSP_WORKER_MANAGER: RtspWorkerManager = RtspWorkerManager::new();
}
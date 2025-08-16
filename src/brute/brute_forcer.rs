use crate::brute::task_manager::TaskManager;
use crate::errors::errors::RtspError;
use crate::iterator::credential_iterator::CredentialIterator;
use crate::iterator::ip_iterator::{IpIterator, IpPortAddr};
use crate::rtsp::rtsp_worker::RTSP_WORKER_MANAGER;
use log::{debug, error, info, trace};
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::Semaphore;
use tokio::time::Instant;

/// 存储找到的RTSP认证凭据信息
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FoundCredential {
    pub ip_port: IpPortAddr,
    pub username: String,
    pub password: String,
}

impl Display for FoundCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Found credentials for {}: {}:{}",
            self.ip_port, self.username, self.password
        )
    }
}

/// 暴力枚举器 - 负责创建和管理暴力破解任务
pub struct BruteForcer {
    credential_iterator: CredentialIterator,
    ip_iterator: IpIterator,
    max_concurrent: u32,
    found_credentials: Arc<Mutex<HashSet<FoundCredential>>>, // 跟踪已找到的认证凭据
    task_manager: TaskManager,
}

impl BruteForcer {
    pub fn new() -> Self {
        const DEFAULT_MAX_CONCURRENT: u32 = 5;
        BruteForcer {
            credential_iterator: CredentialIterator::new(vec![], vec![]),
            ip_iterator: IpIterator::new(vec![]),
            max_concurrent: DEFAULT_MAX_CONCURRENT,
            found_credentials: Arc::new(Mutex::new(HashSet::new())),
            task_manager: TaskManager::new(DEFAULT_MAX_CONCURRENT),
        }
    }

    /// 设置凭据迭代器
    pub fn with_cred_iterator(mut self, credential_iterator: CredentialIterator) -> Self {
        self.credential_iterator = credential_iterator;
        self
    }

    /// 设置IP迭代器
    pub fn with_ip_iterator(mut self, ip_iterator: IpIterator) -> Self {
        self.ip_iterator = ip_iterator;
        self
    }

    /// 设置最大并发数
    pub fn with_max_concurrent(mut self, max_concurrent: u32) -> Self {
        self.max_concurrent = max_concurrent;
        self.task_manager = TaskManager::new(max_concurrent);
        self
    }

    /// 尝试单个用户名、密码和URL
    pub async fn try_credentials(
        &self,
        username: &str,
        password: &str,
        ip_port: &IpPortAddr,
    ) -> Result<Option<FoundCredential>, RtspError> {
        let rtsp_url = format!("rtsp://{}:{}", ip_port.ip, ip_port.port);
        debug!(
            "Task started: Scanning {}: {}:{} on thread {:?}",
            rtsp_url,
            username,
            password,
            thread::current().id()
        );

        // 检查IP是否已经找到有效的凭据
        if self.has_valid_credentials_for_ip(ip_port) {
            debug!(
                "Skipping {}:{} as valid credentials already found",
                &ip_port.ip, &ip_port.port
            );
            return Ok(None);
        }

        let start_time = Instant::now();

        match RTSP_WORKER_MANAGER
            .auth_request(username, password, &rtsp_url)
            .await
        {
            Ok(result) => {
                let duration = start_time.elapsed();
                debug!(
                    "Task completed in {:?}: Scanning {}: {}:{}",
                    duration, rtsp_url, username, password
                );

                match result {
                    Some((valid_username, valid_password)) => {
                        let found_cred = FoundCredential {
                            ip_port: ip_port.clone(),
                            username: valid_username.to_string(),
                            password: valid_password.to_string(),
                        };

                        // 添加到已找到凭据集合
                        self.add_found_credential(found_cred.clone());

                        Ok(Some(found_cred))
                    }
                    None => {
                        debug!("Failed attempt: {}:{}", username, password);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                error!("Error during authentication attempt: {:?}", e);
                Err(e)
            }
        }
    }

    /// 添加找到的凭据到集合
    pub fn add_found_credential(&self, credential: FoundCredential) {
        let mut found_credentials = self.found_credentials.lock().unwrap();
        if found_credentials.insert(credential.clone()) {
            info!("{}", credential);
        }
    }

    /// 检查IP是否已经找到有效凭据
    pub fn has_valid_credentials_for_ip(&self, ip: &IpPortAddr) -> bool {
        let found_credentials = self.found_credentials.lock().unwrap();
        found_credentials.iter().any(|cred| cred.ip_port == *ip)
    }

    /// 执行暴力枚举
    pub async fn brute_force(self: Arc<Self>) -> Result<(), RtspError> {
        let start_time = Instant::now();
        info!("Max concurrent attempts: {}", self.max_concurrent);
        info!("Total IPs to scan: {}", self.ip_iterator.clone().count());
        info!(
            "Total credential combinations: {}",
            self.credential_iterator.clone().count()
        );

        // 创建信号量限制并发数
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent as usize));
        let total_tasks =
            self.ip_iterator.clone().count() * self.credential_iterator.clone().count();
        info!("Total tasks to create: {}", total_tasks);
        let mut tasks = Vec::new();

        // 为每个IP和凭据组合创建任务
        for (ip_idx, ip) in self.ip_iterator.clone().enumerate() {
            for (cred_idx, (username, password)) in self.credential_iterator.clone().enumerate() {
                // 提前获取信号量许可
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let this_clone = self.clone();
                let ip_clone = ip.clone();
                let username_clone = username.clone();
                let password_clone = password.clone();
                let task_idx = ip_idx * self.credential_iterator.clone().count() + cred_idx;

                trace!("Creating task {} of {}", task_idx + 1, total_tasks);
                let task = tokio::spawn(async move {
                    let _permit = permit;
                    trace!(
                        "Task {} started on thread {:?}",
                        task_idx + 1,
                        thread::current().id()
                    );
                    let result = this_clone
                        .try_credentials(&username_clone, &password_clone, &ip_clone)
                        .await;
                    trace!("Task {} completed", task_idx + 1);
                    result
                });
                tasks.push(task);
            }
        }

        info!("All tasks created. Waiting for completion...");
        // 等待所有任务完成并处理结果
        self.task_manager
            .process_task_results(tasks, start_time, total_tasks)
            .await;

        Ok(())
    }
}

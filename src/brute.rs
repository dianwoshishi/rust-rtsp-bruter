use crate::credential_iterator::CredentialIterator;
use crate::error::RtspError;
use crate::ip_iterator::{IpIterator, IpPortAddr};
use crate::rtsp_worker::RTSP_WORKER_MANAGER;
use log::{debug, error, info};
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{Semaphore, broadcast::channel as broadcast_channel};
use tokio::time::sleep;

// 存储找到的RTSP认证凭据信息
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

// 暴力枚举器
#[derive(Clone)]
pub struct BruteForcer {
    credential_iterator: CredentialIterator,
    ip_iterator: IpIterator,
    max_concurrent: u32,
    delay: u64,                                              // 每次尝试之间的延迟(毫秒)
    found_credentials: Arc<Mutex<HashSet<FoundCredential>>>, // 跟踪已找到的认证凭据
}

impl BruteForcer {
    pub fn new() -> Self {
        BruteForcer {
            credential_iterator: CredentialIterator::new(vec![], vec![]),
            ip_iterator: IpIterator::new(vec![]),
            max_concurrent: 5, // 默认最大并发数
            delay: 100,        // 默认延迟100ms
            found_credentials: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    // 设置凭据迭代器
    pub fn with_cred_iterator(mut self, credential_iterator: CredentialIterator) -> Self {
        self.credential_iterator = credential_iterator;
        self
    }

    // 设置IP迭代器
    pub fn with_ip_iterator(mut self, ip_iterator: IpIterator) -> Self {
        self.ip_iterator = ip_iterator;
        self
    }

    // 设置最大并发数
    pub fn with_max_concurrent(mut self, max_concurrent: u32) -> Self {
        self.max_concurrent = max_concurrent;
        self
    }

    // 设置延迟
    pub fn with_delay(mut self, delay: u64) -> Self {
        self.delay = delay;
        self
    }

    // 尝试单个用户名、密码和URL
    pub async fn try_credentials(
        &self,
        username: &str,
        password: &str,
        ip_port: &IpPortAddr,
    ) -> Result<Option<FoundCredential>, RtspError> {
        let rtsp_url = format!("rtsp://{}:{}", ip_port.ip, ip_port.port);
        debug!("Scanning {}: {}:{}", rtsp_url, username, password);


        match RTSP_WORKER_MANAGER.auth_request(username, password, &rtsp_url).await {
            Ok(result) => match result {
                Some((valid_username, valid_password)) => {

                    let found_cred = FoundCredential {
                        ip_port: ip_port.clone(),
                        username: valid_username.to_string(),
                        password: valid_password.to_string(),
                    };

                    // 添加到已找到凭据集合
                    self.add_found_credential(found_cred.clone());

                    Ok(Some(found_cred))
                },
                None => {
                    debug!("Failed attempt: {}:{}", username, password);
                    Ok(None)
                }
            },
            Err(e) => {
                error!("Error during authentication attempt: {:?}", e);
                Err(e)
            }
        }
    }

    // 添加找到的凭据到集合
    fn add_found_credential(&self, credential: FoundCredential) {
        let mut found_credentials = self.found_credentials.lock().unwrap();
        if found_credentials.insert(credential.clone()) {
            info!("{}", credential);
            // println!("{}", credential);
        }
    }

    // 检查IP是否已经找到有效凭据
    fn has_valid_credentials_for_ip(&self, ip: &IpPortAddr) -> bool {
        let found_credentials = self.found_credentials.lock().unwrap();
        found_credentials.iter().any(|cred| cred.ip_port == *ip)
    }

    // 为单个IP地址执行暴力破解
    async fn brute_force_single_ip(
        &self,
        ip: IpPortAddr,
        delay: u64,
    ) -> Result<Option<FoundCredential>, RtspError> {
        let (cancel_tx, _) = broadcast_channel::<()>(1);
        let mut cancel_rx = cancel_tx.subscribe();
        let credential_iterator = self.credential_iterator.clone();
        let this_clone = self.clone();

        let mut found_cred: Option<FoundCredential> = None;

        'cred_loop: for (ref username, ref password) in credential_iterator {

            info!("For IP {}: Trying {}:{}", &ip, &username, &password);

            // 为每次迭代克隆必要的变量
            let this_clone_iter = this_clone.clone();
            let ip_clone = ip.clone();
            let delay_clone = delay;

            // 检查是否收到取消信号
            tokio::select! {
                _ = cancel_rx.recv() => {
                    debug!("IP brute force cancelled for {}", ip);
                    break 'cred_loop;
                },
                result = async move {
                    // 检查IP是否已经找到有效的凭据
                    if this_clone_iter.has_valid_credentials_for_ip(&ip_clone) {
                        debug!(
                            "Skipping IP {}:{} as valid credentials already found",
                            &ip_clone.ip, &ip_clone.port

                        );
                        return Ok(None);
                    }

                    // 尝试凭据
                    let result = this_clone_iter.try_credentials(&username, &password, &ip_clone).await;

                    // // 延迟下一次尝试
                    // sleep(Duration::from_millis(delay_clone)).await;

                    result
                } => {
                    match result {
                        Ok(Some(cred)) => {
                            found_cred = Some(cred);
                            // 找到有效凭据，取消当前IP的剩余尝试
                            break 'cred_loop;
                        },
                        Ok(None) => {
                            // 继续尝试下一个凭据
                            debug!("For IP {}: Failed {}:{}", &ip_clone, &username, &password);

                        },
                        Err(e) => {
                            error!("Task failed with error: {:?}", e);
                        }
                    }
                }
            }
        }

        Ok(found_cred)
    }

    // 执行暴力枚举
    pub async fn brute_force(&self) -> Result<(), RtspError> {
        info!("Max concurrent attempts: {}", self.max_concurrent);
        info!("Delay between attempts: {}ms", self.delay);

        // 创建信号量限制并发数
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent as usize));
        let mut ip_tasks = Vec::new();

        // 为每个IP地址创建一个任务
        for ip in self.ip_iterator.clone() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let this_clone = self.clone();
            let delay = self.delay;

            let task = tokio::spawn(async move {
                let _permit = permit; // 保持许可直到任务完成
                this_clone.brute_force_single_ip(ip, delay).await
            });

            ip_tasks.push(task);
        }

        // 等待所有IP任务完成
        self.wait_for_tasks(ip_tasks).await;

        Ok(())
    }

    // 等待所有任务完成并处理结果
    async fn wait_for_tasks(&self, tasks: Vec<tokio::task::JoinHandle<Result<Option<FoundCredential>, RtspError>>>) {
        let mut success_found = false;

        for task in tasks {
            match task.await {
                Ok(Ok(Some(_found_cred))) => {
                    success_found = true;
                },
                Ok(Ok(None)) => {
                    // 该IP未找到有效凭据
                },
                Ok(Err(e)) => {
                    error!("IP task failed with error: {:?}", e);
                },
                Err(e) => {
                    error!("IP task panicked: {:?}", e);
                }
                //todo 错误信息太多，不便于查看探测结果。
                // 可以考虑只打印成功的IP地址

            }
        }

        if success_found {
            info!("Brute force attack completed. Valid credentials found.");
        } else {
            info!("Brute force attack completed. No valid credentials found.");
            println!("Error: No valid credentials found in the provided lists");
        }
    }
}

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use std::time::Duration;
use log::{info, error, debug};
use crate::error::RtspError;
use crate::rtsp_worker::RTSP_WORKER_MANAGER;

// 暴力枚举器
#[derive(Clone)]
pub struct BruteForcer {
    rtsp_url: String,
    users_file: String,
    passwords_file: String,
    max_concurrent: u32,
    delay: u64, // 每次尝试之间的延迟(毫秒)
}

impl BruteForcer {
    pub fn new(rtsp_url: &str, users_file: &str, passwords_file: &str) -> Self {
        BruteForcer {
            rtsp_url: rtsp_url.to_string(),
            users_file: users_file.to_string(),
            passwords_file: passwords_file.to_string(),
            max_concurrent: 5, // 默认最大并发数
            delay: 100, // 默认延迟100ms
        }
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

    // 从文件读取用户名列表
    fn read_usernames(&self) -> Result<Vec<String>, RtspError> {
        let file = File::open(&self.users_file).map_err(|e| {
RtspError::IoError(e)
        })?;
        let reader = BufReader::new(file);
        let mut usernames = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| {
                RtspError::IoError(e)
            })?;
            if !line.trim().is_empty() {
                usernames.push(line.trim().to_string());
            }
        }

        Ok(usernames)
    }

    // 从文件读取密码列表
    fn read_passwords(&self) -> Result<Vec<String>, RtspError> {
        let file = File::open(&self.passwords_file).map_err(|e| {
            RtspError::IoError(e)
        })?;
        let reader = BufReader::new(file);
        let mut passwords = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| {
                RtspError::IoError(e)
            })?;
            if !line.trim().is_empty() {
                passwords.push(line.trim().to_string());
            }
        }

        Ok(passwords)
    }

    // 尝试单个用户名和密码
    async fn try_credentials(&self, username: &str, password: &str) -> Result<bool, RtspError> {
        match RTSP_WORKER_MANAGER.auth_request(username, password, &self.rtsp_url).await {
            Ok(result) => {
                match result {
                    Some((valid_username, valid_password)) => {
                        info!("Success! Found valid credentials: {}:{}", valid_username, valid_password);
                        println!("Success! Found valid credentials: {}:{}", valid_username, valid_password);
                        Ok(true)
                    },
                    None => {
                        debug!("Failed attempt: {}:{}", username, password);
                        Ok(false)
                    }
                }
            },
            Err(e) => {
                error!("Error during authentication attempt: {:?}", e);
                Err(e)
            }
        }
    }

    // 执行暴力枚举
    pub async fn brute_force(&self) -> Result<(), RtspError> {
        info!("Starting brute force attack on {}", self.rtsp_url);
        info!("Using users file: {}", self.users_file);
        info!("Using passwords file: {}", self.passwords_file);
        info!("Max concurrent attempts: {}", self.max_concurrent);
        info!("Delay between attempts: {}ms", self.delay);

        // 读取用户名和密码列表
        let usernames = self.read_usernames()?;
        let passwords = self.read_passwords()?;

        info!("Loaded {} usernames and {} passwords", usernames.len(), passwords.len());

        // 创建信号量限制并发数
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent as usize));
        let mut tasks = Vec::new();

        // 尝试所有组合
        for username in &usernames {
            for password in &passwords {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let username_clone = username.clone();
                let password_clone = password.clone();
                let this_clone = self.clone();
                let delay = self.delay;

                let task = tokio::spawn(async move {
                    let _permit = permit; // 保持许可直到任务完成

                    // 尝试凭据
                    let result = this_clone.try_credentials(&username_clone, &password_clone).await;

                    // 延迟下一次尝试
                    sleep(Duration::from_millis(delay)).await;

                    result
                });

                tasks.push(task);
            }
        }

        // 等待所有任务完成
        let mut success_found = false;
        for task in tasks {
            match task.await {
                Ok(Ok(success)) => {
                    if success {
                        success_found = true;
                    }
                },
                Ok(Err(e)) => {
                    error!("Task failed with error: {:?}", e);
                },
                Err(e) => {
                    error!("Task panicked: {:?}", e);
                }
            }
        }

        if success_found {
            info!("Brute force attack completed. Valid credentials found.");
        } else {
            info!("Brute force attack completed. No valid credentials found.");
            println!("Error: No valid credentials found in the provided lists");
        }

        Ok(())
    }
}

use crate::brute::FoundCredential;
use crate::error::RtspError;
use log::{debug, error, info};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use futures::stream::StreamExt;

/// 任务管理器 - 负责处理任务的执行和结果收集
pub struct TaskManager {
    max_concurrent: u32,
}

impl TaskManager {
    pub fn new(max_concurrent: u32) -> Self {
        TaskManager {
            max_concurrent,
        }
    }

    /// 处理任务结果并返回统计信息
    pub async fn process_task_results(
        &self,
        tasks: Vec<JoinHandle<Result<Option<FoundCredential>, RtspError>>>,
        start_time: Instant,
        total_tasks: usize,
    ) -> (usize, usize, bool) {
        // 使用原子计数器保护共享变量
        let success_found = Arc::new(Mutex::new(false));
        let total_attempts = Arc::new(Mutex::new(0));
        let successful_attempts = Arc::new(Mutex::new(0));
        info!("Waiting for {} tasks to complete...", total_tasks);

        // 使用原子计数器跟踪活跃任务
        let active_tasks = Arc::new(Mutex::new(total_tasks));

        // 将任务转换为流并并发处理
        futures::stream::iter(tasks)
            .for_each_concurrent(Some(self.max_concurrent as usize), |task| {
                let success_found = success_found.clone();
                let total_attempts = total_attempts.clone();
                let successful_attempts = successful_attempts.clone();
                let active_tasks = active_tasks.clone();

                async move {
                    match task.await {
                        Ok(Ok(Some(found_cred))) => {
                            { 
                                let mut found = success_found.lock().unwrap();
                                *found = true;
                            }
                            { 
                                let mut count = successful_attempts.lock().unwrap();
                                *count += 1;
                            }
                            info!("Found valid credentials: {:?}", found_cred);
                        },
                        Ok(Ok(None)) => {
                            debug!("Authentication failed");
                        },
                        Ok(Err(e)) => {
                            error!("Authentication error: {:?}", e);
                        },
                        Err(e) => {
                            error!("Task failed with error: {:?}", e);
                        }
                    }

                    // 更新总尝试次数
                    { 
                        let mut count = total_attempts.lock().unwrap();
                        *count += 1;
                    }

                    // 更新活跃任务计数
                    { 
                        let mut count = active_tasks.lock().unwrap();
                        *count -= 1;
                        info!("Task completed. {} remaining.", *count);
                    }
                }
            })
            .await;

        let duration = start_time.elapsed();
        let total = *total_attempts.lock().unwrap();
        let successful = *successful_attempts.lock().unwrap();
        let found = *success_found.lock().unwrap();

        info!("Brute force completed in {:?}", duration);
        info!("Total attempts: {}, Successful: {}", total, successful);
        info!("Throughput: {:.2} attempts/second", 
              total as f64 / duration.as_secs_f64());

        if found {
            info!("Valid credentials found.");
        } else {
            info!("No valid credentials found.");
            println!("Error: No valid credentials found in the provided lists");
        }

        (total, successful, found)
    }
}
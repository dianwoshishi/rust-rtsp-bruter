use crate::brute::FoundCredential;
use crate::errors::errors::RtspError;
use futures::stream::StreamExt;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc};
use tokio::task::JoinHandle;
use tokio::time::Instant;

/// 任务管理器 - 负责处理任务的执行和结果收集
#[derive(Clone)]
pub struct TaskManager {
    max_concurrent: u32,
}

impl TaskManager {
    pub fn new(max_concurrent: u32) -> Self {
        TaskManager { max_concurrent }
    }

    /// 处理任务结果并返回统计信息
    pub async fn process_task_results(
        &self,
        tasks: Vec<JoinHandle<Result<Option<FoundCredential>, RtspError>>>,
        start_time: Instant,
        total_tasks: Arc<AtomicUsize>,
    ) -> (usize, usize, f64) {

        // 使用原子计数器保护共享变量
        let success_found = Arc::new(AtomicBool::new(false));
        let total_attempts = Arc::new(AtomicUsize::new(0));
        let successful_attempts = Arc::new(AtomicUsize::new(0));
        info!("Waiting for {} tasks to complete...", total_tasks.load(Ordering::Relaxed));

        // 使用原子计数器跟踪活跃任务
        let active_tasks = Arc::new(AtomicUsize::new(total_tasks.load(Ordering::Relaxed)));



        // 将任务转换为流并并发处理
        futures::stream::iter(tasks)
            .for_each_concurrent(Some(self.max_concurrent as usize), |task| {
                let success_found = success_found.clone();
                let total_attempts = total_attempts.clone();
                let successful_attempts = successful_attempts.clone();
                let active_tasks = active_tasks.clone();

                async move {
                    match task.await {
                        Ok(Ok(Some(_))) => {
                            {
                                let found = true;
                                success_found.store(found, Ordering::Relaxed);
                            }
                            {
                                let mut count = successful_attempts.load(Ordering::Relaxed);
                                count += 1;
                                successful_attempts.store(count, Ordering::Relaxed);
                            }
                            // info!("Found valid credentials: {:?}", found_cred);
                        }
                        Ok(Ok(None)) => {
                            debug!("Authentication failed");
                        }
                        Ok(Err(e)) => {
                            debug!("Authentication error: {:?}", e);
                        }
                        Err(e) => {
                            error!("Task failed with error: {:?}", e);
                        }
                    }

                    // 更新总尝试次数
                    {
                        let mut count = total_attempts.load(Ordering::Relaxed);
                        count += 1;
                        total_attempts.store(count, Ordering::Relaxed);
                    }

                    // 更新活跃任务计数
                    {
                        let mut count = active_tasks.load(Ordering::Relaxed);
                        count -= 1;
                        active_tasks.store(count, Ordering::Relaxed);
                        debug!("Task completed. {} remaining.", count);
                    }
                }
            })
            .await;

        let duration = start_time.elapsed();
        let total = total_attempts.load(Ordering::Relaxed);
        let successful = successful_attempts.load(Ordering::Relaxed);

        // 
        println!("\n{} Task Summary {}", "-".repeat(20), "-".repeat(20));
        info!("Brute force completed in {:?}", duration);
        info!("Total attempts: {}, Successful: {}", total, successful);
        
        info!(
            "Throughput: {:.2} attempts/second",
            total as f64 / duration.as_secs_f64()

        );
        (total, successful, duration.as_secs_f64())

    }
}

use std::error::Error;
extern crate lazy_static;

use clap::Parser;
use log::{debug, info};
use log4rs;
use rust_rtsp_bruter::cli::{self, handle_cli};
use rust_rtsp_bruter::rtsp_worker::RTSP_WORKER_MANAGER;
use tokio;
use std::time::Instant;

// 主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 记录开始时间
    let start_time = Instant::now();

    // 初始化日志
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");

    // 统一启动RTSP工作线程
    RTSP_WORKER_MANAGER.start().await;
    debug!("RTSP worker manager started");

    // 处理命令行参数
    handle_cli(cli::Cli::parse()).await?;

    // 统一停止RTSP工作线程
    RTSP_WORKER_MANAGER.stop().await;
    debug!("RTSP worker manager stopped");

    // 计算并输出总耗时
    let duration = start_time.elapsed();
    info!("Total execution time: {:?}", duration);

    Ok(())
}

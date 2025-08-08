use std::error::Error;
#[macro_use]
extern crate lazy_static;

use clap::Parser;
use log::{info, error, debug};
use log4rs;
use tokio;
use crate::rtsp_worker::RTSP_WORKER_MANAGER;
use crate::cli::{Cli, handle_cli};

// 主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    log4rs::init_file("log4rs.yaml", Default::default())
        .expect("Failed to initialize log4rs");

    // 解析命令行参数
    let cli = Cli::parse();

    // 统一启动RTSP工作线程
    RTSP_WORKER_MANAGER.start().await;
    debug!("RTSP worker manager started");

    // 处理命令行参数
    handle_cli(cli).await?;

    // 统一停止RTSP工作线程
    RTSP_WORKER_MANAGER.stop().await;
    debug!("RTSP worker manager stopped");

    Ok(())
}

// 模块声明
mod cli;
mod error;
mod auth;
mod client;
mod common;
mod brute;
mod rtsp_worker;
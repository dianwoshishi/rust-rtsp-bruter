use std::error::Error;
extern crate lazy_static;
use log::{debug};
use log4rs;
use rust_rtsp_bruter::config::config::{load_and_merge_config, load_config_and_handle_cli};
use rust_rtsp_bruter::rtsp::rtsp_worker::RTSP_WORKER_MANAGER;
use timing_macro::timing;
use tokio;

// 主函数
#[tokio::main]
#[timing]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");

    // 加载并合并配置文件和命令行参数
    let merged_config = load_and_merge_config()?;

    // 统一启动RTSP工作线程
    RTSP_WORKER_MANAGER.start().await;
    debug!("RTSP worker manager started");

    // 加载配置并处理命令行参数
    match load_config_and_handle_cli(merged_config).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error loading config: {:?}", e);
            // 统一停止RTSP工作线程
            RTSP_WORKER_MANAGER.stop().await;
            debug!("RTSP worker manager stopped");
            return Err(e);
        }
    }

    RTSP_WORKER_MANAGER.stop().await;
    debug!("RTSP worker manager stopped");
    Ok(())
}

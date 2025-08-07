use std::error::Error;
use std::env;
use log::info;
use log::debug;
use log::error;
use tokio;
use url::Url;
use crate::client::RtspClient;
use crate::error::RtspError;

// 主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 || args.len() > 5 {
        eprintln!("Usage: {} <rtsp_url> <username> <password> [stream_path]", args[0]);
        eprintln!("  [stream_path] is optional, defaults to empty string");
        std::process::exit(1);
    }

    let rtsp_url = &args[1];
    let username = &args[2];
    let password = &args[3];
    let stream_path = if args.len() == 5 { &args[4] } else { "" };
    let modified_rtsp_url = if stream_path.is_empty() {
        rtsp_url.to_string()
    } else {
        // 构建带有自定义流路径的URL
        let mut parsed_url = Url::parse(rtsp_url).map_err(|_| RtspError::UrlParseError)?;
        parsed_url.set_path(stream_path);
        parsed_url.to_string()
    };
    info!("Using RTSP URL: {}", modified_rtsp_url);

    info!("RTSP Bruteforcer started");
    info!("Target URL: {}", rtsp_url);
    info!("Username: {}", username);
    info!("Password: {}", password.replace(|_| true, "*")); // 隐藏密码

    // 创建RTSP客户端
    let client = RtspClient::new(username, password);
    info!("RTSP client created");

    // 发送DESCRIBE请求
    info!("Sending DESCRIBE request to {}", modified_rtsp_url);
    match client.describe(&modified_rtsp_url).await {
        Ok(_) => {
            info!("DESCRIBE request completed successfully");
            println!("Success: DESCRIBE request completed and media description received");
        },
        Err(e) => {
            error!("DESCRIBE request failed: {:?}", e);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

// 模块声明
mod error;
mod auth;
mod client;
mod common;
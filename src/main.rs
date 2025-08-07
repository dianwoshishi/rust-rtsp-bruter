use std::error::Error;
use std::env;
use log::info;
use log::debug;
use log::error;
use tokio;
use url::Url;
use crate::client::RtspClient;
use crate::error::RtspError;
use crate::brute::BruteForcer;

// 主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 检查是否使用暴力枚举模式
    if args.len() > 1 && args[1] == "--brute" {
        if args.len() != 5 {
            eprintln!("Usage: {} --brute <rtsp_url> <users_file> <passwords_file>", args[0]);
            std::process::exit(1);
        }

        let rtsp_url = &args[2];
        let users_file = &args[3];
        let passwords_file = &args[4];

        info!("RTSP Bruteforcer started in brute force mode");
        info!("Target URL: {}", rtsp_url);
        info!("Users file: {}", users_file);
        info!("Passwords file: {}", passwords_file);

        // 创建暴力枚举器
        let brute_forcer = BruteForcer::new(rtsp_url, users_file, passwords_file)
            .with_max_concurrent(5)
            .with_delay(100);

        // 执行暴力枚举
        info!("Starting brute force attack");
        match brute_forcer.brute_force().await {
            Ok(_) => {
                info!("Brute force attack completed");
            },
            Err(e) => {
                error!("Brute force attack failed: {:?}", e);
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // 传统模式: 单个用户名和密码
        if args.len() < 4 || args.len() > 5 {
            eprintln!("Usage: {} <rtsp_url> <username> <password> [stream_path]", args[0]);
            eprintln!("  [stream_path] is optional, defaults to empty string");
            eprintln!("  or");
            eprintln!("Usage: {} --brute <rtsp_url> <users_file> <passwords_file>", args[0]);
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

        info!("RTSP Bruteforcer started in single credential mode");
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
    }
    Ok(())
}

// 模块声明
mod error;
mod auth;
mod client;
mod common;
mod brute;
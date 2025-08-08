use clap::Parser;
use std::error::Error;
use url::Url;
use log::{info, debug};
use log::error;
use crate::error::RtspError;

// 定义命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    /// 使用单个凭据模式
    Single {
        /// RTSP URL
        rtsp_url: String,
        /// 用户名 (留空表示无需认证)
        #[arg(default_value = "")]
        username: String,
        /// 密码 (留空表示无需认证)
        #[arg(default_value = "")]
        password: String,
        /// 可选的流路径
        #[arg(short, long, default_value = "")]
        stream_path: String,
    },
    /// 使用暴力枚举模式
    Brute {
        /// RTSP URL
        rtsp_url: String,
        /// 包含用户名的文件路径
        users_file: String,
        /// 包含密码的文件路径
        passwords_file: String,
        /// 最大并发连接数
        #[arg(short, long, default_value_t = 5)]
        max_concurrent: u32,
        /// 尝试之间的延迟(毫秒)
        #[arg(short, long, default_value_t = 100)]
        delay: u64,
    },
}

// 处理命令行参数并执行相应的操作
pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli {
        Cli::Brute { rtsp_url, users_file, passwords_file, max_concurrent, delay } => {
            log::debug!("RTSP Bruteforcer started in brute force mode");
            log::debug!("Target URL: {}", rtsp_url);
            log::debug!("Users file: {}", users_file);
            log::debug!("Passwords file: {}", passwords_file);
            log::debug!("Max concurrent connections: {}", max_concurrent);
            log::debug!("Delay between attempts: {}ms", delay);

            // 创建暴力枚举器
            let brute_forcer = crate::brute::BruteForcer::new(&rtsp_url, &users_file, &passwords_file)
                .with_max_concurrent(max_concurrent)
                .with_delay(delay);

            // 执行暴力枚举
            log::debug!("Starting brute force attack");
            match brute_forcer.brute_force().await {
                Ok(_) => {
                    log::debug!("Brute force attack completed");
                },
                Err(e) => {
                    log::error!("Brute force attack failed: {:?}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Cli::Single { rtsp_url, username, password, stream_path } => {
            let modified_rtsp_url = if stream_path.is_empty() {
                rtsp_url.clone()
            } else {
                // 构建带有自定义流路径的URL
                let mut parsed_url = Url::parse(&rtsp_url).map_err(|_| RtspError::UrlParseError)?;
                parsed_url.set_path(&stream_path);
                parsed_url.to_string()
            };
            log::debug!("Using RTSP URL: {}", modified_rtsp_url);

            log::debug!("RTSP Bruteforcer started in single credential mode");
            log::info!("Target URL: {}", rtsp_url);
            log::debug!("Username: {}", username);
            log::debug!("Password: {}", password.replace(|_| true, "*")); // 隐藏密码

            // 发送认证请求
            log::debug!("Sending authentication request to {}", modified_rtsp_url);
            match crate::rtsp_worker::RTSP_WORKER_MANAGER.auth_request(&username, &password, &modified_rtsp_url).await {
                Ok(result) => {
                    match result {
                        Some((valid_username, valid_password)) => {
                            log::debug!("Authentication successful");
                            println!("Success: Valid credentials found");
                            println!("Username: {}", valid_username);
                            println!("Password: {}", valid_password);
                        },
                        None => {
                            log::debug!("No authentication required");
                            println!("Success: No authentication required");
                        }
                    }
                },
                Err(e) => {
                    log::error!("Authentication request failed: {:?}", e);
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        },
    }
    Ok(())
}
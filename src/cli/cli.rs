use crate::errors::errors::RtspError;
use clap::Parser;
use log;
use std::error::Error;
use std::sync::Arc;
// use url::Url;  // 未使用的导入，已注释
use crate::iterator::ip_reader::{IpReader, IpSource};

use crate::iterator::credential_iterator::CredentialIterator;
use crate::iterator::credential_reader::{CredentialReader, CredentialSource};
use crate::iterator::ip_iterator::IpIterator;
use crate::brute::brute_forcer::BruteForcer;


// 定义命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    /// 使用暴力枚举模式
    Brute {
        /// 包含用户名的文件路径 (与users_string二选一)
        #[arg(long, conflicts_with = "users_string")]
        users_file: Option<String>,
        /// 用户名字符串 (与users_file二选一)
        #[arg(long, conflicts_with = "users_file")]
        users_string: Option<String>,
        /// 包含密码的文件路径 (与passwords_string二选一)
        #[arg(long, conflicts_with = "passwords_string")]
        passwords_file: Option<String>,
        /// 密码字符串 (与passwords_file二选一)
        #[arg(long, conflicts_with = "passwords_file")]
        passwords_string: Option<String>,
        /// 包含IP地址的文件路径 (与ips_string二选一)
        #[arg(long, conflicts_with = "ips_string")]
        ips_file: Option<String>,
        /// IP地址字符串 (与ips_file二选一)
        #[arg(long, conflicts_with = "ips_file")]
        ips_string: Option<String>,
        /// 最大并发连接数
        #[arg(short, long, default_value_t = 5)]
        max_concurrent: u32,
        // /// 尝试之间的延迟(毫秒)
        // #[arg(short, long, default_value_t = 100)]
        // delay: u64,
    },
}

// 解析Brute模式的命令行参数
pub fn parse_brute_args(
    brute: Cli,
) -> Result<(IpIterator, CredentialIterator, u32), Box<dyn Error>> {
    let Cli::Brute {
        users_file,
        users_string,
        passwords_file,
        passwords_string,
        ips_file,
        ips_string,
        max_concurrent,
        // delay,
    } = brute;

    log::debug!("RTSP Bruteforcer started in brute force mode");
    match &users_file {
        Some(file) => log::debug!("Users file: {}", file),
        None => log::debug!(
            "User string: {}",
            users_string.as_ref().unwrap_or(&"none".to_string())
        ),
    }
    match &passwords_file {
        Some(file) => log::debug!("Passwords file: {}", file),
        None => log::debug!(
            "Password string: {}",
            passwords_string.as_ref().unwrap_or(&"none".to_string())
        ),
    }
    match &ips_file {
        Some(file) => log::debug!("IPs file: {}", file),
        None => log::debug!(
            "IP string: {}",
            ips_string.as_ref().unwrap_or(&"none".to_string())
        ),
    }
    log::debug!("Max concurrent connections: {}", max_concurrent);
    // log::debug!("Delay between attempts: {}ms", delay);

    // 创建IP读取器
    let ip_reader = match (ips_file, ips_string) {
        (Some(file), None) => IpReader::<IpSource>::from_file(&file),
        (None, Some(ip)) => IpReader::<IpSource>::from_string(&ip),
        _ => {
            return Err(RtspError::InvalidArgument(
                "Either ips_file or ips_string must be provided".to_string(),
            )
            .into());
        }
    };
    let ip_iterator = ip_reader.into_iterator()?;

    // 创建凭据读取器和迭代器
    let credential_reader = match (users_file, users_string, passwords_file, passwords_string) {
        (Some(u_file), None, Some(p_file), None) => {
            CredentialReader::<CredentialSource>::from_files(&u_file, &p_file)
        }
        (Some(u_file), None, None, Some(p_str)) => {
            CredentialReader::<CredentialSource>::from_file_and_string(&u_file, p_str)
        }
        (None, Some(u_str), Some(p_file), None) => {
            CredentialReader::<CredentialSource>::from_string_and_file(u_str, &p_file)
        }
        (None, Some(u_str), None, Some(p_str)) => {
            CredentialReader::<CredentialSource>::from_strings(u_str, p_str)
        }
        _ => {
            return Err(RtspError::InvalidArgument(
                "Invalid combination of username and password sources".to_string(),
            )
            .into());
        }
    };
    let cred_iterator = credential_reader.into_iterator()?;

    Ok((ip_iterator, cred_iterator, max_concurrent))
}

// 处理命令行参数并执行相应的操作
pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn Error>> {
    let (ip_iterator, cred_iterator, max_concurrent) = parse_brute_args(cli)?;

    // 创建暴力枚举器
    let brute_forcer = Arc::new(BruteForcer::new()
        .with_max_concurrent(max_concurrent)
        // .with_delay(delay)
        .with_ip_iterator(ip_iterator)
        .with_cred_iterator(cred_iterator));

    // 执行暴力枚举
    log::debug!("Starting brute force attack");
    match brute_forcer.brute_force().await {
        Ok(_) => {
            log::debug!("Brute force attack completed");
        }
        Err(e) => {
            log::error!("Brute force attack failed: {:?}", e);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

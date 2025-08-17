use crate::errors::errors::RtspError;
use log;
use std::error::Error;
use std::sync::Arc;
// use url::Url;  // 未使用的导入，已注释
use crate::brute::brute_forcer::BruteForcer;
use crate::config::config::Cli;
use crate::iterator::credential_iterator::CredentialIterator;
use crate::iterator::credential_reader::{CredentialReader, CredentialSource};
use crate::iterator::ip_iterator::IpIterator;
use crate::iterator::ip_reader::{IpReader, IpSource};

// 解析Brute模式的命令行参数
pub fn parse_brute_args(cli: Cli) -> Result<(IpIterator, CredentialIterator, u32), Box<dyn Error>> {
    let Cli::Args {
        users_file,
        users_string,
        passwords_file,
        passwords_string,
        ips_file,
        ips_string,
        max_concurrent,
        // delay,
    } = cli;

    // ip迭代器，从文件或者命令行参数中获取Ip地址并解析
    let ip_iterator: IpIterator = {
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
        ip_reader.into_iterator()?
    };

    // 凭证迭代器，从文件或命令行参数中获取（username，password）对
    let cred_iterator:CredentialIterator = {
        // 创建凭据读取器和迭代器
        let credential_reader = match (
                                                                    users_file,
                                                                    users_string,
                                                                    passwords_file,
                                                                    passwords_string)
        {
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
        credential_reader.into_iterator()?
    };

    Ok((ip_iterator, cred_iterator, max_concurrent))
}

// 处理命令行参数并执行相应的操作
pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn Error>> {
    // 从配置中获取ip，（用户名，密码）列表，接下来用于生成rtsp任务，验证ip端口和密码
    let (ip_iterator, cred_iterator, max_concurrent) = parse_brute_args(cli)?;

    // 创建暴力枚举器
    let brute_forcer = Arc::new(
        BruteForcer::new()
            .with_max_concurrent(max_concurrent)
            .with_ip_iterator(ip_iterator)
            .with_cred_iterator(cred_iterator),
    );

    // 执行暴力枚举
    log::debug!("Starting brute force attack");
    match brute_forcer.brute_force().await {
        Ok(()) => {
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

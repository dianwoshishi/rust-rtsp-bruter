use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use toml;

/// 命令行参数枚举
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    /// 使用暴力枚举模式
    Args {
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

/// 从配置文件中读取的配置内容
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// 运行模式 (brute)
    pub mode: String,
    /// 包含用户名的文件路径
    pub users_file: Option<String>,
    /// 用户名字符串
    pub users_string: Option<String>,
    /// 包含密码的文件路径
    pub passwords_file: Option<String>,
    /// 密码字符串
    pub passwords_string: Option<String>,
    /// 包含IP地址的文件路径
    pub ips_file: Option<String>,
    /// IP地址字符串
    pub ips_string: Option<String>,
    /// 最大并发连接数
    pub max_concurrent: u32,
    // /// 尝试之间的延迟(毫秒)
    // pub delay: u64,
}

impl AppConfig {
    /// 从配置文件中加载配置
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_content)?;
        Ok(config)
    }

    /// 从命令行参数创建配置
    pub fn from_cli(cli: Cli) -> Result<Self, Box<dyn std::error::Error>> {
        match cli {
            Cli::Args {
                users_file,
                users_string,
                passwords_file,
                passwords_string,
                ips_file,
                ips_string,
                max_concurrent,
            } => Ok(AppConfig {
                mode: "brute".to_string(),
                users_file,
                users_string,
                passwords_file,
                passwords_string,
                ips_file,
                ips_string,
                max_concurrent,
            }),
        }
    }

    /// 合并配置文件和命令行参数，命令行参数优先级更高
    pub fn merge_with_cli(&self, cli: Cli) -> Result<Self, Box<dyn std::error::Error>> {
        let cli_config = AppConfig::from_cli(cli)?;

        Ok(AppConfig {
            mode: self.mode.clone(),
            users_file: cli_config.users_file.or(self.users_file.clone()),
            users_string: cli_config.users_string.or(self.users_string.clone()),
            passwords_file: cli_config.passwords_file.or(self.passwords_file.clone()),
            passwords_string: cli_config
                .passwords_string
                .or(self.passwords_string.clone()),
            ips_file: cli_config.ips_file.or(self.ips_file.clone()),
            ips_string: cli_config.ips_string.or(self.ips_string.clone()),
            max_concurrent: cli_config.max_concurrent,
        })
    }
}

pub fn load_and_merge_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    // 解析命令行参数，如果解析失败则使用默认值
    let cli = Cli::try_parse().unwrap_or_else(|e| {
        log::warn!("Failed to parse CLI arguments: {}. Using default values.", e);
        Cli::Args {
            users_file: None,
            users_string: None,
            passwords_file: None,
            passwords_string: None,
            ips_file: None,
            ips_string: None,
            max_concurrent: 5,
        }
    });
    log::debug!("{:?}", cli);

    // 加载配置文件
    let config_path = std::path::PathBuf::from("./config.toml");
    let config = AppConfig::load_from_file(&config_path).unwrap_or_else(|_| {
        log::debug!("Failed to load config file, using default values");
        AppConfig {
            mode: "brute".to_string(),
            users_file: Some("users.txt".to_string()),
            users_string: None,
            passwords_file: Some("passwords.txt".to_string()),
            passwords_string: None,
            ips_file: Some("iplist.txt".to_string()),
            ips_string: None,
            max_concurrent: 5,
        }
    });
    log::debug!("{:?}", config);

    // 合并配置文件和命令行参数
    let merged_config = config.merge_with_cli(cli)?;
    log::debug!("{:?}", merged_config);
    Ok(merged_config)
}

/// 加载配置文件并处理命令行参数
pub async fn load_config_and_handle_cli() -> Result<(), Box<dyn std::error::Error>> {
    // 加载并合并配置文件和命令行参数
    let merged_config = load_and_merge_config()?;
    
    crate::cli::cli::handle_cli(Cli::Args {
        users_file: merged_config.users_file,
        users_string: merged_config.users_string,
        passwords_file: merged_config.passwords_file,
        passwords_string: merged_config.passwords_string,
        ips_file: merged_config.ips_file,
        ips_string: merged_config.ips_string,
        max_concurrent: merged_config.max_concurrent,
    })
    .await?;

    Ok(())
}

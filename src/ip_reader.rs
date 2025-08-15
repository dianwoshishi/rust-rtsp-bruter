use crate::error::RtspError;
use crate::ip_iterator::IpIterator;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, ToSocketAddrs};
use std::vec::Vec;

// 定义IP数据源类型
pub enum IpSource {
    FilePath(String),
    IpString(String),
}

// IP地址读取器 - 支持多种IP数据源
#[derive(Clone)]
pub struct IpReader<T> {
    source: T,
}

impl IpReader<IpSource> {
    // 从文件路径创建IP读取器
    pub fn from_file(ips_file: &str) -> Self {
        IpReader {
            source: IpSource::FilePath(ips_file.to_string()),
        }
    }

    // 从IP字符串创建IP读取器
    pub fn from_string(ip_string: &str) -> Self {
        IpReader {
            source: IpSource::IpString(ip_string.to_string()),
        }
    }

    // 读取IP地址列表
    fn read_ips(&self) -> Result<Vec<String>, RtspError> {
        let ips = match &self.source {
            IpSource::FilePath(file_path) => {
                let file = File::open(file_path).map_err(|e| RtspError::IoError(e))?;
                let reader = BufReader::new(file);
                let mut ips = Vec::new();

                for line in reader.lines() {
                    let line = line.map_err(|e| RtspError::IoError(e))?;
                    let trimmed_line = line.trim();
                    if trimmed_line.is_empty() {
                        continue;
                    }
                    ips.push(trimmed_line.to_string());
                }

                ips
            }
            IpSource::IpString(ip_string) => {
                let trimmed_line = ip_string.trim();
                let mut ips = Vec::new();

                ips.push(trimmed_line.to_string());
                ips
            }
        };

        // 解析IP地址（支持带端口格式）
        let mut parsed_ips = Vec::new();
        for ip in &ips {
            // 处理带端口的IP地址
            let parts: Vec<&str> = ip.split(':').collect();
            let ip_without_port = parts[0].trim();
            let port = if parts.len() > 1 { parts[1].trim() } else { "" };

            // 尝试解析IP部分
            match ip_without_port.parse::<IpAddr>() {
                Ok(_) => {
                    // IP部分有效，保留原始格式（包括端口）
                    parsed_ips.push(ip.clone());
                }
                Err(ip_err) => {
                    // IP部分无效，尝试解析为域名
                    match (ip_without_port, 0).to_socket_addrs() {
                        Ok(mut addrs) => {
                            if let Some(addr) = addrs.next() {
                                // 对于域名，使用解析后的IP地址，并保留原始端口
                                let ip_with_port = if !port.is_empty() {
                                    format!("{}:{}", addr.ip().to_string(), port)
                                } else {
                                    addr.ip().to_string()
                                };
                                parsed_ips.push(ip_with_port);
                            } else {
                                return Err(RtspError::InvalidIpAddress(format!(
                                    "Invalid IP address: {}. Error: No address found for domain",
                                    ip
                                )));
                            }
                        }
                        Err(dns_err) => {
                            return Err(RtspError::InvalidIpAddress(format!(
                                "Invalid IP address: {}. IP parse error: {:?}, DNS error: {:?}",
                                ip, ip_err, dns_err
                            )));
                        }
                    }
                }
            }
        }

        Ok(parsed_ips)
    }

    // 创建IP地址迭代器
    pub fn into_iterator(&self) -> Result<IpIterator, RtspError> {
        let ip_strings = self.read_ips()?;
        IpIterator::from_strings(ip_strings)
    }
}

// 为了向后兼容，保留原来的IpReader实现
pub type FileIpReader = IpReader<IpSource>;

impl FileIpReader {
    pub fn new(ips_file: &str) -> Self {
        Self::from_file(ips_file)
    }
}

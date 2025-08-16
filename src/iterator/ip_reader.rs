use crate::errors::errors::RtspError;
use crate::iterator::ip_iterator::IpIterator;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, ToSocketAddrs};
use std::vec::Vec;

// 定义IP数据源类型
#[derive(Clone)]
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

    fn parse_ips(&self, ips: Vec<String>) -> Result<Vec<String>, RtspError> {
        // 使用ip_port_parser解析IP地址（支持带端口格式、CIDR和花括号展开）
        let mut parsed_ips = Vec::new();
        for ip in &ips {
            // 首先尝试使用ip_port_parser解析
            // println!("{}", &ip);

            match super::ip_port_parser::parse_ip_port(ip) {
                Ok(ip_ports) => {
                    // 解析成功，转换为字符串形式
                    println!("{:?}", ip_ports);

                    for ip_port in ip_ports {
                        if ip_port.ports.is_empty() {
                            parsed_ips.push(ip_port.ip.to_string());
                        } else {
                            for port in &ip_port.ports {
                                parsed_ips.push(format!("{}:{}", ip_port.ip, port));
                            }
                        }
                    }
                },
                Err(parse_err) => {
                    // 解析失败，尝试作为域名处理
                    // 提取IP部分和端口部分
                    let (ip_without_port, port) = if ip.contains(':') {
                        let parts: Vec<&str> = ip.splitn(2, ':').collect();
                        (parts[0].trim(), parts[1].trim())
                    } else {
                        (ip.trim(), "")
                    };

                    match (ip_without_port, 0).to_socket_addrs() {
                        Ok(mut addrs) => {
                            if let Some(addr) = addrs.next() {
                                let ip_with_port = if !port.is_empty() {
                                    format!("{}:{}", addr.ip().to_string(), port)
                                } else {
                                    addr.ip().to_string()
                                };
                                parsed_ips.push(ip_with_port);
                            } else {
                                return Err(RtspError::InvalidIpAddress(format!(
                                    "Invalid IP address: {}. Error: No address found for domain and parsing failed: {:?}",
                                    ip, parse_err
                                )));
                            }
                        },
                        Err(dns_err) => {
                            return Err(RtspError::InvalidIpAddress(format!(
                                "Invalid IP address: {}. Parsing error: {:?}, DNS error: {:?}",
                                ip, parse_err, dns_err
                            )));
                        }
                    }
                }
            }
        }
        let mut unique_ips = Vec::new();
        for ip in parsed_ips {
            if !unique_ips.contains(&ip) {
                unique_ips.push(ip);
            }
        }
        Ok(unique_ips)
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

        self.parse_ips(ips)

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

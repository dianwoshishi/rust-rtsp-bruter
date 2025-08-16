use crate::error::RtspError;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::str::FromStr;
use std::vec::Vec;

// 存储IP地址和端口信息的结构体
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpPortAddr {
    pub ip: IpAddr,
    pub port: u16,
}

impl Display for IpPortAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

impl Hash for IpPortAddr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.port.hash(state);
    }
}

// IP地址迭代器
#[derive(Clone)]
pub struct IpIterator {
    ip_ports: Vec<IpPortAddr>,
    index: usize,
}

impl Iterator for IpIterator {
    type Item = IpPortAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.ip_ports.len() {
            return None;
        }

        let current_ip_port = self.ip_ports[self.index].clone();
        self.index += 1;

        Some(current_ip_port)
    }
}

impl IpIterator {
    pub fn new(ip_ports: Vec<IpPortAddr>) -> Self {
        IpIterator { ip_ports, index: 0 }
    }

    // 从字符串列表解析IP地址和端口
    pub fn from_strings(ip_strings: Vec<String>) -> Result<Self, RtspError> {
        let mut ip_ports = Vec::new();

        for ip_str in ip_strings {
            // 处理带端口的IP地址
            let parts: Vec<&str> = ip_str.trim().split(':').collect();
            if parts.len() < 1 || parts.len() > 2 {
                return Err(RtspError::InvalidIpAddress(format!(
                    "Invalid IP address format: {}",
                    ip_str
                )));
            }

            let ip_part = parts[0].trim();
            let port_part = if parts.len() == 2 {
                parts[1].trim()
            } else {
                "554"
            };

            // 解析IP地址
            let ip = match IpAddr::from_str(ip_part) {
                Ok(ip) => ip,
                Err(e) => {
                    return Err(RtspError::InvalidIpAddress(format!(
                        "Invalid IP address: {}. Error: {:?}",
                        ip_str, e
                    )));
                }
            };

            // 解析端口
            let port = match port_part.parse::<u16>() {
                Ok(port) => port,
                Err(e) => {
                    return Err(RtspError::InvalidIpAddress(format!(
                        "Invalid port: {}. Error: {:?}",
                        port_part, e
                    )));
                }
            };

            ip_ports.push(IpPortAddr { ip, port });
        }

        Ok(IpIterator::new(ip_ports))
    }
}

use std::{collections::HashSet, net::{IpAddr, Ipv4Addr}};

use crate::errors::errors::{ParseError, Result};

/// 表示IP地址中的一个段
#[derive(Debug, Clone)]
enum IpSegment {
    /// 单个数值 (如 1, 255)
    Single(u8),
    /// 数值范围 (如 1-100)
    Range(u8, u8),
    /// 多个选择项 (如 1,5,10-20)
    Multiple(Vec<IpSegment>),
    /// 嵌套的花括号表达式 (如 {1-100}, {1,5,10-20})
    Braced(Vec<IpSegment>)
}

/// IP地址模式
#[derive(Debug, Clone)]
struct IpAddrPattern {
    /// IP地址的四个段
    segments: [IpSegment; 4],
    /// 可选的CIDR掩码 (如 24 表示 /24)
    cidr: Option<u8>
}

/// 端口规范
#[derive(Debug, Clone)]
enum PortSpec {
    /// 单个端口 (如 80)
    Single(u16),
    /// 端口范围 (如 8000-8010)
    Range(u16, u16),
    /// 多个端口或范围 (如 80,443,8000-8010)
    Multiple(Vec<PortSpec>)
}

/// IP端口组合
#[derive(Debug, Clone)]
pub struct IpPort {
    pub ip: IpAddr,
    pub ports: Vec<u16>
}

/// IP段解析器
struct IpSegmentParser;

impl IpSegmentParser {
    /// 解析单个IP段表达式
    fn parse_segment(input: &str) -> Result<IpSegment> {
        // 处理花括号表达式
        if input.starts_with('{') && input.ends_with('}') {
            let content = &input[1..input.len()-1];
            let segments = Self::parse_multiple_segments(content)?;
            return Ok(IpSegment::Braced(segments));
        }

        // 处理多个选择项
        if input.contains(',') {
            let segments = Self::parse_multiple_segments(input)?;
            return Ok(IpSegment::Multiple(segments));
        }

        // 处理范围
        if input.contains('-') {
            let parts: Vec<&str> = input.split('-').collect();
            if parts.len() != 2 {
                return Err(ParseError::InvalidIpSegmentFormat(input.to_string()));
            }

            let start = parts[0].parse::<u8>()
                .map_err(|_| ParseError::InvalidIpSegmentFormat(input.to_string()))?;
            let end = parts[1].parse::<u8>()
                .map_err(|_| ParseError::InvalidIpSegmentFormat(input.to_string()))?;

            if start > end {
                return Err(ParseError::InvalidIpSegmentRange(start, end));
            }

            return Ok(IpSegment::Range(start, end));
        }

        // 处理单个数值
        let val = input.parse::<u8>()
            .map_err(|_| ParseError::InvalidIpSegmentFormat(input.to_string()))?;
        Ok(IpSegment::Single(val))
    }

    /// 解析多个IP段
    fn parse_multiple_segments(input: &str) -> Result<Vec<IpSegment>> {
        let mut segments = Vec::new();
        // 按逗号分割并递归解析每个部分
        for part in input.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                segments.push(Self::parse_segment(trimmed)?);
            }
        }

        if segments.is_empty() {
            return Err(ParseError::EmptyIpSegment);
        }

        Ok(segments)
    }
}

/// IP地址模式解析器
struct IpAddrPatternParser;

impl IpAddrPatternParser {
    /// 解析IP地址模式
    fn parse(input: &str) -> Result<IpAddrPattern> {
        // 处理CIDR掩码
        let (ip_part, cidr) = if input.contains('/') {
            let parts: Vec<&str> = input.split('/').collect();
            if parts.len() != 2 {
                return Err(ParseError::InvalidCidrFormat(input.to_string()));
            }

            let cidr_val = parts[1].parse::<u8>()
                .map_err(|_| ParseError::InvalidCidrValue(parts[1].to_string()))?;

            if cidr_val > 32 {
                return Err(ParseError::InvalidCidrValue(cidr_val.to_string()));
            }

            (parts[0], Some(cidr_val))
        } else {
            (input, None)
        };

        // 解析IP部分
        let octets: Vec<&str> = ip_part.split('.').collect();
        if octets.len() != 4 {
            return Err(ParseError::InvalidIpFormat(input.to_string()));
        }

        // 使用const块初始化数组，因为IpSegment没有实现Copy trait
        const DEFAULT_SEGMENT: IpSegment = IpSegment::Single(0);
        let mut segments = [DEFAULT_SEGMENT; 4];
        for (i, octet) in octets.iter().enumerate() {
            segments[i] = IpSegmentParser::parse_segment(octet)?;
        }

        Ok(IpAddrPattern {
            segments,
            cidr
        })
    }
}

/// 端口解析器
struct PortParser;

impl PortParser {
    /// 解析端口规范
    fn parse(input: &str) -> Result<PortSpec> {
        // 处理花括号表达式
        if input.starts_with('{') && input.ends_with('}') {
            let content = &input[1..input.len()-1];
            return Self::parse(content);
        }

        // 处理多个选择项
        if input.contains(',') {
            let mut specs = Vec::new();
            for part in input.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    specs.push(Self::parse(trimmed)?);
                }
            }

            if specs.is_empty() {
                return Err(ParseError::EmptyPortSpec);
            }

            return Ok(PortSpec::Multiple(specs));
        }

        // 处理范围
        if input.contains('-') {
            let parts: Vec<&str> = input.split('-').collect();
            if parts.len() != 2 {
                return Err(ParseError::InvalidPortRangeFormat(input.to_string()));
            }

            let start = parts[0].parse::<u16>()
                .map_err(|_| ParseError::InvalidPortNumber(parts[0].to_string()))?;
            let end = parts[1].parse::<u16>()
                .map_err(|_| ParseError::InvalidPortNumber(parts[1].to_string()))?;

            if start > end {
                return Err(ParseError::InvalidPortRange(start, end));
            }

            return Ok(PortSpec::Range(start, end));
        }

        // 处理单个数值
        let val = input.parse::<u16>()
            .map_err(|_| ParseError::InvalidPortNumber(input.to_string()))?;
        Ok(PortSpec::Single(val))
    }
}

/// 展开IP段为具体数值
fn expand_segment(segment: &IpSegment) -> Vec<u8> {
    match segment {
        IpSegment::Single(val) => vec![*val],
        IpSegment::Range(start, end) => (*start..=*end).collect(),
        IpSegment::Multiple(segments) => segments.iter()
              .flat_map(expand_segment)
              .collect(),
        IpSegment::Braced(segments) => segments.iter()
              .flat_map(expand_segment)
              .collect(),
    }
}

/// 展开端口规范为具体端口列表
fn expand_port_spec(spec: &PortSpec) -> Vec<u16> {
    match spec {
        PortSpec::Single(port) => vec![*port],
        PortSpec::Range(start, end) => (*start..=*end).collect(),
        PortSpec::Multiple(specs) => specs.iter()
            .flat_map(expand_port_spec)
            .collect()
    }
}

/// 从IP地址模式生成所有可能的IP地址
fn generate_ips(pattern: &IpAddrPattern) -> Vec<IpAddr> {
    // 展开所有段
    let seg1 = expand_segment(&pattern.segments[0]);
    let seg2 = expand_segment(&pattern.segments[1]);
    let seg3 = expand_segment(&pattern.segments[2]);
    let seg4 = expand_segment(&pattern.segments[3]);

    // 生成所有组合
    let mut ips = Vec::new();
    for s1 in &seg1 {
        for s2 in &seg2 {
            for s3 in &seg3 {
                for s4 in &seg4 {
                    ips.push(IpAddr::V4(Ipv4Addr::new(*s1, *s2, *s3, *s4)));
                }
            }
        }
    }

    ips
}

/// 应用CIDR掩码过滤IP地址或生成CIDR范围内的所有IP
fn apply_cidr(ips: Vec<IpAddr>, cidr: u8) -> Result<Vec<IpAddr>> {
    if ips.is_empty() {
        return Ok(ips);
    }

    // 检查所有IP是否为IPv4
    for ip in &ips {
        if !ip.is_ipv4() {
            return Err(ParseError::InvalidIpFormat("Only IPv4 addresses are supported with CIDR".to_string()));
        }
    }

    assert!(cidr >= 16, "the cidr of ipv4 should greater than 16 cause the ipv4 address space.");
    // 创建CIDR掩码
    let mask = if cidr == 0 {
        0
    } else {
        u32::MAX << (32 - cidr)
    };

    let mut network_addresses = HashSet::new();
    for ip in ips {
        // 获取第一个IP的网络地址
        let first_ip = if let IpAddr::V4(ipv4) = ip {
            ipv4
        } else {
            // 这里理论上不会发生，因为我们已经检查了所有IP都是IPv4
            return Err(ParseError::InvalidIpFormat("Only IPv4 addresses are supported with CIDR".to_string()));
        };        
        let network_addr = u32::from(first_ip) & mask;
        network_addresses.insert(network_addr);
    }
    // println!("{:?}", &network_addresses);

    let mut cidr_ips_collection = Vec::new();
    for network_addr in network_addresses{
        // 如果输入只有一个IP，我们生成整个CIDR范围的IP

        // 计算主机数量
        let num_hosts = 1 << (32 - cidr);
        let mut cidr_ips = Vec::with_capacity(num_hosts as usize);
        for i in 0..num_hosts {
            let ip_val = network_addr + i;
            let ipv4 = Ipv4Addr::new(
                ((ip_val >> 24) & 0xFF) as u8,
                ((ip_val >> 16) & 0xFF) as u8,
                ((ip_val >> 8) & 0xFF) as u8,
                (ip_val & 0xFF) as u8
            );
            cidr_ips.push(IpAddr::V4(ipv4));
        }
        cidr_ips_collection.extend(cidr_ips);
    }
    // println!("{:?}", &cidr_ips_collection);

    Ok(cidr_ips_collection)
    
}

/// 拆分IP部分和端口部分
fn split_ip_port(input: &str) -> Result<(&str, &str)> {
    if input.contains(':') {
        let parts: Vec<&str> = input.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(ParseError::InvalidIpPortFormat(input.to_string()));
        }
        Ok((parts[0], parts[1]))
    } else {
        Ok((input, ""))
    }
}

/// 解析IP端口字符串
/// 例如，"10.{1-2,{3-4}}.{{5-6},7}.8:{80,443}",
pub fn parse_ip_port(input: &str) -> Result<Vec<IpPort>> {
    // 1. 拆分IP部分和端口部分
    let (ip_part, port_part) = split_ip_port(input)?;

    // 2. 解析IP部分（包含CIDR）
    let ip_pattern = IpAddrPatternParser::parse(ip_part)?;

    // 3. 解析端口部分
    let port_spec = if !port_part.is_empty() {
        Some(PortParser::parse(port_part)?)
    } else {
        None
    };

    // 4. 生成IP地址
    let mut ips = generate_ips(&ip_pattern);
    // println!("{:?}", ips);

    // 5. 应用CIDR掩码（如果有）
    if let Some(cidr) = ip_pattern.cidr {
        ips = apply_cidr(ips, cidr)?;
    }
    // println!("{:?}", &ips.len());

    // 6. 生成端口列表
    let ports = if let Some(spec) = port_spec {
        expand_port_spec(&spec)
    } else {
        vec![]
    };

    // 7. 组合IP和端口
    Ok(ips.into_iter()
        .map(|ip| IpPort {
            ip,
            ports: ports.clone()
        })
        .collect())
}
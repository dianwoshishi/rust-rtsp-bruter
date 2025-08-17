use std::error::Error as StdError;
use std::fmt;
use std::io;

/// 定义结果类型别名
pub type Result<T> = std::result::Result<T, ParseError>;

/// 解析错误类型
#[derive(Debug)]
pub enum ParseError {
    /// 无效的IP段格式
    InvalidIpSegmentFormat(String),
    /// 无效的IP段范围
    InvalidIpSegmentRange(u8, u8),
    /// 空的IP段
    EmptyIpSegment,
    /// 无效的CIDR格式
    InvalidCidrFormat(String),
    /// 无效的CIDR值
    InvalidCidrValue(String),
    /// 无效的IP格式
    InvalidIpFormat(String),
    /// 无效的端口范围格式
    InvalidPortRangeFormat(String),
    /// 无效的端口号
    InvalidPortNumber(String),
    /// 无效的端口范围
    InvalidPortRange(u16, u16),
    /// 空的端口规范
    EmptyPortSpec,
    /// 无效的IP端口格式
    InvalidIpPortFormat(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidIpSegmentFormat(s) => write!(f, "Invalid IP segment format: {}", s),
            ParseError::InvalidIpSegmentRange(start, end) => {
                write!(f, "Invalid IP segment range: {}-{}", start, end)
            }
            ParseError::EmptyIpSegment => write!(f, "Empty IP segment"),
            ParseError::InvalidCidrFormat(s) => write!(f, "Invalid CIDR format: {}", s),
            ParseError::InvalidCidrValue(s) => write!(f, "Invalid CIDR value: {}", s),
            ParseError::InvalidIpFormat(s) => write!(f, "Invalid IP format: {}", s),
            ParseError::InvalidPortRangeFormat(s) => write!(f, "Invalid port range format: {}", s),
            ParseError::InvalidPortNumber(s) => write!(f, "Invalid port number: {}", s),
            ParseError::InvalidPortRange(start, end) => {
                write!(f, "Invalid port range: {}-{}", start, end)
            }
            ParseError::EmptyPortSpec => write!(f, "Empty port specification"),
            ParseError::InvalidIpPortFormat(s) => write!(f, "Invalid IP:port format: {}", s),
        }
    }
}

impl Error for ParseError {}

use std::error::Error;

// 定义认证结果类型
#[derive(Debug)]
pub enum AuthenticationResult {
    // 认证成功
    Success,
    // 无需认证
    NoAuthenticationRequired,
    // 认证失败
    Failed,
}

// 定义RTSP错误类型
#[derive(Debug)]
pub enum RtspError {
    // 取消错误
    Cancelled,
    // URL解析错误
    UrlParseError,
    // 超时错误
    TimeoutError(String),
    ConnectionError(String),
    IoError(io::Error),
    AuthenticationError(String),
    ProtocolError(String),
    InvalidIpAddress(String),
    InvalidArgument(String),
}

impl fmt::Display for RtspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RtspError::UrlParseError => write!(f, "Failed to parse RTSP URL"),
            RtspError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            RtspError::IoError(e) => write!(f, "IO error: {}", e),
            RtspError::AuthenticationError(e) => write!(f, "Authentication error: {}", e),
            RtspError::ProtocolError(e) => write!(f, "Protocol error: {}", e),
            RtspError::InvalidIpAddress(e) => write!(f, "Invalid IP address: {}", e),
            RtspError::InvalidArgument(e) => write!(f, "Invalid argument: {}", e),
            RtspError::Cancelled => write!(f, "Operation cancelled"),
            RtspError::TimeoutError(e) => write!(f, "Timeout error: {}", e),
        }
    }
}

impl Error for RtspError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            RtspError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// 从IO错误转换
impl From<io::Error> for RtspError {
    fn from(error: io::Error) -> Self {
        RtspError::IoError(error)
    }
}

// impl From<tokio::time::error::Elapsed> for RtspError {
//     fn from(error: tokio::time::error::Elapsed) -> Self {
//         RtspError::TimeoutError(error.to_string())
//     }
// }

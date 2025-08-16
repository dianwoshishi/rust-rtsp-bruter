use std::error::Error as StdError;
use std::fmt;
use std::io;

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

    // 连接错误
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
        }
    }
}

use std::error::Error;

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

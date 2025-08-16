use crate::auth;
use crate::common::{build_rtsp_request, parse_sdp_content, read_response, send_request};
use crate::error::{AuthenticationResult, RtspError};
use rand::Rng;
use std::marker::Send;
use std::pin::Pin;
use tokio;
use tokio::net::TcpStream;
use url::Url;

// RTSP客户端
pub struct RtspClient {
    username: String,
    password: String,
}

// 随机User-Agent列表
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36",
];

// RTSP响应类型枚举
#[derive(PartialEq)]
enum RtspResponseType {
    Unauthorized,
    Ok,
    Other(String),
}

impl RtspClient {
    pub fn new(username: &str, password: &str) -> Self {
        RtspClient {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    // 随机选择一个User-Agent
    pub fn select_random_user_agent(&self) -> &'static str {
        let user_agent = {
            let mut rng = rand::thread_rng();
            let random_index = rng.r#gen_range(0..USER_AGENTS.len());
            USER_AGENTS[random_index]
        };
        log::trace!("Selected User-Agent: {}", user_agent);
        user_agent
    }

    // 构建RTSP请求的辅助方法
    pub fn build_request(
        &self,
        method: &str,
        host: &str,
        port: u16,
        path: &str,
        user_agent: &str,
        cseq: u32,
        auth_header: Option<&str>,
    ) -> String {
        let full_url = format!("rtsp://{}:{}{}", host, port, path);
        build_rtsp_request(
            method,
            &full_url,
            host,
            port,
            cseq,
            user_agent,
            auth_header,
        )
    }

    // 解析RTSP响应类型
    fn parse_response_type(&self, response: &str) -> RtspResponseType {
        if response.contains("401 Unauthorized") {
            RtspResponseType::Unauthorized
        } else if response.contains("200 OK") {
            RtspResponseType::Ok
        } else {
            RtspResponseType::Other(response.to_string())

        }
    }

    // 发送请求并处理响应的通用方法
    pub fn send_and_process_request<'a>(
        &'a self,
        stream: &'a mut TcpStream,
        request: &'a str,
        host: &'a str,
        port: u16,
        path: &'a str,
        user_agent: &'a str,
        auth_header: Option<&'a str>
    ) -> Pin<Box<dyn futures::Future<Output = Result<AuthenticationResult, RtspError>> + Send + 'a>> {
        Box::pin(async move {
            // 发送请求
            send_request(stream, request).await?;

            // 读取响应
            let response = read_response(stream).await?;

            match self.parse_response_type(&response) {
                RtspResponseType::Unauthorized => {
                    log::debug!("Unauthorized response received");
                    // 通过auth_header设置判断是否已经进行过一次无认证头的认证。
                    // 如果有认证头，说明已经自动解析过一次了。则直接返回失败。

                    match auth_header {
                        // 无认证头，则需要根据响应进一步认证
                        None => {
                            self.handle_auth(
                                stream,
                                &response,
                                host,
                                port,
                                path,
                                user_agent,
                                "DESCRIBE",
                                2,
                            ).await
                        }
                        //有认证头，说明已经认证过一次了，直接返回失败
                        Some(_) => Ok(AuthenticationResult::Failed),
                    }
                }
                RtspResponseType::Ok => {
                    log::debug!("Ok response received");
                    parse_sdp_content(&response, true);
                    // 解析认证类型
                    match auth_header {
                        None => Ok(AuthenticationResult::NoAuthenticationRequired),
                        Some(_) => Ok(AuthenticationResult::Success)
                    }
                }
                RtspResponseType::Other(msg) => {
                    log::debug!("Other response received: {}", msg);
                    Err(RtspError::ProtocolError(msg))
                }
            }
        })
    }

    // 通用认证处理方法
    async fn handle_auth<'a>(
        &'a self,
        stream: &'a mut TcpStream,
        response: &'a str,
        host: &'a str,
        port: u16,
        path: &'a str,
        user_agent: &'a str,
        method: &'a str,
        cseq: u32,
    ) -> Result<AuthenticationResult, RtspError> {
        log::debug!("Handling authentication for {} request", method);
        let auth_type = auth::parse_auth_challenge(response)?;
        
        // 生成完整URL
        let full_url = format!("rtsp://{}:{}{}", host, port, path);
        
        // 生成认证头
        let auth_header = auth::generate_auth_header(
            &auth_type,
            &self.username,
            &self.password,
            method,
            &full_url,
        )?;
        log::debug!("Generated authentication header: {}", auth_header);

        // 构建带认证的请求
        let authenticated_request = build_rtsp_request(
            method,
            &full_url,
            host,
            port,
            cseq,
            user_agent,
            Some(&auth_header),
        );

        // 发送认证请求并处理响应
        self.send_and_process_request(
            stream,
            &authenticated_request,
            host,
            port,
            path,
            user_agent,
            Some(&auth_header),
        ).await
    }

    // 发送DESCRIBE请求，返回认证结果
    pub async fn describe(&self, url: &str) -> Result<AuthenticationResult, RtspError> {
        log::debug!("Parsing RTSP URL: {}", url);
        let parsed_url = Url::parse(url).map_err(|_| RtspError::UrlParseError)?;
        let host = parsed_url.host_str().ok_or(RtspError::UrlParseError)?;
        let port = parsed_url.port().unwrap_or(554);
        let path = parsed_url.path().to_string();

        log::debug!(
            "Parsed URL - Host: {}, Port: {}, Path: {}",
            host,
            port,
            path
        );

        // 连接到RTSP服务器
        let addr = format!("{}:{}", host, port);
        log::debug!("Connecting to RTSP server at {}", addr);
        // 设置连接超时为5秒
        let mut stream =
            tokio::time::timeout(std::time::Duration::from_secs(5), TcpStream::connect(addr))
                .await
                .map_err(|_| RtspError::ConnectionError("Connection timeout".to_string()))?
                .map_err(|e| {
                    RtspError::ConnectionError(format!("Failed to connect to RTSP server: {}", e))
                })?;
        log::debug!("Connected to RTSP server");

        // 选择随机User-Agent
        let user_agent = self.select_random_user_agent();

        let auth_header: Option<&str> = None;
        // 构建初始RTSP请求
        let request = self.build_request("DESCRIBE", host, port, &path, user_agent, 1, auth_header);


        // 使用通用方法发送请求并处理响应
        return self.send_and_process_request(
            &mut stream,
            &request,
            host,
            port,
            &path,
            user_agent,
            auth_header
        ).await;
    }
}



#[cfg(test)]
mod tests {
    // use super::*;
    // use tokio::test;

    // 测试成功连接到不需要认证的RTSP服务器
    // #[test]
    // async fn test_describe_no_auth() {
    //     // 注意：这个测试需要一个真实的不需要认证的RTSP服务器
    //     // 在实际运行测试前，你可能需要修改这个URL
    //     let url = "rtsp://211.79.64.12:554"; 
    //     let client = RtspClient::new("", "");

    //     let result = client.describe(url).await;
    //     assert!(result.is_ok());
    //     let auth_result = result.unwrap();
    //     match auth_result {
    //         AuthenticationResult::NoAuthenticationRequired => {}
    //         _ => panic!("Expected Success, got {:?}", auth_result)
    //     }
    // }

    // // 测试连接到需要认证的RTSP服务器但提供错误凭据，预期认证失败
    // #[test]
    // async fn test_describe_auth_failed() {
    //     // 注意：这个测试需要一个真实的需要认证的RTSP服务器
    //     // 在实际运行测试前，你可能需要修改这个URL和凭据
    //     let url = "rtsp://119.49.2.87:554";
    //     let client = RtspClient::new("invalid_user", "invalid_password");

    //     let result = client.describe(url).await;
    //     assert!(result.is_ok());
    //     let auth_result = result.unwrap();
    //     match auth_result {
    //         AuthenticationResult::Failed => {}
    //         _ => panic!("Expected Failed, got {:?}", auth_result),
    //     }
    // }

    // // 测试连接到需要认证的RTSP服务器但提供错误凭据
    // #[test]
    // async fn test_describe_auth_failure() {
    //     // 注意：这个测试需要一个真实的需要认证的RTSP服务器
    //     // 在实际运行测试前，你可能需要修改这个URL和凭据
    //     let url = "rtsp://example.com/auth";
    //     let client = RtspClient::new("invalid_user", "invalid_password");

    //     let result = client.describe(url).await;
    //     assert!(result.is_ok());
    //     match result.unwrap() {
    //         AuthenticationResult::Failed => {}
    //         _ => panic!("Expected Failed, got {:?}", result),
    //     }
    // }

    // // 测试连接到不存在的RTSP服务器
    // #[test]
    // async fn test_describe_connection_error() {
    //     let url = "rtsp://non_existent_domain_123456789:554/stream";
    //     let client = RtspClient::new("user", "pass");

    //     let result = client.describe(url).await;
    //     assert!(result.is_err());
    //     match result.unwrap_err() {
    //         RtspError::ConnectionError(_) => {}
    //         _ => panic!("Expected ConnectionError, got {:?}", result),
    //     }
    // }

    // // 测试无效的RTSP URL
    // #[test]
    // async fn test_describe_invalid_url() {
    //     let url = "invalid_rtsp_url";
    //     let client = RtspClient::new("user", "pass");

    //     let result = client.describe(url).await;
    //     assert!(result.is_err());
    //     match result.unwrap_err() {
    //         RtspError::UrlParseError => {}
    //         _ => panic!("Expected UrlParseError, got {:?}", result),
    //     }
    // }
}

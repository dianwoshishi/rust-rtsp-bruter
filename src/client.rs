use std::error::Error;
use std::net::ToSocketAddrs;
use tokio;
use tokio::net::TcpStream;
use chrono::Utc;
use rand::Rng;
use rand::rngs::ThreadRng;
use url::Url;
use crate::error::RtspError;
use crate::auth::{self, AuthType};
use crate::common::{self, build_rtsp_request, send_request, read_response, parse_sdp_content};

// RTSP客户端
pub struct RtspClient {
    username: String,
    password: String,
}

impl RtspClient {
    pub fn new(username: &str, password: &str) -> Self {
        RtspClient {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    // 发送DESCRIBE请求
    pub async fn describe(&self, url: &str) -> Result<(), RtspError> {
        log::debug!("Parsing RTSP URL: {}", url);
        let parsed_url = Url::parse(url).map_err(|_| RtspError::UrlParseError)?;
        let host = parsed_url.host_str().ok_or(RtspError::UrlParseError)?;
        let port = parsed_url.port().unwrap_or(554);
        let path = parsed_url.path().to_string();

        log::debug!("Parsed URL - Host: {}, Port: {}, Path: {}", host, port, path);

        // 连接到RTSP服务器
        let addr = format!("{}:{}", host, port);
        log::debug!("Connecting to RTSP server at {}", addr);
        // 设置连接超时为5秒
        let mut stream = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            TcpStream::connect(addr)
        )
        .await
        .map_err(|_| RtspError::ConnectionError("Connection timeout".to_string()))?
        .map_err(|e| {
            RtspError::ConnectionError(format!("Failed to connect to RTSP server: {}", e))
        })?;
        log::info!("Connected to RTSP server");

        // 随机User-Agent列表
        let user_agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36"
        ];

        // 随机选择一个User-Agent
        // 生成随机用户代理 (在await前完成所有随机操作)
        let user_agent = {
            let mut rng = rand::thread_rng();
            let random_index = rng.r#gen_range(0..user_agents.len());
            user_agents[random_index]
        };
        log::debug!("Selected User-Agent: {}", user_agent);

        // 构建RTSP请求
        let full_url = format!("rtsp://{}:{}{}", host, port, path);
        let request = build_rtsp_request(
            "DESCRIBE",
            &full_url,
            host,
            port,
            1,
            user_agent,
            None
        );

        // 发送请求
        send_request(&mut stream, &request).await?;

        // 读取响应
        let response = read_response(&mut stream).await?;

        // 检查是否需要认证
        if response.contains("401 Unauthorized") {
            log::info!("Authentication required");
            let auth_type = auth::parse_auth_challenge(&response)?;
            // 生成认证头，使用完整URL作为路径参数
            let full_url = format!("rtsp://{}:{}{}", host, port, path);
            let auth_header = auth::generate_auth_header(&auth_type, &self.username, &self.password, "DESCRIBE", &full_url)?;
            log::debug!("Generated authentication header: {}", auth_header);

            // 构建带认证的请求
            let full_url = format!("rtsp://{}:{}{}", host, port, path);
            let authenticated_request = build_rtsp_request(
                "DESCRIBE",
                &full_url,
                host,
                port,
                2,
                user_agent,
                Some(&auth_header)
            );

            // 发送认证请求
            send_request(&mut stream, &authenticated_request).await?;

            // 读取认证响应
            let auth_response = read_response(&mut stream).await?;

            if auth_response.contains("200 OK") {
                log::info!("Authentication successful");
                println!("Authentication successful: Valid credentials provided");
                // 解析SDP内容
                parse_sdp_content(&auth_response, true);
            } else {
                return Err(RtspError::AuthenticationError(
                    "Authentication failed".to_string()
                ));
            }
        } else if response.contains("200 OK") {
            log::info!("No authentication required");
            // 解析SDP内容
            parse_sdp_content(&response, false);
        } else {
            return Err(RtspError::ProtocolError(format!(
                "Unexpected response: {}", response
            )));
        }

        Ok(())
    }
}
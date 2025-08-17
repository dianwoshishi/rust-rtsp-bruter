use crate::errors::errors::RtspError;
use chrono::Utc;
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;


// 随机User-Agent列表
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36",
];
// 随机选择一个User-Agent
pub fn select_random_user_agent() -> &'static str {
    let user_agent = {
        let mut rng = rand::thread_rng();
        let random_index = rng.r#gen_range(0..USER_AGENTS.len());
        USER_AGENTS[random_index]
    };
    log::trace!("Selected User-Agent: {}", user_agent);
    user_agent
}

// 构建RTSP请求
pub fn build_rtsp_request(
    method: &str,
    url: &str,
    host: &str,
    port: u16,
    cseq: u32,
    auth_header: Option<&str>,
) -> String {
    let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let mut request = format!("{} {} RTSP/1.0\r\n", method, url)
        + &format!("CSeq: {}\r\n", cseq)
        + &format!("Host: {}:{}\r\n", host, port)
        + &format!("Date: {}\r\n", date)
        + &format!("User-Agent: {}\r\n", select_random_user_agent())
        + &format!("Accept: application/sdp\r\n")
        + &format!("Transport: RTP/AVP;unicast;client_port=8000-8001\r\n");

    if let Some(auth) = auth_header {
        request += &format!("Authorization: {}\r\n", auth);
    }

    request + &format!("\r\n")
}

// 发送RTSP请求
pub async fn send_request(stream: &mut TcpStream, request: &str) -> Result<(), RtspError> {
    log::trace!("Sending RTSP request:\n{}", request.replace("\r\n", "\n"));
    // 发送请求，设置10秒超时
    time::timeout(
        std::time::Duration::from_secs(10),
        stream.write_all(request.as_bytes()),
    )
    .await
    .map_err(|_| RtspError::ConnectionError("Write timeout".to_string()))?
    .map_err(|e| RtspError::IoError(e))?;
    log::debug!(
        "{} request sent",
        if request.contains("Authorization") {
            "Authenticated"
        } else {
            ""
        }
    );
    Ok(())
}

// 读取RTSP响应
pub async fn read_response(stream: &mut TcpStream) -> Result<String, RtspError> {
    let mut buffer = [0; 4096];
    log::debug!("Waiting for response from server");
    let n = time::timeout(std::time::Duration::from_secs(10), stream.read(&mut buffer))
        .await
        .map_err(|_| RtspError::ConnectionError("Read timeout".to_string()))?
        .map_err(|e| RtspError::IoError(e))?;

    if n == 0 {
        log::debug!("Received empty response (0 bytes) - server closed connection");
        return Err(RtspError::ProtocolError(
            "Empty response received from server".to_string(),
        ));
    }

    let response = String::from_utf8_lossy(&buffer[..n]).to_string();
    log::debug!(
        "Received response ({} bytes):\n{}",
        n,
        response.replace("\r\n", "\n")
    );
    log::debug!("RTSP response received");
    Ok(response)
}

// 解析SDP内容
pub fn parse_sdp_content(response: &str) {
    if let Some(sdp_start) = response.find("\r\n\r\n") {
        let sdp_content = &response[sdp_start + 4..];
        log::debug!("Received SDP content:\n{}", sdp_content);
    } else {
        log::debug!("No SDP content found in response. Response: {}", response);
    }
}

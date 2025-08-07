use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time;
use crate::error::RtspError;
use chrono::Utc;

// 构建RTSP请求
pub fn build_rtsp_request(
    method: &str,
    url: &str,
    host: &str,
    port: u16,
    cseq: u32,
    user_agent: &str,
    auth_header: Option<&str>,
) -> String {
    let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let mut request = format!(
        "{} {} RTSP/1.0\r\n",
        method, url
    ) + &format!("CSeq: {}\r\n", cseq)
      + &format!("Host: {}:{}\r\n", host, port)
      + &format!("Date: {}\r\n", date)
      + &format!("User-Agent: {}\r\n", user_agent)
      + &format!("Accept: application/sdp\r\n")
      + &format!("Transport: RTP/AVP;unicast;client_port=8000-8001\r\n");

    if let Some(auth) = auth_header {
        request += &format!("Authorization: {}\r\n", auth);
    }

    request + &format!("\r\n")
}

// 发送RTSP请求
pub async fn send_request(
    stream: &mut TcpStream,
    request: &str,
) -> Result<(), RtspError> {
    log::debug!("Sending RTSP request:\n{}", request.replace("\r\n", "\n"));
    // 发送请求，设置10秒超时
    time::timeout(
        std::time::Duration::from_secs(10),
        stream.write_all(request.as_bytes())
    )
    .await
    .map_err(|_| RtspError::ConnectionError("Write timeout".to_string()))?
    .map_err(|e| {
        RtspError::IoError(e)
    })?;
    log::info!("{} request sent", if request.contains("Authorization") { "Authenticated" } else { "" });
    Ok(())
}

// 读取RTSP响应
pub async fn read_response(
    stream: &mut TcpStream,
) -> Result<String, RtspError> {
    let mut buffer = [0; 4096];
    log::debug!("Waiting for response from server");
    let n = time::timeout(
        std::time::Duration::from_secs(10),
        stream.read(&mut buffer)
    )
    .await
    .map_err(|_| RtspError::ConnectionError("Read timeout".to_string()))?
    .map_err(|e| {
        RtspError::IoError(e)
    })?;

    if n == 0 {
        log::debug!("Received empty response (0 bytes) - server closed connection");
        return Err(RtspError::ProtocolError(
            "Empty response received from server".to_string()
        ));
    }

    let response = String::from_utf8_lossy(&buffer[..n]).to_string();
    log::debug!("Received response ({} bytes):\n{}", n, response.replace("\r\n", "\n"));
    log::info!("RTSP response received");
    Ok(response)
}

// 解析SDP内容
pub fn parse_sdp_content(response: &str, auth_success: bool) {
    if let Some(sdp_start) = response.find("\r\n\r\n") {
        let sdp_content = &response[sdp_start + 4..];
        log::info!("Received SDP content:\n{}", sdp_content);
        if auth_success {
            println!("Success: Received media description (SDP)");
        } else {
            println!("Success: Received media description (SDP) - no authentication required");
        }
    } else {
        log::info!("No SDP content found in response");
        println!("Warning: No media description (SDP) found in response");
    }
}
use crate::error::RtspError;
use base64::Engine;
use md5::{Digest, Md5};
use rand::Rng;

// RTSP认证类型
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AuthType {
    None,
    Basic(()),
    Digest(DigestAuthInfo),
}

// Digest认证信息
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DigestAuthInfo {
    pub realm: String,
    pub nonce: String,
    pub qop: Option<String>,
    pub algorithm: Option<String>,
    // pub opaque: Option<String>,
}

// 认证工具函数
pub fn parse_auth_challenge(response: &str) -> Result<AuthType, RtspError> {
    // 查找WWW-Authenticate头
    for line in response.lines() {
        if let Some(auth_header) = line.strip_prefix("WWW-Authenticate: ") {
            let auth_str = auth_header.trim();

            if auth_str.starts_with("Basic ") {
                log::info!("Basic authentication required");
                return Ok(AuthType::Basic(()));
            } else if auth_str.starts_with("Digest ") {
                let digest_info = parse_digest_challenge(&auth_str["Digest ".len()..])?;
                log::info!(
                    "Digest authentication required, realm: {}",
                    digest_info.realm
                );
                return Ok(AuthType::Digest(digest_info));
            }
        }
    }

    Ok(AuthType::None)
}

// 解析Digest认证挑战
pub fn parse_digest_challenge(challenge: &str) -> Result<DigestAuthInfo, RtspError> {
    let mut realm = String::new();
    let mut nonce = String::new();
    let mut qop = None;
    let mut algorithm = None;
    // let mut opaque = None;

    for param in challenge.split(",") {
        let parts: Vec<&str> = param.trim().splitn(2, "=").collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].trim();
        let value = parts[1].trim().trim_matches('"');

        match key {
            "realm" => realm = value.to_string(),
            "nonce" => nonce = value.to_string(),
            "qop" => qop = Some(value.to_string()),
            "algorithm" => algorithm = Some(value.to_string()),
            // "opaque" => opaque = Some(value.to_string()),
            _ => {}
        }
    }

    if realm.is_empty() || nonce.is_empty() {
        return Err(RtspError::AuthenticationError(
            "Invalid Digest challenge: missing realm or nonce".to_string(),
        ));
    }

    Ok(DigestAuthInfo {
        realm,
        nonce,
        qop,
        algorithm,
        // opaque,
    })
}

// 生成认证头
pub fn generate_auth_header(
    auth_type: &AuthType,
    username: &str,
    password: &str,
    method: &str,
    path: &str,
) -> Result<String, RtspError> {
    match auth_type {
        AuthType::Basic(_) => {
            // Basic认证: base64(username:password)
            let credentials = format!("{}:{}", username, password);
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
            Ok(format!("Basic {}", encoded))
        }
        AuthType::Digest(info) => {
            // Digest认证
            let algorithm = info.algorithm.as_deref().unwrap_or("MD5");
            let cnonce = generate_cnonce();
            let nc = "00000001";
            let qop = info.qop.as_deref().unwrap_or("auth");

            // 计算HA1 = MD5(username:realm:password)
            let ha1_input = format!("{}:{}:{}", username, info.realm, password);
            let ha1 = format!("{:x}", Md5::digest(ha1_input.as_bytes()));

            // 计算HA2 = MD5(method:path)
            let ha2_input = format!("{}:{}", method, path);
            let ha2 = format!("{:x}", Md5::digest(ha2_input.as_bytes()));

            // 计算response = MD5(ha1:nonce:nc:cnonce:qop:ha2)
            let response_input =
                format!("{}:{}:{}:{}:{}:{}", ha1, info.nonce, nc, cnonce, qop, ha2);
            let response = format!("{:x}", Md5::digest(response_input.as_bytes()));

            // 构建Digest认证头
            let digest_header = format!(
                "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\", response=\"{}\", algorithm=\"{}\", cnonce=\"{}\", nc={}, qop=\"{}\"",
                username, info.realm, info.nonce, path, response, algorithm, cnonce, nc, qop
            );

            Ok(digest_header)
        }
        AuthType::None => Err(RtspError::AuthenticationError(
            "No authentication type specified".to_string(),
        )),
    }
}

// 生成客户端随机数
pub fn generate_cnonce() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 16] = rng.r#gen();
    format!("{:x}", Md5::digest(&random_bytes))
}

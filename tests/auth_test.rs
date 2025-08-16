use rust_rtsp_bruter::rtsp::auth::{parse_auth_challenge, generate_auth_header, AuthType, DigestAuthInfo};
use md5::Digest;

// 测试Digest认证响应生成
#[test]
fn test_digest_auth_response_generation() {
    // 从用户提供的认证过程中提取参数
    let username = "admin";
    let password = "123456";
    let realm = "RTSP SERVER";
    let nonce = "72fb3f3f23ded5a9d8f9be5a4535bf84";
    let method = "DESCRIBE";
    let path = "rtsp://60.243.26.171:555";
    let expected_response = "bacab71a291d142b597913dc07633ea6";

    // 手动计算Digest响应值以进行调试
    let ha1_input = format!("{}:{}:{}", username, realm, password);
    let ha1 = format!("{:x}", md5::Md5::digest(ha1_input.as_bytes()));
    println!("HA1: {}", ha1);

    let ha2_input = format!("{}:{}", method, path);
    let ha2 = format!("{:x}", md5::Md5::digest(ha2_input.as_bytes()));
    println!("HA2: {}", ha2);

    let response_input = format!("{}:{}:{}", ha1, nonce, ha2);
    let response = format!("{:x}", md5::Md5::digest(response_input.as_bytes()));
    println!("Calculated response with qop=auth: {}", response);
    println!("Expected response: {}", expected_response);
    assert_eq!(response, expected_response);

    // 构建Digest认证信息
    let auth_type = AuthType::Digest(DigestAuthInfo {
        realm: realm.to_string(),
        nonce: nonce.to_string(),
        qop: None,
        algorithm: None,
        opaque: None,
    });

    // 生成认证头
    let auth_header = generate_auth_header(&auth_type, username, password, method, path).unwrap();
    println!("Generated auth header: {}", auth_header);

    // 验证响应值是否匹配预期
    assert!(auth_header.contains(&format!("response=\"{}\"", expected_response)),
        "Response does not match expected value. Expected: {}, Got: {}",
        expected_response,
        auth_header
    );
}

// 测试完整认证流程解析
#[test]
fn test_full_auth_flow_parsing() {
    // 原始401响应
    let unauthorized_response = "RTSP/1.0 401 Unauthorized\r\n"
        .to_string() + "CSeq: 3\r\n"
        + "Date: Sat, Aug 16 2025 01:19:15 GMT\r\n"
        + "Expires: Sat, Aug 16 2025 01:19:15 GMT\r\n"
        + "WWW-Authenticate: Digest realm=\"RTSP SERVER\", nonce=\"72fb3f3f23ded5a9d8f9be5a4535bf84\", stale=\"FALSE\"\r\n\r\n";

    // 解析认证挑战
    let auth_type = parse_auth_challenge(&unauthorized_response).unwrap();

    // 验证解析结果
    match &auth_type {
        AuthType::Digest(info) => {
            assert_eq!(info.realm, "RTSP SERVER");
            assert_eq!(info.nonce, "72fb3f3f23ded5a9d8f9be5a4535bf84");
        },
        _ => panic!("Expected Digest authentication type"),
    }

    // 生成认证头
    let username = "admin";
    let password = "123456";
    let method = "DESCRIBE";
    let path = "rtsp://60.243.26.171:555";
    let auth_header = generate_auth_header(&auth_type, username, password, method, path).unwrap();

    // 验证认证头包含必要的字段
    assert!(auth_header.starts_with("Digest "));
    assert!(auth_header.contains(&format!("username=\"{}\"", username)));
    assert!(auth_header.contains(&"realm=\"RTSP SERVER\"".to_string()));
    assert!(auth_header.contains(&"nonce=\"72fb3f3f23ded5a9d8f9be5a4535bf84\"".to_string()));
    assert!(auth_header.contains(&format!("uri=\"{}\"", path)));
    assert!(auth_header.contains("response=\""));
}
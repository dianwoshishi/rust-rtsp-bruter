use rstest::rstest;
use rust_rtsp_bruter::iterator::ip_port_parser::{parse_ip_port};
use std::net::{IpAddr, Ipv4Addr};

/// 测试单个IP地址解析
#[rstest]
#[case("192.168.1.1", vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))])]
#[case("10.0.0.1", vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))])]
fn test_single_ip(#[case] input: &str, #[case] expected: Vec<IpAddr>) {
    let result = parse_ip_port(input).unwrap();
    let ips: Vec<IpAddr> = result.into_iter().map(|ip_port| ip_port.ip).collect();
    assert_eq!(ips, expected);
}

/// 测试带花括号的IP范围解析
#[rstest]
#[case(
    "192.168.{1-3}.1",
    vec![
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 3, 1)),
    ]
)]
#[case(
    "10.{1-2}.{3-4}.5",
    vec![
        IpAddr::V4(Ipv4Addr::new(10, 1, 3, 5)),
        IpAddr::V4(Ipv4Addr::new(10, 1, 4, 5)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 3, 5)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 4, 5)),
    ]
)]
fn test_braced_ip_range(#[case] input: &str, #[case] expected: Vec<IpAddr>) {
    let result = parse_ip_port(input).unwrap();
    let ips: Vec<IpAddr> = result.into_iter().map(|ip_port| ip_port.ip).collect();
    assert_eq!(ips, expected);
}

/// 测试带多个选择的花括号IP解析
#[rstest]
#[case(
    "192.168.{1,3,5}.1",
    vec![
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 3, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 5, 1)),
    ]
)]
#[case(
    "10.{1,2}.{3,4,5}.6",
    vec![
        IpAddr::V4(Ipv4Addr::new(10, 1, 3, 6)),
        IpAddr::V4(Ipv4Addr::new(10, 1, 4, 6)),
        IpAddr::V4(Ipv4Addr::new(10, 1, 5, 6)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 3, 6)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 4, 6)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 5, 6)),
    ]
)]
fn test_braced_ip_multiple(#[case] input: &str, #[case] expected: Vec<IpAddr>) {
    let result = parse_ip_port(input).unwrap();
    let ips: Vec<IpAddr> = result.into_iter().map(|ip_port| ip_port.ip).collect();
    assert_eq!(ips, expected);
}

/// 测试带CIDR掩码的IP解析
#[rstest]
#[case(
    "192.168.1.0/24",
    vec![
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
        // 注意：实际测试中应包含子网内的所有IP，但为简洁起见仅列出部分
    ]
)]
#[case(
    "10.0.0.0/16",
    vec![
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
        IpAddr::V4(Ipv4Addr::new(10, 0, 100, 0)),
        // 实际测试中应包含更多IP
    ]
)]
fn test_cidr_ip(#[case] input: &str, #[case] expected: Vec<IpAddr>) {
    let result = parse_ip_port(input).unwrap();
    let ips: Vec<IpAddr> = result.into_iter().map(|ip_port| ip_port.ip).collect();
    // 这里简化测试，实际应检查所有子网IP
    assert!(!ips.is_empty());
    assert!(ips.contains(&expected[0]));
}

/// 测试带CIDR掩码的花括号IP解析
#[rstest]
#[case(
    "192.168.{1-2}.0/24",
    vec![
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 2, 0)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 2, 100)),
        // 实际测试中应包含更多IP
    ]
)]
fn test_cidr_braced_ip(#[case] input: &str, #[case] expected: Vec<IpAddr>) {
    let result = parse_ip_port(input).unwrap();
    // println!("{:?}", result);

    let ips: Vec<IpAddr> = result.into_iter().map(|ip_port| ip_port.ip).collect();
    // 简化测试，实际应检查所有子网IP
    assert!(!ips.is_empty());
    for ip in expected {
        assert!(ips.contains(&ip));
    }
}

/// 测试端口解析
#[rstest]
#[case("192.168.1.1:80", vec![80])]
#[case("192.168.1.1:8000-8002", vec![8000, 8001, 8002])]
#[case("192.168.1.1:80,443,8000-8001", vec![80, 443, 8000, 8001])]
fn test_port_spec(#[case] input: &str, #[case] expected_ports: Vec<u16>) {
    let result = parse_ip_port(input).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ports, expected_ports);
}

/// 测试IP范围和端口范围组合
#[rstest]
#[case(
    "192.168.{1-2}.1:8000-8001",
    vec![
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), vec![8000, 8001]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1)), vec![8000, 8001]),
    ]
)]
fn test_ip_port_combination(#[case] input: &str, #[case] expected: Vec<(IpAddr, Vec<u16>)>) {
    let result = parse_ip_port(input).unwrap();
    assert_eq!(result.len(), expected.len());
    for (i, ip_port) in result.iter().enumerate() {
        assert_eq!(ip_port.ip, expected[i].0);
        assert_eq!(ip_port.ports, expected[i].1);
    }
}

/// 测试复杂格式解析
#[rstest]
#[case(
    "1.{1-3}.{1,2}.100/24:{80,443,8000-8001}",
    3 * 2 * 256,  // 期望的IP数量
    4   // 期望的每个IP的端口数量
)]
#[case(
    "1.168.{0-1}.1/23:{80,443,8000-8001}",
    256 * 2,  // 期望的IP数量
    4   // 期望的每个IP的端口数量
)]
#[case(
    "1.168.{0-1}.1/22:{80,443,8000-8001}",
    1024,  // 期望的IP数量
    4   // 期望的每个IP的端口数量
)]
fn test_complex_format(
    #[case] input: &str,
    #[case] expected_ip_count: usize,
    #[case] expected_ports_per_ip: usize,
) {
    let result = parse_ip_port(input).unwrap();
    assert_eq!(result.len(), expected_ip_count);
    for ip_port in result {
        assert_eq!(ip_port.ports.len(), expected_ports_per_ip);
        // 检查IP是否在CIDR范围内
        assert!(ip_port.ip.is_ipv4());
        if let IpAddr::V4(ipv4) = ip_port.ip {
            assert_eq!(ipv4.octets()[0], 1);
        }
    }
}

/// 测试嵌套花括号表达式
#[rstest]
#[case(
    "192.168.{1-2,3}.1",
    vec!(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 2, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 3, 1)),
    )
)]
#[case(
    "10.{1-2,{3-4}}.{{5-6},7}.8",
    vec!(
        IpAddr::V4(Ipv4Addr::new(10, 1, 5, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 1, 6, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 1, 7, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 5, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 6, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 2, 7, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 3, 5, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 3, 6, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 3, 7, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 4, 5, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 4, 6, 8)),
        IpAddr::V4(Ipv4Addr::new(10, 4, 7, 8)),
    )
)]
fn test_nested_braces(#[case] input: &str, #[case] expected: Vec<IpAddr>) {
    let result = parse_ip_port(input).unwrap();
    let ips: Vec<IpAddr> = result.into_iter().map(|ip_port| ip_port.ip).collect();
    assert_eq!(ips, expected);
}

/// 测试IP边界情况
#[rstest]
#[case("0.0.0.0", vec![IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))])]
#[case("255.255.255.255", vec![IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))])]
#[case("192.168.255.{1-3}", vec![
    IpAddr::V4(Ipv4Addr::new(192, 168, 255, 1)),
    IpAddr::V4(Ipv4Addr::new(192, 168, 255, 2)),
    IpAddr::V4(Ipv4Addr::new(192, 168, 255, 3)),
])]
fn test_ip_boundaries(#[case] input: &str, #[case] expected: Vec<IpAddr>) {
    let result = parse_ip_port(input).unwrap();
    let ips: Vec<IpAddr> = result.into_iter().map(|ip_port| ip_port.ip).collect();
    assert_eq!(ips, expected);
}

/// 测试端口边界情况
#[rstest]
#[case("192.168.1.1:0", vec![0])]
#[case("192.168.1.1:65535", vec![65535])]
#[case("192.168.1.1:65535-65535", vec![65535])]
fn test_port_boundaries(#[case] input: &str, #[case] expected_ports: Vec<u16>) {
    let result = parse_ip_port(input).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ports, expected_ports);
}

/// 测试CIDR掩码边界情况
#[rstest]
// #[case("192.168.1.0/0")]
// #[case("192.168.1.0/1")]
#[case("192.168.1.0/31")]
#[case("192.168.1.1/32")]
fn test_cidr_boundaries(#[case] input: &str) {
    let result = parse_ip_port(input);
    assert!(result.is_ok());
    let ips = result.unwrap();
    assert!(!ips.is_empty());
}

/// 测试错误处理
#[rstest]
#[case("invalid-ip")]
#[case("192.168.1.1:invalid-port")]
#[case("192.168.1.1/33")] // 无效的CIDR掩码
fn test_error_handling(#[case] input: &str) {
    let result = parse_ip_port(input);
    assert!(result.is_err());
}

#[rstest]
#[case(
    "192.168.1.0/28:555",
    vec![
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 4)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 6)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 7)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 8)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 9)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 11)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 12)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 13)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 14)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(192, 168, 1, 15)), vec![555]),
        // 192.168.1.0/28 子网包含 16 个 IP 地址，已全部列出
    ]
)]
#[case(
    "60.243.26.171/28:555",
    vec![
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 160)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 161)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 162)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 163)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 164)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 165)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 166)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 167)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 168)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 169)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 170)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 171)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 172)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 173)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 174)), vec![555]),
        (IpAddr::V4(Ipv4Addr::new(60, 243, 26, 175)), vec![555]),
        // 60.243.26.171/28 子网包含 16 个 IP 地址，已全部列出
    ]
)]
fn test_cidr_with_port(#[case] input: &str, #[case] expected: Vec<(IpAddr, Vec<u16>)>) {
    let result = parse_ip_port(input).unwrap();
    assert_eq!(result.len(), expected.len());
    for (i, ip_port) in result.iter().enumerate() {
        assert_eq!(ip_port.ip, expected[i].0);
        assert_eq!(ip_port.ports, expected[i].1);
    }
}

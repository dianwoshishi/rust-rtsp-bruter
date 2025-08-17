// 这是一个展示ip_port_parser功能的示例程序
use rust_rtsp_bruter::iterator::ip_port_parser::parse_ip_port;

fn main() {
    // 测试各种IP端口格式
    let test_cases = [
        // 单个IP和端口
        "192.168.1.1:80",
        // IP范围和端口范围
        "192.168.1.1-5:8000-8002",
        // 带CIDR掩码的IP
        "192.168.1.0/24:8080",
        // 复杂的IP模式（带花括号）
        "10.{1-2,{3-4}}.{{5-6},7}.8:{80,443}",
        // 带CIDR掩码的复杂IP模式
        "192.168.{1-2}.0/28:555",
    ];

    for input in &test_cases {
        println!("\n解析: {}", input);
        match parse_ip_port(input) {
            Ok(ip_ports) => {
                println!("解析成功，找到 {} 个IP端口组合:", ip_ports.len());
                // 只打印前5个结果，避免输出过多
                let display_count = std::cmp::min(ip_ports.len(), 5);
                for i in 0..display_count {
                    let ip_port = &ip_ports[i];
                    println!("  {}. IP: {}, 端口: {:?}", i + 1, ip_port.ip, ip_port.ports);
                }
                if ip_ports.len() > display_count {
                    println!("  ... 以及其他 {} 个组合", ip_ports.len() - display_count);
                }
            }
            Err(e) => {
                println!("解析失败: {:?}", e);
            }
        }
    }
}

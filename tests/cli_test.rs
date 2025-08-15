use rand::Rng;
use rust_rtsp_bruter::cli::Cli;
use rust_rtsp_bruter::cli::parse_brute_args;
use std::io::Write;

// 临时文件结构体，实现Drop特性自动删除文件
struct TempFile {
    path: String,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // 尝试删除文件，忽略错误
        let _ = std::fs::remove_file(&self.path);
    }
}

impl TempFile {
    // 获取文件路径
    fn path(&self) -> &str {
        &self.path
    }
}

// 测试辅助函数：创建临时文件并写入内容
fn create_temp_file(content: &str) -> TempFile {
    // 使用标准库创建临时文件
    let dir = std::env::temp_dir();
    let random_suffix = rand::thread_rng().gen_range(0..u64::MAX);
    let file_path = dir.join(format!("temp_{}.txt", random_suffix));
    let mut file = std::fs::File::create(&file_path).expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    // 返回TempFile实例
    TempFile {
        path: file_path.to_str().unwrap().to_string(),
    }
}

// 测试有效的参数组合：文件+文件
#[test]
fn test_parse_brute_args_files_files() {
    let users_file = create_temp_file("user1\nuser2");
    let passwords_file = create_temp_file("pass1\npass2");
    let ips_file = create_temp_file("192.168.1.1\n192.168.1.2");

    let cli = Cli::Brute {
        users_file: Some(users_file.path().to_string()),
        users_string: None,
        passwords_file: Some(passwords_file.path().to_string()),
        passwords_string: None,
        ips_file: Some(ips_file.path().to_string()),
        ips_string: None,
        max_concurrent: 5,
        delay: 100,
    };

    // 我们不能真正执行brute_force，所以这里只测试参数解析
    // 实际测试中，我们会使用mock对象来模拟BruteForcer
    match parse_brute_args(cli) {
        Ok(_) => assert!(true),
        Err(e) => {
            panic!("Failed to parse brute args: {}", e);
        }
    }
}

// 测试有效的参数组合：文件+字符串
#[test]
fn test_parse_brute_args_files_string() {
    let users_file = create_temp_file("user1\nuser2");
    let password = "password123".to_string();
    let ips_file = create_temp_file("192.168.1.1");

    let cli = Cli::Brute {
        users_file: Some(users_file.path().to_string()),
        users_string: None,
        passwords_file: None,
        passwords_string: Some(password),
        ips_file: Some(ips_file.path().to_string()),
        ips_string: None,
        max_concurrent: 5,
        delay: 100,
    };

    match parse_brute_args(cli) {
        Ok(_) => assert!(true),
        Err(e) => {
            panic!("Failed to parse brute args: {}", e);
        }
    }
}

// 测试有效的参数组合：字符串+文件
#[test]
fn test_parse_brute_args_string_files() {
    let username = "admin".to_string();
    let passwords_file = create_temp_file("pass1\npass2");
    let ips_string = "127.0.0.1".to_string();

    let cli = Cli::Brute {
        users_file: None,
        users_string: Some(username),
        passwords_file: Some(passwords_file.path().to_string()),
        passwords_string: None,
        ips_file: None,
        ips_string: Some(ips_string),
        max_concurrent: 5,
        delay: 100,
    };

    match parse_brute_args(cli) {
        Ok(_) => assert!(true),
        Err(e) => {
            panic!("Failed to parse brute args: {}", e);
        }
    }
}

// 测试有效的参数组合：字符串+字符串
#[test]
fn test_parse_brute_args_string_string() {
    let username = "admin".to_string();
    let password = "password".to_string();
    let ips_string = "127.0.0.1".to_string();

    let cli = Cli::Brute {
        users_file: None,
        users_string: Some(username),
        passwords_file: None,
        passwords_string: Some(password),
        ips_file: None,
        ips_string: Some(ips_string),
        max_concurrent: 5,
        delay: 100,
    };

    assert!(parse_brute_args(cli).is_ok());
}

// 测试无效的参数组合：同时提供ips_file和ips_string
#[test]
fn test_parse_brute_args_invalid_ip_source() {
    let users_file = create_temp_file("user1");
    let passwords_file = create_temp_file("pass1");
    let ips_file = create_temp_file("192.168.1.1");
    let ips_string = "127.0.0.1".to_string();

    let cli = Cli::Brute {
        users_file: Some(users_file.path().to_string()),
        users_string: None,
        passwords_file: Some(passwords_file.path().to_string()),
        passwords_string: None,
        ips_file: Some(ips_file.path().to_string()),
        ips_string: Some(ips_string),
        max_concurrent: 5,
        delay: 100,
    };

    let result = parse_brute_args(cli);
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Either ips_file or ips_string must be provided"));
}

// 测试无效的参数组合：同时提供users_file和users_string
#[test]
fn test_parse_brute_args_invalid_user_source() {
    let users_file = create_temp_file("user1");
    let users_string = "admin".to_string();
    let passwords_file = create_temp_file("pass1");
    let ips_string = "127.0.0.1".to_string();

    let cli = Cli::Brute {
        users_file: Some(users_file.path().to_string()),
        users_string: Some(users_string),
        passwords_file: Some(passwords_file.path().to_string()),
        passwords_string: None,
        ips_file: None,
        ips_string: Some(ips_string),
        max_concurrent: 5,
        delay: 100,
    };

    // 这里我们期望Clap会在解析阶段就失败，而不是在handle_cli中
    // 但为了演示，我们仍然测试这种情况
    let result = parse_brute_args(cli);
    assert!(result.is_err());
}

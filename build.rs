use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // 获取输出目录路径
    let out_dir = env::var("OUT_DIR").expect("Failed to get OUT_DIR");
    println!("cargo:info=OUT_DIR: {}", out_dir);

    // 构建目标目录路径 (向上导航到 target/debug 或 target/release)
    let out_path = PathBuf::from(out_dir);
    let target_dir = match out_path.ancestors().nth(3) {
        Some(path) => path,
        None => {
            eprintln!("cargo:warning=Failed to determine target directory");
            return;
        }
    };

    println!("cargo:info=Target directory: {:?}", target_dir);

    // 定义要复制的文件列表
    let files_to_copy = [
        ("iplist.txt", "iplist.txt"),
        ("log4rs.yaml", "log4rs.yaml"),
        ("config.toml", "config.toml"),
        ("passwords.txt", "passwords.txt"),
        ("users.txt", "users.txt"),
    ];

    // 复制文件
    for (src, dest) in &files_to_copy {
        let src_path = Path::new(src);
        let dest_path = target_dir.join(dest);

        println!("cargo:info=Copying {} to {:?}", src, dest_path);

        match fs::copy(src_path, &dest_path) {
            Ok(bytes) => println!("cargo:info=Successfully copied {} bytes from {} to {:?}", bytes, src, dest_path),
            Err(e) => eprintln!("cargo:warning=Failed to copy {}: {:?}", src, e),
        }
    }

    // 告诉Cargo这个构建脚本需要在文件变化时重新运行
    for (src, _) in &files_to_copy {
        println!("cargo:rerun-if-changed={}", src);
    }
}
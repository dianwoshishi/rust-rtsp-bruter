// 导入计时宏（根据实际路径调整）
use timing_macro::{timing};

/// 无参数同步函数示例
// #[timing]
pub fn example_function(x: i32) {
    std::thread::sleep(std::time::Duration::from_secs(x as u64));
    println!("Example function executed");
}

#[timing]
// 测试宏功能的函数
pub fn test_macro() -> () {
    example_function(1);
    ()
}

#[timing]
pub async fn async_function(){
    example_function(1);

}

// 主函数
#[tokio::main]
async fn main() {
    // 测试计时宏功能

    test_macro();
    async_function().await;

}

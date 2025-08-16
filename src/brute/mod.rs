// 定义brute模块的子模块
pub mod brute_forcer;
pub mod task_manager;

// 重新导出子模块中的类型，方便外部使用
pub use brute_forcer::BruteForcer;
pub use brute_forcer::FoundCredential;

pub mod simple_migrator;

pub use simple_migrator::{
    SimpleMigrator, 
    MigrationRecord, 
    MigrationFile, 
    MigrationSummary, 
    MigrationStatus,
    FailedMigration
};

// 便利的重导出
pub type Result<T> = anyhow::Result<T>;

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// 默认配置
pub struct MigratorConfig {
    pub continue_on_failure: bool,
    pub validate_checksums: bool,
    pub concurrent_file_scan: bool,
}

impl Default for MigratorConfig {
    fn default() -> Self {
        Self {
            continue_on_failure: false,
            validate_checksums: true,
            concurrent_file_scan: true,
        }
    }
}

impl MigratorConfig {
    pub fn from_env() -> Self {
        Self {
            continue_on_failure: std::env::var("CONTINUE_ON_MIGRATION_FAILURE")
                .unwrap_or_default() == "true",
            validate_checksums: std::env::var("VALIDATE_MIGRATION_CHECKSUMS")
                .unwrap_or("true".to_string()) == "true",
            concurrent_file_scan: std::env::var("CONCURRENT_FILE_SCAN")
                .unwrap_or("true".to_string()) == "true",
        }
    }
}
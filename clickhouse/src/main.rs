use clickhouse_connector::{
    database::ClickHouseConnectionManager,
    clickhouse_migrator::SimpleMigrator,
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 检查是否启用详细模式
    let verbose = env::args().any(|arg| arg == "--verbose" || arg == "-v");
    let debug_mode = env::args().any(|arg| arg == "--debug" || arg == "-d");
    
    if verbose {
        println!("🔍 启用详细模式 - 将显示更多调试信息");
    }
    
    if debug_mode {
        println!("🐛 启用调试模式 - 将显示所有日志信息");
        // 设置日志级别
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
    
    println!("🚀 ClickHouse 数据库连接器和迁移工具");
    
    // 创建连接管理器（只创建一次连接）
    let connection_manager = ClickHouseConnectionManager::new(
        "http://localhost:8123",
        "default", 
        "default", 
        "ClickHouse@123"
    )?;
    
    println!("✅ 连接管理器创建成功");
    
    // 使用连接管理器创建数据库实例
    let db = connection_manager.create_db();
    
    // 测试连接
    match db.test_connection().await {
        Ok(true) => println!("✅ 数据库连接测试成功"),
        Ok(false) => println!("❌ 数据库连接测试失败"),
        Err(e) => println!("❌ 数据库连接测试出错: {}", e),
    }
    
    // 使用连接管理器创建迁移器
    let migrator = SimpleMigrator::new(
        "http://localhost:8123",
        "my_service",
        "migrations"
    ).await?;
    
    println!("✅ 迁移器创建成功");
    
    // 获取迁移状态
    match migrator.get_migration_status().await {
        Ok(status) => {
            println!("📊 迁移状态:");
            println!("  服务名称: {}", status.service_name);
            println!("  迁移表: {}", status.migrations_table);
            println!("  已应用迁移数: {}", status.total_migrations);
        }
        Err(e) => println!("❌ 获取迁移状态失败: {}", e),
    }
    
    // 运行迁移
    println!("🔧 开始运行迁移...");
    match migrator.migrate().await {
        Ok(summary) => {
            if summary.is_success() {
                println!("✅ 迁移完成成功!");
                println!("  成功迁移数: {}", summary.successful.len());
                println!("  总耗时: {:?}", summary.total_time);
            } else {
                println!("⚠️  迁移完成，但有失败:");
                println!("  成功迁移数: {}", summary.successful.len());
                println!("  失败迁移数: {}", summary.failed.len());
                println!("  总耗时: {:?}", summary.total_time);
                
                // 显示详细的失败信息
                if !summary.failed.is_empty() {
                    println!("\n❌ 失败的迁移详情:");
                    for (i, failed) in summary.failed.iter().enumerate() {
                        println!("  {}. 版本: {} - {}", i + 1, failed.version, failed.name);
                        println!("     错误: {}", failed.error);
                        println!();
                    }
                }
                
                // 显示成功的迁移信息
                if !summary.successful.is_empty() {
                    println!("✅ 成功的迁移:");
                    for (i, success) in summary.successful.iter().enumerate() {
                        println!("  {}. 版本: {} - {} (耗时: {}ms)", 
                                i + 1, success.version, success.name, success.execution_time_ms);
                    }
                }
                
                // 在详细模式下，尝试获取更多错误信息
                if verbose {
                    println!("\n🔍 详细错误诊断:");
                    match migrator.get_failed_migrations().await {
                        Ok(failed_migrations) => {
                            if !failed_migrations.is_empty() {
                                println!("  数据库中的失败记录:");
                                for failed in failed_migrations {
                                    println!("    - {}: {} (执行时间: {}ms)", 
                                            failed.version, failed.name, failed.execution_time_ms);
                                    if !failed.error_message.is_empty() {
                                        println!("      错误: {}", failed.error_message);
                                    }
                                }
                            }
                        }
                        Err(e) => println!("  无法获取失败记录: {}", e),
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ 迁移失败: {}", e);
            
            // 尝试获取更详细的错误信息
            if let Some(cause) = e.source() {
                println!("   原因: {}", cause);
            }
            
            // 显示错误链
            let mut current_error: &dyn std::error::Error = e.as_ref();
            let mut depth = 1;
            while let Some(source) = current_error.source() {
                println!("   Caused by ({}): {}", depth, source);
                current_error = source;
                depth += 1;
                if depth > 5 { // 限制深度避免无限循环
                    break;
                }
            }
        }
    }
    
    Ok(())
}

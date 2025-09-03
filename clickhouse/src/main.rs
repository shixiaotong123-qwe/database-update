use clickhouse_connector::{
    database::ClickHouseConnectionManager,
    clickhouse_migrator::SimpleMigrator,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
            }
        }
        Err(e) => println!("❌ 迁移失败: {}", e),
    }
    
    Ok(())
}

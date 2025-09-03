use anyhow::Result;
use clickhouse::Client;
use serde::{Deserialize, Serialize};
use clickhouse_connector::SimpleMigrator;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 ClickHouse 数据库连接器启动中...");
    
    // 获取配置
    let database_url = env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".to_string());
    let service_name = env::var("CLICKHOUSE_SERVICE")
        .unwrap_or_else(|_| "clickhouse_service".to_string());
    let migrations_path = env::var("CLICKHOUSE_MIGRATIONS")
        .unwrap_or_else(|_| "./migrations".to_string());
    
    println!("🔧 配置信息:");
    println!("  数据库 URL: {}", database_url);
    println!("  服务名称: {}", service_name);
    println!("  迁移路径: {}", migrations_path);
    println!();

    // 连接ClickHouse数据库
    let client = Client::default()
        .with_url(&database_url)
        .with_database("default")
        .with_user("default")
        .with_password("ClickHouse@123");

    println!("正在连接ClickHouse数据库...");

    // 测试连接
    match client.query("SELECT 1").execute().await {
        Ok(_) => println!("✅ 成功连接到ClickHouse数据库！"),
        Err(e) => {
            println!("❌ 连接失败: {}", e);
            return Ok(());
        }
    }

    // 获取数据库信息
    println!("\n📊 数据库信息:");
    println!("数据库名称: default");
    println!("用户名: default");
    println!("主机: {}", database_url);

    // 🔥 自动执行数据库迁移
    println!("\n🚀 自动检查并执行数据库迁移...");
    
    let migrator = SimpleMigrator::new(&database_url, &service_name, &migrations_path).await?;
    
    // 检查迁移状态
    let status = migrator.get_migration_status().await?;
    println!("📊 当前迁移状态:");
    println!("  服务名称: {}", status.service_name);
    println!("  迁移表: {}", status.migrations_table);
    println!("  总迁移数: {}", status.total_migrations);
    
    // 执行待处理的迁移
    let migration_result = migrator.migrate().await?;
    
    if migration_result.is_success() {
        if migration_result.successful.is_empty() {
            println!("✅ 没有待处理的迁移，数据库已是最新状态");
        } else {
            println!("✅ 所有迁移执行成功！");
            println!("📊 成功执行 {} 个迁移", migration_result.successful.len());
            for migration in &migration_result.successful {
                println!("  - {}: {}", migration.version, migration.name);
            }
        }
        if !migration_result.total_time.is_zero() {
            println!("⏱️  总耗时: {:?}", migration_result.total_time);
        }
    } else {
        println!("❌ 部分迁移执行失败:");
        for failed in &migration_result.failed {
            println!("  - {}: {}", failed.version, failed.error);
        }
        // 迁移失败时可以选择继续运行或退出
        println!("⚠️  迁移失败，但程序将继续运行...");
    }

    // 查询表结构
    println!("\n🔍 查询表结构...");
    
    // 查询 data_api_audit_log 表结构
    println!("\n📋 data_api_audit_log 表结构:");
    match client.query("DESCRIBE data_api_audit_log").execute().await {
        Ok(_) => println!("表结构查询成功"),
        Err(e) => println!("表结构查询失败: {}", e),
    }

    // 查询 test_ttl_where 表结构
    println!("\n📋 test_ttl_where 表结构:");
    match client.query("DESCRIBE test_ttl_where").execute().await {
        Ok(_) => println!("表结构查询成功"),
        Err(e) => println!("表结构查询失败: {}", e),
    }

    // 查询新创建的表
    println!("\n📋 users 表结构:");
    match client.query("DESCRIBE users").execute().await {
        Ok(_) => println!("表结构查询成功"),
        Err(e) => println!("表结构查询失败: {}", e),
    }

    // 查询插入的数据
    println!("\n🔍 查询数据...");
    
    // 查询 data_api_audit_log 表数据
    println!("\n📊 data_api_audit_log 表数据 (前3条):");
    match client.query("SELECT id, timestamp, client_ip, request_method, request_uri, response_status, latency FROM data_api_audit_log ORDER BY timestamp DESC LIMIT 3").execute().await {
        Ok(_) => println!("数据查询成功"),
        Err(e) => println!("数据查询失败: {}", e),
    }

    // 查询 test_ttl_where 表数据
    println!("\n📊 test_ttl_where 表数据 (前3条):");
    match client.query("SELECT * FROM test_ttl_where ORDER BY timestamp DESC LIMIT 3").execute().await {
        Ok(_) => println!("数据查询成功"),
        Err(e) => println!("数据查询失败: {}", e),
    }

    // 查询新创建的表数据
    println!("\n📊 users 表数据:");
    match client.query("SELECT * FROM users LIMIT 5").execute().await {
        Ok(_) => println!("数据查询成功"),
        Err(e) => println!("数据查询失败: {}", e),
    }

    println!("\n🎉 ClickHouse数据库操作完成！");
    println!("\n💡 提示: 程序启动时会自动检查并执行迁移，无需手动操作");
    Ok(())
}

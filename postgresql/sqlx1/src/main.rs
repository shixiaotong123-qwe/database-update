mod database;
mod tables;
mod data;

use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始数据库自动化管理程序");
    
    // 加载环境变量
    dotenv::dotenv().ok();
    
    // 连接数据库并创建管理器
    let db_manager = match database::connect().await {
        Ok(manager) => {
            info!("✅ 数据库连接成功");
            manager
        }
        Err(e) => {
            error!("❌ 数据库连接失败: {}", e);
            return Err(e);
        }
    };
    
    // 执行自动迁移
    info!("📋 开始执行数据库迁移...");
    match db_manager.safe_migrate().await {
        Ok(_) => {
            info!("✅ 数据库迁移完成");
        }
        Err(e) => {
            error!("❌ 数据库迁移失败: {}", e);
            return Err(e);
        }
    }
    
    // 验证表结构
    info!("🔍 验证表结构...");
    match tables::validate_tables(&db_manager.pool).await {
        Ok(_) => {
            info!("✅ 表结构验证通过");
        }
        Err(e) => {
            error!("❌ 表结构验证失败: {}", e);
            return Err(e);
        }
    }
    
    // 检查表统计信息
    match tables::get_table_stats(&db_manager.pool).await {
        Ok(_) => {
            info!("✅ 表统计信息获取完成");
        }
        Err(e) => {
            error!("⚠️  表统计信息获取失败: {}", e);
            // 这不是致命错误，继续执行
        }
    }
    
    // 检查当前数据量，决定是否插入示例数据
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&db_manager.pool)
        .await
        .unwrap_or(0);
    
    if user_count == 0 {
        info!("📝 数据库为空，插入示例数据...");
        match data::insert_sample_data(&db_manager.pool).await {
            Ok(_) => {
                info!("✅ 示例数据插入完成");
            }
            Err(e) => {
                error!("❌ 数据插入失败: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("📊 数据库已包含数据 ({} 个用户)，跳过示例数据插入", user_count);
    }
    
    // 显示数据统计
    info!("📊 显示数据统计信息...");
    match data::show_data_statistics(&db_manager.pool).await {
        Ok(_) => {
            info!("✅ 数据统计显示完成");
        }
        Err(e) => {
            error!("⚠️  数据统计失败: {}", e);
            // 这不是致命错误，继续执行
        }
    }
    
    // 显示高级统计信息
    match data::show_advanced_statistics(&db_manager.pool).await {
        Ok(_) => {
            info!("✅ 高级统计信息显示完成");
        }
        Err(e) => {
            error!("⚠️  高级统计信息获取失败: {}", e);
            // 这不是致命错误，继续执行
        }
    }
    
    // 验证外键关系
    match tables::validate_foreign_keys(&db_manager.pool).await {
        Ok(_) => {
            info!("✅ 外键关系验证通过");
        }
        Err(e) => {
            error!("⚠️  外键关系验证失败: {}", e);
            // 这不是致命错误，继续执行
        }
    }
    
    // 检查索引状态
    match tables::check_indexes(&db_manager.pool).await {
        Ok(_) => {
            info!("✅ 索引状态检查完成");
        }
        Err(e) => {
            error!("⚠️  索引状态检查失败: {}", e);
            // 这不是致命错误，继续执行
        }
    }
    
    info!("🎉 程序执行完成");
    info!("📈 数据库自动化管理成功完成!");
    
    // 优雅关闭数据库连接
    db_manager.close().await;
    
    Ok(())
}
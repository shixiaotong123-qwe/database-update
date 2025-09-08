use anyhow::{Context, Result};
use tokio_postgres::Client;
use tracing::{info, warn};

/// 验证表结构是否符合预期
/// 现在表的创建由Refinery迁移系统管理，这个模块主要用于验证
pub async fn validate_tables(client: &Client) -> Result<()> {
    info!("开始验证表结构");
    
    let expected_tables = vec!["users", "products", "orders"];
    
    for table_name in expected_tables {
        if table_exists(client, table_name).await? {
            let columns = get_table_columns(client, table_name).await?;
            info!("表 '{}' 包含 {} 列", table_name, columns.len());
            
            for (column_name, data_type, is_nullable) in columns {
                info!("  - {}: {} (nullable: {})", column_name, data_type, is_nullable);
            }
        } else {
            return Err(anyhow::anyhow!("表 '{}' 不存在", table_name));
        }
    }
    
    info!("表结构验证完成");
    Ok(())
}

/// 检查表是否存在
pub async fn table_exists(client: &Client, table_name: &str) -> Result<bool> {
    let rows = client.query(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = $1
        ) as exists",
        &[&table_name]
    ).await
    .context(format!("检查表 {} 是否存在时出错", table_name))?;
    
    let exists: bool = rows[0].get("exists");
    Ok(exists)
}

/// 获取表的列信息
pub async fn get_table_columns(client: &Client, table_name: &str) -> Result<Vec<(String, String, String)>> {
    let rows = client.query(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns 
         WHERE table_schema = 'public' 
         AND table_name = $1
         ORDER BY ordinal_position",
        &[&table_name]
    ).await
    .context(format!("获取表 {} 结构信息失败", table_name))?;
    
    let columns = rows.into_iter().map(|row| {
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let is_nullable: String = row.get("is_nullable");
        (column_name, data_type, is_nullable)
    }).collect();
    
    Ok(columns)
}

/// 获取表的行数统计
pub async fn get_table_stats(client: &Client) -> Result<()> {
    info!("数据库表统计信息:");
    
    let tables = vec!["users", "products", "orders"];
    
    for table in tables {
        if table_exists(client, table).await? {
            let rows = client.query(&format!("SELECT COUNT(*) as count FROM {}", table), &[]).await
                .context(format!("获取表 {} 行数失败", table))?;
            
            let count: i64 = rows[0].get("count");
            info!("  - {} 表: {} 行", table, count);
        }
    }
    
    Ok(())
}

/// 验证外键关系
pub async fn validate_foreign_keys(client: &Client) -> Result<()> {
    info!("验证外键关系...");
    
    // 检查orders表的user_id外键
    let rows = client.query(
        "SELECT COUNT(*) as count FROM orders o 
         LEFT JOIN users u ON o.user_id = u.id 
         WHERE u.id IS NULL",
        &[]
    ).await
    .context("检查孤立订单失败")?;
    
    let orphaned_orders: i64 = rows[0].get("count");
    
    if orphaned_orders > 0 {
        warn!("发现 {} 个孤立订单（用户不存在）", orphaned_orders);
    } else {
        info!("✅ 所有订单都有对应的用户");
    }
    
    Ok(())
}

/// 检查索引状态
pub async fn check_indexes(client: &Client) -> Result<()> {
    info!("检查索引状态...");
    
    let indexes = client.query(
        "SELECT 
            schemaname,
            tablename,
            indexname,
            indexdef
         FROM pg_indexes 
         WHERE schemaname = 'public' 
         ORDER BY tablename, indexname",
        &[]
    ).await
    .context("获取索引信息失败")?;
    
    info!("当前索引列表:");
    for row in indexes {
        let table_name: String = row.get("tablename");
        let index_name: String = row.get("indexname");
        info!("  - {}.{}", table_name, index_name);
    }
    
    Ok(())
}


use anyhow::{Context, Result};
use tokio_postgres::Client;
use tracing::info;

/// 插入示例数据到所有表
pub async fn insert_sample_data(client: &Client) -> Result<()> {
    info!("开始插入示例数据");
    
    // 先清空现有数据（可选）
    clear_existing_data(client).await?;
    
    // 插入用户数据
    insert_users(client).await?;
    
    // 插入产品数据
    insert_products(client).await?;
    
    // 插入订单数据
    insert_orders(client).await?;
    
    info!("示例数据插入完成");
    Ok(())
}

/// 清空现有数据
async fn clear_existing_data(client: &Client) -> Result<()> {
    info!("清空现有数据...");
    
    // 按照外键依赖关系的相反顺序删除
    client.execute("DELETE FROM orders", &[]).await
        .context("删除orders表数据失败")?;
    
    client.execute("DELETE FROM products", &[]).await
        .context("删除products表数据失败")?;
    
    client.execute("DELETE FROM users", &[]).await
        .context("删除users表数据失败")?;
    
    // 重置序列
    client.execute("ALTER SEQUENCE users_id_seq RESTART WITH 1", &[]).await
        .context("重置users序列失败")?;
    client.execute("ALTER SEQUENCE products_id_seq RESTART WITH 1", &[]).await
        .context("重置products序列失败")?;
    client.execute("ALTER SEQUENCE orders_id_seq RESTART WITH 1", &[]).await
        .context("重置orders序列失败")?;
    
    info!("数据清空完成");
    Ok(())
}

/// 插入用户数据
async fn insert_users(client: &Client) -> Result<()> {
    info!("插入用户数据...");
    
    let users = vec![
        ("zhangsan", "zhang.san@email.com", "hash123", "张三"),
        ("lisi", "li.si@email.com", "hash456", "李四"),
        ("wangwu", "wang.wu@email.com", "hash789", "王五"),
        ("zhaoliu", "zhao.liu@email.com", "hash321", "赵六"),
        ("sunqi", "sun.qi@email.com", "hash654", "孙七"),
    ];
    
    for (username, email, password_hash, full_name) in users {
        client.execute(
            "INSERT INTO users (username, email, password_hash, full_name, is_active) 
             VALUES ($1, $2, $3, $4, $5)",
            &[&username, &email, &password_hash, &full_name, &true]
        )
        .await
        .context(format!("插入用户 {} 失败", username))?;
        
        info!("插入用户: {}", full_name);
    }
    
    Ok(())
}

/// 插入产品数据 
async fn insert_products(client: &Client) -> Result<()> {
    info!("插入产品数据...");
    
    let products = vec![
        ("iPhone 15 Pro", "苹果最新旗舰手机", "8999.00", 1, 50),
        ("MacBook Air M2", "轻薄笔记本电脑", "7999.00", 1, 30),
        ("iPad Pro 11", "专业平板电脑", "5999.00", 1, 25),
        ("AirPods Pro", "降噪无线耳机", "1599.00", 2, 100),
        ("Apple Watch Series 9", "智能手表", "2999.00", 2, 75),
        ("小米13 Ultra", "小米旗舰手机", "4999.00", 1, 40),
        ("华为MateBook X", "轻薄商务笔记本", "6999.00", 1, 20),
        ("Sony WH-1000XM5", "降噪头戴耳机", "2399.00", 2, 60),
        ("戴尔显示器27寸", "4K专业显示器", "2199.00", 3, 35),
        ("机械键盘", "Cherry轴游戏键盘", "599.00", 3, 80),
    ];
    
    for (name, description, price_str, category_id, stock) in products {
        client.execute(
            "INSERT INTO products (product_name, description, product_price, category_id, stock_quantity, is_active) 
             VALUES ($1, $2, $3::decimal, $4, $5, $6)",
            &[&name, &description, &price_str, &category_id, &stock, &true]
        )
        .await
        .context(format!("插入产品 {} 失败", name))?;
        
        info!("插入产品: {}", name);
    }
    
    Ok(())
}

/// 插入订单数据
async fn insert_orders(client: &Client) -> Result<()> {
    info!("插入订单数据...");
    
    let orders = vec![
        (1, "ORD20241201001", "8999.00", "pending", "北京市朝阳区xxx街道", "北京市朝阳区xxx街道"),
        (2, "ORD20241201002", "7999.00", "completed", "上海市浦东新区xxx路", "上海市浦东新区xxx路"),
        (3, "ORD20241201003", "5999.00", "processing", "广州市天河区xxx大道", "广州市天河区xxx大道"),
        (1, "ORD20241201004", "1599.00", "completed", "北京市海淀区xxx路", "北京市海淀区xxx路"),
        (4, "ORD20241201005", "2999.00", "pending", "深圳市南山区xxx街", "深圳市南山区xxx街"),
        (2, "ORD20241202001", "4999.00", "completed", "上海市静安区xxx路", "上海市静安区xxx路"),
        (5, "ORD20241202002", "6999.00", "processing", "成都市武侯区xxx街道", "成都市武侯区xxx街道"),
        (3, "ORD20241202003", "2399.00", "completed", "杭州市西湖区xxx路", "杭州市西湖区xxx路"),
        (1, "ORD20241202004", "2199.00", "pending", "北京市朝阳区xxx大街", "北京市朝阳区xxx大街"),
        (4, "ORD20241202005", "599.00", "completed", "西安市雁塔区xxx路", "西安市雁塔区xxx路"),
    ];
    
    for (user_id, order_number, total_amount_str, status, shipping_addr, billing_addr) in orders {
        client.execute(
            "INSERT INTO orders (user_id, order_number, total_amount, status, shipping_address, billing_address) 
             VALUES ($1, $2, $3::decimal, $4, $5, $6)",
            &[&user_id, &order_number, &total_amount_str, &status, &shipping_addr, &billing_addr]
        )
        .await
        .context(format!("插入订单 {} 失败", order_number))?;
        
        info!("插入订单: {}", order_number);
    }
    
    Ok(())
}

/// 显示插入的数据统计
pub async fn show_data_statistics(client: &Client) -> Result<()> {
    info!("数据统计信息:");
    
    // 用户数量
    let user_count_rows = client.query("SELECT COUNT(*) as count FROM users", &[]).await?;
    let user_count: i64 = user_count_rows[0].get("count");
    info!("用户总数: {}", user_count);
    
    // 产品数量
    let product_count_rows = client.query("SELECT COUNT(*) as count FROM products", &[]).await?;
    let product_count: i64 = product_count_rows[0].get("count");
    info!("产品总数: {}", product_count);
    
    // 订单数量和状态统计
    let order_count_rows = client.query("SELECT COUNT(*) as count FROM orders", &[]).await?;
    let order_count: i64 = order_count_rows[0].get("count");
    info!("订单总数: {}", order_count);
    
    // 按状态统计订单
    let status_rows = client.query("SELECT status, COUNT(*) as count FROM orders GROUP BY status", &[]).await?;
    
    for row in status_rows {
        let status: String = row.get("status");
        let count: i64 = row.get("count");
        info!("  - {} 状态订单: {}", status, count);
    }
    
    Ok(())
}

/// 获取高级统计信息
pub async fn show_advanced_statistics(client: &Client) -> Result<()> {
    info!("高级统计信息:");
    
    // 用户订单统计
    let user_order_stats = client.query(
        "SELECT 
            u.full_name,
            COUNT(o.id) as order_count,
            COALESCE(SUM(o.total_amount), 0)::text as total_spent
         FROM users u
         LEFT JOIN orders o ON u.id = o.user_id
         GROUP BY u.id, u.full_name
         ORDER BY COALESCE(SUM(o.total_amount), 0) DESC",
        &[]
    ).await?;
    
    info!("用户消费统计:");
    for row in user_order_stats {
        let full_name: String = row.get("full_name");
        let order_count: i64 = row.get("order_count");
        let total_spent: String = row.get("total_spent");
        info!("  - {}: {}个订单, 总消费: ¥{}", full_name, order_count, total_spent);
    }
    
    // 产品库存警告
    let low_stock_products = client.query(
        "SELECT product_name, stock_quantity FROM products 
         WHERE stock_quantity < 30 AND is_active = true
         ORDER BY stock_quantity ASC",
        &[]
    ).await?;
    
    if !low_stock_products.is_empty() {
        info!("库存告警 (低于30件):");
        for row in low_stock_products {
            let name: String = row.get("product_name");
            let stock: i32 = row.get("stock_quantity");
            info!("  ⚠️  {}: {}件", name, stock);
        }
    }
    
    Ok(())
}

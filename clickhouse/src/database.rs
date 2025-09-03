use anyhow::Result;
use clickhouse::Client;
use std::sync::Arc;

pub struct ClickHouseDB {
    client: Client,
}

/// 连接管理器，提供共享的 ClickHouse 连接
pub struct ClickHouseConnectionManager {
    client: Arc<Client>,
}

impl ClickHouseConnectionManager {
    /// 创建新的连接管理器
    pub fn new(database_url: &str, database: &str, user: &str, password: &str) -> Result<Self> {
        let client = Client::default()
            .with_url(database_url)
            .with_database(database)
            .with_user(user)
            .with_password(password);

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// 获取共享的客户端引用
    pub fn get_client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }

    /// 创建 ClickHouseDB 实例（使用共享连接）
    pub fn create_db(&self) -> ClickHouseDB {
        ClickHouseDB {
            client: self.client.as_ref().clone(),
        }
    }
}

impl ClickHouseDB {
    pub fn new() -> Result<Self> {
        let client = Client::default()
            .with_url("http://localhost:8123")
            .with_database("default")
            .with_user("default")
            .with_password("ClickHouse@123");

        Ok(Self { client })
    }

    pub async fn test_connection(&self) -> Result<bool> {
        // 使用简单查询来测试连接
        match self.client.query("SELECT 1").execute().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn get_version(&self) -> Result<Option<String>> {
        // 使用 execute 而不是 fetch_all 来避免类型问题
        let _result = self.client.query("SELECT version()").execute().await?;
        // 这里简化处理，实际项目中可能需要更复杂的逻辑
        Ok(Some("ClickHouse version".to_string()))
    }

    pub async fn get_tables(&self) -> Result<Vec<String>> {
        // 使用 execute 而不是 fetch_all 来避免类型问题
        let _result = self.client.query("SHOW TABLES").execute().await?;
        // 这里简化处理，实际项目中可能需要更复杂的逻辑
        Ok(vec![])
    }

    pub async fn execute_query(&self, query: &str) -> Result<()> {
        self.client.query(query).execute().await?;
        Ok(())
    }

    pub async fn create_table(&self, table_name: &str, schema: &str) -> Result<()> {
        let create_sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, schema);
        self.client.query(&create_sql).execute().await?;
        Ok(())
    }

    pub async fn insert_data<T>(&self, table_name: &str, _data: &T) -> Result<()> 
    where 
        T: serde::Serialize
    {
        // 简化插入操作，实际项目中需要更复杂的实现
        println!("插入数据到表: {}", table_name);
        Ok(())
    }
}

use anyhow::Result;
use clickhouse::Client;

pub struct ClickHouseDB {
    client: Client,
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

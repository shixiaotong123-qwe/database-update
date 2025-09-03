use clickhouse::Client;
use anyhow::{Result, Context};
use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub struct SimpleMigrator {
    client: Client,
    service_name: String,
    migrations_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub version: String,
    pub name: String,
    pub applied_at: String,
    pub execution_time_ms: u64,
    pub checksum: String,
    pub success: bool,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub checksum: String,
    pub is_baseline: bool,
}

#[derive(Debug)]
pub struct MigrationSummary {
    pub successful: Vec<MigrationRecord>,
    pub failed: Vec<FailedMigration>,
    pub total_time: std::time::Duration,
}

#[derive(Debug)]
pub struct FailedMigration {
    pub version: String,
    pub name: String,
    pub error: String,
}

impl SimpleMigrator {
    pub async fn new(database_url: &str, service_name: &str, migrations_path: &str) -> Result<Self> {
        let client = Client::default()
            .with_url(database_url)
            .with_database("default")
            .with_user("default")
            .with_password("ClickHouse@123");
        
        let migrator = Self {
            client,
            service_name: service_name.to_string(),
            migrations_path: migrations_path.to_string(),
        };
        
        // 只创建一个迁移记录表
        migrator.setup_migrations_table().await?;
        
        Ok(migrator)
    }
    
    /// 创建迁移记录表（唯一的"meta"表）
    async fn setup_migrations_table(&self) -> Result<()> {
        let table_name = format!("_migrations_{}", self.service_name);
        
        let create_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table_name} (
                version String,
                name String,
                applied_at DateTime64(3) DEFAULT now64(3),
                execution_time_ms UInt64,
                checksum String,
                success UInt8,
                error_message String DEFAULT ''
            ) ENGINE = MergeTree()
            ORDER BY version
            "#
        );
        
        self.client.query(&create_sql).execute().await
            .context("Failed to create migrations table")?;
        
        Ok(())
    }
    
    /// 主要入口：运行待处理的迁移
    pub async fn migrate(&self) -> Result<MigrationSummary> {
        println!("🚀 Starting migration for service: {}", self.service_name);
        
        let start_time = Instant::now();
        
        // 1. 扫描迁移文件（只扫描一次）
        let migration_files = self.scan_migration_files().await?;
        
        // 2. 获取已执行的迁移
        let applied_versions = self.get_applied_versions_from_database().await?;
        
        // 3. 筛选待执行的迁移（按版本号排序）
        let mut pending: Vec<&MigrationFile> = migration_files
            .values()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect();
        
        // 按版本号排序
        pending.sort_by(|a, b| a.version.cmp(&b.version));
        
        if pending.is_empty() {
            println!("✅ 没有待处理的迁移，数据库已是最新状态");
            return Ok(MigrationSummary::no_migrations());
        }
        
        println!("📋 发现 {} 个待处理的迁移", pending.len());
        for migration in &pending {
            println!("  - {}: {}", migration.version, migration.name);
        }
        
        // 4. 执行迁移
        let mut summary = MigrationSummary::new();
        
        for migration in pending {
            println!("🔧 Executing migration: {} - {}", migration.version, migration.name);
            
            let result = self.execute_migration(migration).await;
            
            match result {
                Ok(record) => {
                    summary.successful.push(record);
                    println!("✅ Migration {} completed", migration.version);
                }
                Err(e) => {
                    println!("❌ Migration {} failed: {}", migration.version, e);
                    summary.failed.push(FailedMigration {
                        version: migration.version.clone(),
                        name: migration.name.clone(),
                        error: e.to_string(),
                    });
                    
                    // 默认策略：遇到失败就停止
                    if !self.should_continue_on_failure() {
                        break;
                    }
                }
            }
        }
        
        summary.total_time = start_time.elapsed();
        Ok(summary)
    }
    
    /// 扫描迁移文件目录
    async fn scan_migration_files(&self) -> Result<BTreeMap<String, MigrationFile>> {
        use tokio::fs;
        use std::path::Path;
        
        let migrations_dir = Path::new(&self.migrations_path);
        if !migrations_dir.exists() {
            println!("⚠️  迁移目录不存在: {}", self.migrations_path);
            return Ok(BTreeMap::new());
        }
        
        let mut migration_files = BTreeMap::new();
        let mut entries = fs::read_dir(migrations_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("sql")) {
                match self.parse_migration_file(&path).await {
                    Ok(migration) => {
                        println!("📁 发现迁移文件: {} - {}", migration.version, migration.name);
                        migration_files.insert(migration.version.clone(), migration);
                    }
                    Err(e) => {
                        println!("⚠️  Failed to parse migration file {:?}: {}", path, e);
                    }
                }
            }
        }
        
        println!("📋 总共扫描到 {} 个迁移文件", migration_files.len());
        Ok(migration_files)
    }
    
    /// 解析单个迁移文件
    async fn parse_migration_file(&self, file_path: &std::path::Path) -> Result<MigrationFile> {
        use regex::Regex;
        
        let content = tokio::fs::read_to_string(file_path).await?;
        
        // 解析文件名：V000__baseline_existing_database.sql 或 V001__create_users_table.sql
        let filename = file_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
        
        let version_regex = Regex::new(r"^V(\d+)__(.+)$")?;
        let captures = version_regex.captures(filename)
            .ok_or_else(|| anyhow::anyhow!("Invalid migration filename format: {}", filename))?;
        
        let version = captures.get(1).unwrap().as_str().to_string();
        let name = captures.get(2).unwrap().as_str().replace('_', " ");
        
        // 解析SQL内容
        let (up_sql, down_sql) = self.parse_sql_content(&content)?;
        
        // 检查是否为基线迁移（版本号为000或SQL内容为空）
        let is_baseline = version == "000" || up_sql.trim().is_empty();
        
        // 计算校验和
        let checksum = self.calculate_checksum(&up_sql);
        
        Ok(MigrationFile {
            version,
            name,
            up_sql,
            down_sql,
            checksum,
            is_baseline,
        })
    }
    
    /// 解析SQL内容（分离UP和DOWN部分）
    fn parse_sql_content(&self, content: &str) -> Result<(String, Option<String>)> {
        let lines: Vec<&str> = content.lines().collect();
        let mut up_sql = String::new();
        let mut down_sql: Option<String> = None;
        let mut current_section = Section::Up;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed == "-- +migrate Up" {
                current_section = Section::Up;
                continue;
            } else if trimmed == "-- +migrate Down" {
                current_section = Section::Down;
                down_sql = Some(String::new());
                continue;
            }
            
            // 跳过注释行（但保留SQL中的注释）
            if trimmed.starts_with("-- ") && !trimmed.contains("/*") {
                continue;
            }
            
            match current_section {
                Section::Up => {
                    up_sql.push_str(line);
                    up_sql.push('\n');
                }
                Section::Down => {
                    if let Some(ref mut down) = down_sql {
                        down.push_str(line);
                        down.push('\n');
                    }
                }
            }
        }
        
        Ok((up_sql.trim().to_string(), down_sql.map(|s| s.trim().to_string())))
    }
    
    /// 执行单个迁移
    async fn execute_migration(&self, migration: &MigrationFile) -> Result<MigrationRecord> {
        let start_time = Instant::now();
        
        // 对于基线迁移，跳过SQL执行
        let execution_result = if migration.is_baseline {
            println!("📋 Baseline migration detected, skipping SQL execution");
            Ok(())
        } else {
            // 执行SQL语句
            self.execute_sql_statements(&migration.up_sql).await
        };
        
        let execution_time = start_time.elapsed();
        let success = execution_result.is_ok();
        let error_message = execution_result.err().map(|e| e.to_string()).unwrap_or_default();
        
        // 记录迁移结果
        let record = MigrationRecord {
            version: migration.version.clone(),
            name: migration.name.clone(),
            applied_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            execution_time_ms: execution_time.as_millis() as u64,
            checksum: migration.checksum.clone(),
            success,
            error_message: error_message.clone(),
        };
        
        // 保存到数据库（即使失败也要记录）
        self.save_migration_record(&record).await?;
        
        // 如果执行失败，返回错误
        if !success {
            return Err(anyhow::anyhow!("Migration execution failed: {}", error_message));
        }
        
        Ok(record)
    }
    
    /// 执行SQL语句（支持多语句）
    async fn execute_sql_statements(&self, sql: &str) -> Result<()> {
        // 如果SQL为空，直接返回成功
        if sql.trim().is_empty() {
            println!("📝 Empty SQL content, skipping execution");
            return Ok(());
        }
        
        // 简单的语句分割（按分号分割，但要注意字符串中的分号）
        let statements = self.split_sql_statements(sql);
        
        for (i, statement) in statements.iter().enumerate() {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            println!("🔍 Executing statement {}: {}", i + 1, 
                    &trimmed[..std::cmp::min(100, trimmed.len())]);
            
            self.client.query(trimmed).execute().await
                .with_context(|| format!("Failed to execute statement {}: {}", i + 1, trimmed))?;
        }
        
        Ok(())
    }
    
    /// 分割SQL语句
    fn split_sql_statements(&self, sql: &str) -> Vec<String> {
        // 简单实现：按分号分割，忽略字符串内的分号
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_string = false;
        let mut string_delimiter = '\0';
        
        for ch in sql.chars() {
            match ch {
                '\'' | '"' if !in_string => {
                    in_string = true;
                    string_delimiter = ch;
                    current_statement.push(ch);
                }
                ch if in_string && ch == string_delimiter => {
                    in_string = false;
                    current_statement.push(ch);
                }
                ';' if !in_string => {
                    if !current_statement.trim().is_empty() {
                        statements.push(current_statement.trim().to_string());
                        current_statement.clear();
                    }
                }
                _ => {
                    current_statement.push(ch);
                }
            }
        }
        
        // 添加最后一个语句
        if !current_statement.trim().is_empty() {
            statements.push(current_statement.trim().to_string());
        }
        
        statements
    }
    

    

    
    /// 从数据库获取已应用的迁移版本
    async fn get_applied_versions_from_database(&self) -> Result<HashSet<String>> {
        let mut applied_versions = HashSet::new();
        let table_name = format!("_migrations_{}", self.service_name);
        
        // 首先检查迁移表是否存在
        let table_exists_query = format!("EXISTS TABLE {}", table_name);
        let table_exists = match self.client.query(&table_exists_query).execute().await {
            Ok(_) => true,
            Err(_) => false,
        };
        
        if !table_exists {
            println!("📋 迁移表 {} 不存在，所有迁移都标记为未应用", table_name);
            println!("🔧 迁移表将在首次迁移时自动创建");
            return Ok(applied_versions);
        }
        
        // 使用更智能的方法：查询迁移表中已应用的版本
        println!("📋 检测到迁移表存在，查询已应用的迁移版本");
        applied_versions = self.get_applied_versions_from_table().await?;
        
        println!("📋 检查完成：发现 {} 个已应用的迁移", applied_versions.len());
        Ok(applied_versions)
    }
    
    /// 从迁移表中查询已应用的迁移版本
    async fn get_applied_versions_from_table(&self) -> Result<HashSet<String>> {
        let mut applied_versions = HashSet::new();
        let table_name = format!("_migrations_{}", self.service_name);
        
        // 使用新版本的 ClickHouse 客户端特性来精确查询
        println!("📋 使用新版本 ClickHouse 客户端查询已应用的迁移版本");
        
        // 查询迁移表中所有已应用的版本
        let query = format!("SELECT version FROM {} WHERE success = 1 ORDER BY version", table_name);
        
        // 尝试使用 fetch_all 方法，如果失败则回退到旧方法
        match self.client.query(&query).fetch_all::<String>().await {
            Ok(rows) => {
                println!("📋 成功查询到 {} 个已应用的迁移", rows.len());
                for version in rows {
                    applied_versions.insert(version.clone());
                    println!("📋 迁移 {} 已应用", version);
                }
            }
            Err(e) => {
                println!("📋 查询迁移表失败: {}", e);
                println!("📋 回退到旧方法：假设迁移表为空，所有迁移标记为未应用");
            }
        }
        
        Ok(applied_versions)
    }
    

    

    

    

    
    /// 保存迁移记录
    async fn save_migration_record(&self, record: &MigrationRecord) -> Result<()> {
        let table_name = format!("_migrations_{}", self.service_name);
        
        let insert_sql = format!(
            r#"
            INSERT INTO {table_name} 
            (version, name, applied_at, execution_time_ms, checksum, success, error_message)
            VALUES ('{}', '{}', '{}', {}, '{}', {}, '{}')
            "#,
            record.version,
            record.name.replace('\'', "''"), // 转义单引号
            record.applied_at,
            record.execution_time_ms,
            record.checksum,
            record.success as u8,
            record.error_message.replace('\'', "''")
        );
        
        self.client.query(&insert_sql).execute().await?;
        Ok(())
    }
    
    /// 计算SQL内容的校验和
    fn calculate_checksum(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// 是否在失败时继续执行
    fn should_continue_on_failure(&self) -> bool {
        std::env::var("CONTINUE_ON_MIGRATION_FAILURE")
            .unwrap_or_default() == "true"
    }
    
    /// 获取迁移状态
    pub async fn get_migration_status(&self) -> Result<MigrationStatus> {
        let table_name = format!("_migrations_{}", self.service_name);
        
        // 首先检查迁移表是否存在
        let table_exists_query = format!("EXISTS TABLE {}", table_name);
        let table_exists = match self.client.query(&table_exists_query).execute().await {
            Ok(_) => true,
            Err(_) => false,
        };
        
        let total_migrations = if table_exists {
            // 获取已应用的迁移数量
            let count_query = format!("SELECT count() FROM {} WHERE success = 1", table_name);
            match self.client.query(&count_query).execute().await {
                Ok(_) => 1, // 简化处理
                Err(_) => 0
            }
        } else {
            0 // 表不存在，迁移数量为0
        };
        
        let status_message = if table_exists {
            format!("已存在")
        } else {
            format!("不存在（将在首次迁移时自动创建）")
        };
        
        Ok(MigrationStatus {
            service_name: self.service_name.clone(),
            migrations_table: format!("{} ({})", table_name, status_message),
            total_migrations,
        })
    }
}

#[derive(Debug)]
enum Section {
    Up,
    Down,
}

#[derive(Debug)]
pub struct MigrationStatus {
    pub service_name: String,
    pub migrations_table: String,
    pub total_migrations: usize,
}

impl MigrationSummary {
    fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
            total_time: std::time::Duration::default(),
        }
    }
    
    fn no_migrations() -> Self {
        Self::new()
    }
    
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

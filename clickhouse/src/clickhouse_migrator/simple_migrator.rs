use anyhow::{Result, Context, anyhow};
use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::task::JoinSet;
use tracing::{info, warn, error, debug};
use sha2::{Sha256, Digest};
use crate::database::ClickHouseConnectionManager;

pub struct SimpleMigrator {
    connection_manager: ClickHouseConnectionManager,
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

#[derive(Debug)]
pub struct MigrationStatus {
    pub service_name: String,
    pub migrations_table: String,
    pub total_migrations: usize,
    pub table_exists: bool,
    pub last_migration: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MigrationVersion {
    number: u32,
    original: String,
}

impl MigrationVersion {
    fn parse(version_str: &str) -> Result<Self> {
        let number = version_str.parse::<u32>()
            .with_context(|| format!("Invalid version number: {}", version_str))?;
        
        Ok(MigrationVersion {
            number,
            original: version_str.to_string(),
        })
    }
}

impl MigrationFile {
    pub fn version(&self) -> Result<MigrationVersion> {
        MigrationVersion::parse(&self.version)
    }
}

impl SimpleMigrator {
    pub async fn new(database_url: &str, service_name: &str, migrations_path: &str) -> Result<Self> {
        let connection_manager = ClickHouseConnectionManager::new(
            database_url, 
            "default", 
            "default", 
            "ClickHouse@123"
        )?;
        
        let migrator = Self {
            connection_manager,
            service_name: service_name.to_string(),
            migrations_path: migrations_path.to_string(),
        };
        
        // 创建迁移记录表
        migrator.setup_migrations_table().await?;
        
        Ok(migrator)
    }
    
    /// 获取迁移表名
    fn get_migration_table_name(&self) -> String {
        format!("_migrations_{}", self.service_name)
    }
    
    /// 执行查询并返回单个值（简化版本）
    async fn query_single<T>(&self, query: &str) -> Result<T> 
    where 
        T: serde::de::DeserializeOwned + Send + Unpin + 'static,
    {
        // 简化实现，直接返回错误
        // 在实际使用中，这里需要根据 ClickHouse 客户端的具体 API 来实现
        Err(anyhow::anyhow!("Query result handling not implemented for type T"))
    }
    
    /// 执行查询并返回多个值（简化版本）
    async fn query_all<T>(&self, query: &str) -> Result<Vec<T>>
    where 
        T: serde::de::DeserializeOwned + Send + Unpin + 'static,
    {
        // 简化实现，直接返回错误
        // 在实际使用中，这里需要根据 ClickHouse 客户端的具体 API 来实现
        Err(anyhow::anyhow!("Query result handling not implemented for type T"))
    }
    /// 执行DDL语句
    async fn execute_ddl(&self, query: &str) -> Result<()> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() {
            return Ok(());
        }
        
        debug!("Executing DDL: {}", trimmed_query);
        
        match self.connection_manager.get_client()
            .query(trimmed_query)
            .execute()
            .await
        {
            Ok(_) => {
                debug!("DDL executed successfully: {}", &trimmed_query[..std::cmp::min(100, trimmed_query.len())]);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to execute DDL: {}\nSQL: {}\nError: {}", 
                    e, trimmed_query, e);
                error!("{}", error_msg);
                Err(anyhow!(error_msg))
            }
        }
    }
    
    /// 检查表是否存在
    async fn table_exists(&self, table_name: &str) -> Result<bool> {
        let query = format!(
            "SELECT count() FROM system.tables WHERE database = currentDatabase() AND name = '{}'",
            table_name
        );
        
        let count: u64 = self.query_single(&query).await.unwrap_or(0);
        Ok(count > 0)
    }
    
    /// 创建迁移记录表
    async fn setup_migrations_table(&self) -> Result<()> {
        let table_name = self.get_migration_table_name();
        
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
            SETTINGS index_granularity = 8192
            "#
        );
        
        self.execute_ddl(&create_sql).await
            .context("Failed to create migrations table")?;
        
        debug!("Migration table {} ensured", table_name);
        Ok(())
    }
    
    /// 主要入口：运行待处理的迁移
    pub async fn migrate(&self) -> Result<MigrationSummary> {
        let _span = tracing::info_span!("migrate", service = %self.service_name).entered();
        info!("Starting migration");
        
        let start_time = Instant::now();
        
        // 1. 扫描迁移文件
        let migration_files = self.scan_migration_files().await
            .context("Failed to scan migration files")?;
        
        if migration_files.is_empty() {
            warn!("No migration files found in {}", self.migrations_path);
            return Ok(MigrationSummary::no_migrations());
        }
        
        // 2. 验证现有迁移的校验和
        self.validate_applied_migrations(&migration_files).await
            .context("Migration validation failed")?;
        
        // 3. 获取已执行的迁移
        let applied_versions = self.get_applied_versions().await
            .context("Failed to get applied versions")?;
        
        // 4. 确定待执行的迁移
        let pending = self.get_pending_migrations(&migration_files, &applied_versions)?;
        
        if pending.is_empty() {
            info!("No pending migrations found");
            return Ok(MigrationSummary::no_migrations());
        }
        
        info!("Found {} pending migrations", pending.len());
        self.log_pending_migrations(&pending);
        
        // 5. 执行迁移
        let mut summary = self.execute_pending_migrations(pending).await?;
        summary.total_time = start_time.elapsed();
        
        info!(
            successful = summary.successful.len(),
            failed = summary.failed.len(),
            duration_ms = summary.total_time.as_millis(),
            "Migration completed"
        );
        
        Ok(summary)
    }
    
    /// 扫描迁移文件目录（并发处理）
    async fn scan_migration_files(&self) -> Result<BTreeMap<String, MigrationFile>> {
        use tokio::fs;
        use std::path::Path;
        
        let migrations_dir = Path::new(&self.migrations_path);
        if !migrations_dir.exists() {
            warn!("Migration directory does not exist: {}", self.migrations_path);
            return Ok(BTreeMap::new());
        }
        
        let mut entries = fs::read_dir(migrations_dir).await
            .with_context(|| format!("Failed to read migrations directory: {}", self.migrations_path))?;
        
        let mut join_set = JoinSet::new();
        
        // 并发读取所有SQL文件
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("sql")) {
                let path_clone = path.clone();
                join_set.spawn(async move {
                    let content = tokio::fs::read_to_string(&path_clone).await?;
                    Ok::<_, anyhow::Error>((path_clone, content))
                });
            }
        }
        
        let mut migration_files = BTreeMap::new();
        
        // 收集所有文件内容并解析
        while let Some(result) = join_set.join_next().await {
            match result? {
                Ok((path, content)) => {
                    match self.parse_migration_content(&path, &content) {
                        Ok(migration) => {
                            debug!("Parsed migration: {} - {}", migration.version, migration.name);
                            migration_files.insert(migration.version.clone(), migration);
                        }
                        Err(e) => {
                            warn!("Failed to parse migration file {:?}: {}", path, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read migration file: {}", e);
                }
            }
        }
        
        info!("Scanned {} migration files", migration_files.len());
        Ok(migration_files)
    }
    
    /// 解析单个迁移文件内容
    fn parse_migration_content(&self, file_path: &std::path::Path, content: &str) -> Result<MigrationFile> {
        use regex::Regex;
        
        // 解析文件名：V001__create_users_table.sql
        let filename = file_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid filename: {:?}", file_path))?;
        
        let version_regex = Regex::new(r"^V(\d+)__(.+)$")
            .context("Failed to compile version regex")?;
        
        let captures = version_regex.captures(filename)
            .ok_or_else(|| anyhow!("Invalid migration filename format: {} (expected: V001__description.sql)", filename))?;
        
        let version = captures.get(1).unwrap().as_str().to_string();
        let name = captures.get(2).unwrap().as_str().replace('_', " ");
        
        // 解析SQL内容
        let (up_sql, down_sql) = self.parse_sql_content(content)?;
        
        // 检查是否为基线迁移
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
            
            // 跳过元数据注释
            if trimmed.starts_with("-- ") && !trimmed.contains("/*") && !trimmed.contains("--") {
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
    
    /// 验证已应用迁移的校验和
    async fn validate_applied_migrations(&self, migration_files: &BTreeMap<String, MigrationFile>) -> Result<()> {
        let table_name = self.get_migration_table_name();
        
        if !self.table_exists(&table_name).await? {
            return Ok(());
        }
        
        let query = format!("SELECT version, checksum FROM {} WHERE success = 1", table_name);
        let applied_records: Vec<(String, String)> = self.query_all(&query).await?;
        
        let mut validation_errors = Vec::new();
        
        for (version, stored_checksum) in applied_records {
            if let Some(migration_file) = migration_files.get(&version) {
                if migration_file.checksum != stored_checksum {
                    validation_errors.push(format!(
                        "Migration {}: checksum mismatch (expected: {}, found: {})",
                        version, stored_checksum, migration_file.checksum
                    ));
                }
            } else {
                warn!("Applied migration {} not found in migration files", version);
            }
        }
        
        if !validation_errors.is_empty() {
            return Err(anyhow!(
                "Migration validation failed:\n{}",
                validation_errors.join("\n")
            ));
        }
        
        Ok(())
    }
    
    /// 获取已应用的迁移版本
    async fn get_applied_versions(&self) -> Result<HashSet<String>> {
        let table_name = self.get_migration_table_name();
        
        if !self.table_exists(&table_name).await? {
            info!("Migration table {} does not exist, treating all migrations as pending", table_name);
            return Ok(HashSet::new());
        }
        
        let query = format!("SELECT version FROM {} WHERE success = 1 ORDER BY version", table_name);
        
        let versions: Vec<String> = self.query_all(&query).await
            .unwrap_or_else(|e| {
                warn!("Failed to query applied versions: {}", e);
                Vec::new()
            });
        
        info!("Found {} applied migrations", versions.len());
        Ok(versions.into_iter().collect())
    }
    
    /// 确定待执行的迁移
    fn get_pending_migrations(
        &self, 
        migration_files: &BTreeMap<String, MigrationFile>,
        applied_versions: &HashSet<String>
    ) -> Result<Vec<MigrationFile>> {
        let mut pending: Vec<MigrationFile> = migration_files
            .values()
            .filter(|m| !applied_versions.contains(&m.version))
            .cloned()
            .collect();
        
        // 使用数字版本排序
        pending.sort_by(|a, b| {
            let a_version = a.version().unwrap_or(MigrationVersion { 
                number: 0, 
                original: a.version.clone() 
            });
            let b_version = b.version().unwrap_or(MigrationVersion { 
                number: 0, 
                original: b.version.clone() 
            });
            a_version.cmp(&b_version)
        });
        
        // 验证版本序列的连续性
        self.validate_migration_sequence(&pending)?;
        
        Ok(pending)
    }
    
    /// 验证迁移序列
    fn validate_migration_sequence(&self, pending: &[MigrationFile]) -> Result<()> {
        let mut versions: Vec<u32> = pending.iter()
            .filter_map(|m| m.version().ok())
            .map(|v| v.number)
            .collect();
        
        versions.sort();
        
        // 检查重复版本
        let mut seen = HashSet::new();
        for version in &versions {
            if !seen.insert(*version) {
                return Err(anyhow!("Duplicate migration version: {}", version));
            }
        }
        
        Ok(())
    }
    
    /// 记录待处理的迁移
    fn log_pending_migrations(&self, pending: &[MigrationFile]) {
        for migration in pending {
            info!(
                version = %migration.version,
                name = %migration.name,
                is_baseline = migration.is_baseline,
                checksum = %migration.checksum[..8],
                "Pending migration"
            );
        }
    }
    
    /// 执行待处理的迁移
    async fn execute_pending_migrations(&self, pending: Vec<MigrationFile>) -> Result<MigrationSummary> {
        let mut summary = MigrationSummary::new();
        let total = pending.len();
        
        for (index, migration) in pending.iter().enumerate() {
            let _span = tracing::info_span!("execute_migration", 
                version = %migration.version, 
                progress = format!("{}/{}", index + 1, total)
            ).entered();
            
            info!("Executing migration: {}", migration.name);
            
            match self.execute_migration(migration).await {
                Ok(record) => {
                    summary.successful.push(record);
                    info!("Migration completed successfully");
                }
                Err(e) => {
                    let failed_migration = FailedMigration {
                        version: migration.version.clone(),
                        name: migration.name.clone(),
                        error: e.to_string(),
                    };
                    
                    error!("Migration failed: {}", e);
                    summary.failed.push(failed_migration);
                    
                    if !self.should_continue_on_failure() {
                        error!("Stopping migration execution due to failure");
                        break;
                    }
                }
            }
        }
        
        Ok(summary)
    }
    
    /// 执行单个迁移
    async fn execute_migration(&self, migration: &MigrationFile) -> Result<MigrationRecord> {
        let start_time = Instant::now();
        
        info!("Starting migration: {} - {}", migration.version, migration.name);
        debug!("Migration checksum: {}", migration.checksum);
        
        // 对于基线迁移，跳过SQL执行
        let execution_result = if migration.is_baseline {
            info!("Baseline migration detected, skipping SQL execution");
            Ok(())
        } else {
            info!("Executing migration SQL with {} characters", migration.up_sql.len());
            debug!("Migration SQL preview: {}", &migration.up_sql[..std::cmp::min(200, migration.up_sql.len())]);
            
            self.execute_sql_statements(&migration.up_sql).await
                .with_context(|| format!("Failed to execute migration {}: {}", migration.version, migration.name))
        };
        
        let execution_time = start_time.elapsed();
        let success = execution_result.is_ok();
        let error_message = execution_result.err().map(|e| e.to_string()).unwrap_or_default();
        
        if success {
            info!("Migration {} completed successfully in {:?}", migration.version, execution_time);
        } else {
            error!("Migration {} failed after {:?}: {}", migration.version, execution_time, error_message);
        }
        
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
        
        // 保存到数据库（无论成功失败都记录）
        match self.save_migration_record(&record).await {
            Ok(_) => debug!("Migration record saved to database"),
            Err(e) => warn!("Failed to save migration record: {}", e),
        }
        
        // 如果执行失败，返回错误
        if !success {
            return Err(anyhow!("Migration execution failed: {}", error_message));
        }
        
        Ok(record)
    }
    
    /// 执行SQL语句（支持多语句）
    async fn execute_sql_statements(&self, sql: &str) -> Result<()> {
        if sql.trim().is_empty() {
            debug!("Empty SQL content, skipping execution");
            return Ok(());
        }
        
        let statements = self.split_sql_statements(sql);
        info!("Executing {} SQL statements", statements.len());
        
        for (i, statement) in statements.iter().enumerate() {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            let statement_preview = if trimmed.len() > 100 {
                format!("{}...", &trimmed[..100])
            } else {
                trimmed.to_string()
            };
            
            info!("Executing statement {}/{}: {}", 
                   i + 1, statements.len(), statement_preview);
            
            match self.execute_ddl(trimmed).await {
                Ok(_) => {
                    info!("Statement {}/{} executed successfully", i + 1, statements.len());
                }
                Err(e) => {
                    let error_context = format!(
                        "Failed to execute statement {}/{}: {}\nSQL: {}\nFull error: {}", 
                        i + 1, statements.len(), statement_preview, trimmed, e
                    );
                    error!("{}", error_context);
                    return Err(anyhow!(error_context));
                }
            }
        }
        
        info!("All {} SQL statements executed successfully", statements.len());
        Ok(())
    }
    
    /// 改进的SQL语句分割
    fn split_sql_statements(&self, sql: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_comment = false;
        let mut prev_char = '\0';
        
        for ch in sql.chars() {
            match ch {
                '\'' if !in_double_quote && !in_comment => {
                    // 处理转义的单引号
                    if prev_char != '\\' {
                        in_single_quote = !in_single_quote;
                    }
                    current_statement.push(ch);
                }
                '"' if !in_single_quote && !in_comment => {
                    // 处理转义的双引号
                    if prev_char != '\\' {
                        in_double_quote = !in_double_quote;
                    }
                    current_statement.push(ch);
                }
                '-' if prev_char == '-' && !in_single_quote && !in_double_quote => {
                    in_comment = true;
                    current_statement.push(ch);
                }
                '\n' if in_comment => {
                    in_comment = false;
                    current_statement.push(ch);
                }
                ';' if !in_single_quote && !in_double_quote && !in_comment => {
                    let statement = current_statement.trim();
                    if !statement.is_empty() {
                        statements.push(statement.to_string());
                    }
                    current_statement.clear();
                }
                _ => {
                    current_statement.push(ch);
                }
            }
            prev_char = ch;
        }
        
        // 添加最后一个语句
        let final_statement = current_statement.trim();
        if !final_statement.is_empty() {
            statements.push(final_statement.to_string());
        }
        
        statements
    }
    
    /// 保存迁移记录
    async fn save_migration_record(&self, record: &MigrationRecord) -> Result<()> {
        let table_name = self.get_migration_table_name();
        
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
        
        self.execute_ddl(&insert_sql).await
            .context("Failed to insert migration record")?;
        
        Ok(())
    }
    
    /// 计算校验和
    fn calculate_checksum(&self, content: &str) -> String {
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
        let table_name = self.get_migration_table_name();
        let table_exists = self.table_exists(&table_name).await?;
        
        let (total_migrations, last_migration) = if table_exists {
            let count_query = format!("SELECT count() FROM {} WHERE success = 1", table_name);
            let count: u64 = self.query_single(&count_query).await.unwrap_or(0);
            
            let last_query = format!(
                "SELECT version FROM {} WHERE success = 1 ORDER BY version DESC LIMIT 1", 
                table_name
            );
            let last: Option<String> = self.query_single(&last_query).await.ok();
            
            (count as usize, last)
        } else {
            (0, None)
        };
        
        Ok(MigrationStatus {
            service_name: self.service_name.clone(),
            migrations_table: table_name,
            total_migrations,
            table_exists,
            last_migration,
        })
    }
    
    /// 获取失败的迁移详情
    pub async fn get_failed_migrations(&self) -> Result<Vec<MigrationRecord>> {
        let table_name = self.get_migration_table_name();
        
        if !self.table_exists(&table_name).await? {
            return Ok(Vec::new());
        }
        
        let query = format!(
            "SELECT version, name, applied_at, execution_time_ms, checksum, success, error_message 
             FROM {} WHERE success = 0 ORDER BY applied_at DESC",
            table_name
        );
        
        // 注意：这里需要根据实际的 ClickHouse 客户端 API 调整
        // 这是一个示例实现
        let records = self.query_all::<(String, String, String, u64, String, u8, String)>(&query).await?;
        
        let migrations = records.into_iter().map(|(version, name, applied_at, execution_time_ms, checksum, success, error_message)| {
            MigrationRecord {
                version,
                name,
                applied_at,
                execution_time_ms,
                checksum,
                success: success == 1,
                error_message,
            }
        }).collect();
        
        Ok(migrations)
    }
    
    /// 获取迁移执行的详细日志
    pub async fn get_migration_logs(&self, version: &str) -> Result<Vec<String>> {
        let table_name = self.get_migration_table_name();
        
        if !self.table_exists(&table_name).await? {
            return Ok(Vec::new());
        }
        
        let query = format!(
            "SELECT error_message, applied_at, execution_time_ms 
             FROM {} WHERE version = '{}' ORDER BY applied_at DESC",
            table_name, version
        );
        
        // 这里返回一个简化的日志格式
        // 在实际实现中，你可能需要查询系统日志表或其他日志源
        Ok(vec![format!("Migration {} executed at {}", version, chrono::Utc::now())])
    }
    
    /// 获取已应用迁移的详细记录
    pub async fn get_applied_migrations(&self) -> Result<Vec<MigrationRecord>> {
        let table_name = self.get_migration_table_name();
        
        if !self.table_exists(&table_name).await? {
            return Ok(Vec::new());
        }
        
        let query = format!(
            "SELECT version, name, applied_at, execution_time_ms, checksum, success, error_message 
             FROM {} ORDER BY version",
            table_name
        );
        
        // 注意：这里需要根据实际的 ClickHouse 客户端 API 调整
        // 这是一个示例实现
        let records = self.query_all::<(String, String, String, u64, String, u8, String)>(&query).await?;
        
        let migrations = records.into_iter().map(|(version, name, applied_at, execution_time_ms, checksum, success, error_message)| {
            MigrationRecord {
                version,
                name,
                applied_at,
                execution_time_ms,
                checksum,
                success: success == 1,
                error_message,
            }
        }).collect();
        
        Ok(migrations)
    }
    
    /// 回滚最后一个迁移（如果支持）
    pub async fn rollback_last(&self) -> Result<()> {
        // 获取最后一个成功的迁移
        let table_name = self.get_migration_table_name();
        let query = format!(
            "SELECT version FROM {} WHERE success = 1 ORDER BY version DESC LIMIT 1",
            table_name
        );
        
        let last_version: String = self.query_single(&query).await
            .context("No migrations to rollback")?;
        
        // 扫描迁移文件找到对应的回滚SQL
        let migration_files = self.scan_migration_files().await?;
        
        if let Some(migration_file) = migration_files.get(&last_version) {
            if let Some(down_sql) = &migration_file.down_sql {
                info!("Rolling back migration: {} - {}", migration_file.version, migration_file.name);
                
                // 执行回滚SQL
                self.execute_sql_statements(down_sql).await
                    .context("Failed to execute rollback SQL")?;
                
                // 删除迁移记录
                let delete_sql = format!(
                    "DELETE FROM {} WHERE version = '{}'",
                    table_name, last_version
                );
                self.execute_ddl(&delete_sql).await?;
                
                info!("Successfully rolled back migration: {}", last_version);
            } else {
                return Err(anyhow!("Migration {} does not support rollback", last_version));
            }
        } else {
            return Err(anyhow!("Migration file for version {} not found", last_version));
        }
        
        Ok(())
    }
}

#[derive(Debug)]
enum Section {
    Up,
    Down,
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
    
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }
    
    pub fn total_executed(&self) -> usize {
        self.successful.len() + self.failed.len()
    }
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Migration Status for service: {}", self.service_name)?;
        writeln!(f, "  Table: {}", self.migrations_table)?;
        writeln!(f, "  Table exists: {}", self.table_exists)?;
        writeln!(f, "  Total applied migrations: {}", self.total_migrations)?;
        if let Some(ref last) = self.last_migration {
            writeln!(f, "  Last migration: {}", last)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for MigrationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Migration Summary:")?;
        writeln!(f, "  Successful: {}", self.successful.len())?;
        writeln!(f, "  Failed: {}", self.failed.len())?;
        writeln!(f, "  Total time: {:?}", self.total_time)?;
        
        if !self.failed.is_empty() {
            writeln!(f, "\nFailed migrations:")?;
            for failed in &self.failed {
                writeln!(f, "  - {}: {}", failed.version, failed.error)?;
            }
        }
        
        Ok(())
    }
}
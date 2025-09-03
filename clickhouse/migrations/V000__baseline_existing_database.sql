-- V000__baseline_existing_database.sql
-- 基线迁移：记录已存在的数据库状态

-- +migrate Up
-- 这个迁移文件用于标记数据库的基线状态
-- 由于 data_api_audit_log 和 test_ttl_where 表已经存在
-- 我们只需要创建一个空的迁移记录来标记基线

-- 注意：这是一个特殊的迁移文件，不执行任何SQL操作
-- 它只是用来标记迁移系统的起始点

-- +migrate Down
-- 基线迁移不需要回滚

-- V006__改变数据类型.sql

-- +migrate Up
-- 注意：ClickHouse 不支持直接修改列类型为数组类型
-- 我们需要使用不同的方法

-- 方法1：添加新列，然后删除旧列
ALTER TABLE users ADD COLUMN email_new Array(String) DEFAULT [];

-- 方法2：或者保持 email 为 String 类型，但添加注释说明
-- ALTER TABLE users COMMENT COLUMN email "Email address (considering Array<String> in future)";

-- +migrate Down
-- 恢复列类型
ALTER TABLE users DROP COLUMN IF EXISTS email_new;
-- 如果使用了方法2，则不需要回滚操作

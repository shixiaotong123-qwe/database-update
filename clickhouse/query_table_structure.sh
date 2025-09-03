#!/bin/bash

echo "🔍 查询 ClickHouse 表结构"
echo "================================"

# 设置 ClickHouse 连接参数
CLICKHOUSE_HOST="localhost"
CLICKHOUSE_PORT="8123"
CLICKHOUSE_USER="default"
CLICKHOUSE_PASSWORD="ClickHouse@123"
CLICKHOUSE_DB="default"

echo "📋 查询 data_api_audit_log 表结构:"
echo "----------------------------------------"
curl -s -u "$CLICKHOUSE_USER:$CLICKHOUSE_PASSWORD" \
  "http://$CLICKHOUSE_HOST:$CLICKHOUSE_PORT/?query=DESCRIBE%20$CLICKHOUSE_DB.data_api_audit_log%20FORMAT%20PrettyCompact"

echo -e "\n\n📋 查询 test_ttl_where 表结构:"
echo "----------------------------------------"
curl -s -u "$CLICKHOUSE_USER:$CLICKHOUSE_PASSWORD" \
  "http://$CLICKHOUSE_HOST:$CLICKHOUSE_PORT/?query=DESCRIBE%20$CLICKHOUSE_DB.test_ttl_where%20FORMAT%20PrettyCompact"

echo -e "\n\n📊 查询 data_api_audit_log 表数据 (前5条):"
echo "----------------------------------------"
curl -s -u "$CLICKHOUSE_USER:$CLICKHOUSE_PASSWORD" \
  "http://$CLICKHOUSE_HOST:$CLICKHOUSE_PORT/?query=SELECT%20*%20FROM%20$CLICKHOUSE_DB.data_api_audit_log%20LIMIT%205%20FORMAT%20PrettyCompact"

echo -e "\n\n📊 查询 test_ttl_where 表数据 (前5条):"
echo "----------------------------------------"
curl -s -u "$CLICKHOUSE_USER:$CLICKHOUSE_PASSWORD" \
  "http://$CLICKHOUSE_HOST:$CLICKHOUSE_PORT/?query=SELECT%20*%20FROM%20$CLICKHOUSE_DB.test_ttl_where%20LIMIT%205%20FORMAT%20PrettyCompact"

echo -e "\n\n🎉 表结构查询完成！"

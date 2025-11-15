#!/bin/bash

# ApiError 批量修复脚本
# 用于将元组风格的ApiError调用转换为结构风格

set -e

echo "🔧 开始修复 ApiError 使用方式..."

# 定义要修复的文件列表
FILES=(
    "src/api/handlers/agent_guide.rs"
    "src/api/handlers/claude.rs"
    "src/api/handlers/codex.rs"
    "src/api/handlers/common_config.rs"
    "src/api/handlers/mcp_server.rs"
    "src/services/claude_service.rs"
    "src/services/codex_service.rs"
    "src/api/server.rs"
)

# 修复模式定义
PATTERNS=(
    # Database 错误
    's/ApiError::Database(\([^)]*\))/ApiError::Database { message: \1 }/g'

    # NotFound 错误
    's/ApiError::NotFound(\([^)]*\))/ApiError::NotFound { resource: \1 }/g'

    # Internal 错误
    's/ApiError::Internal(\([^)]*\))/ApiError::Internal { message: \1 }/g'

    # ValidationError 错误 (这个比较复杂，需要特殊处理)
    # 's/ApiError::ValidationError(\([^)]*\))/ApiError::ValidationError { message: \1, field: None }/g'

    # BadRequest 错误
    's/ApiError::BadRequest(\([^)]*\))/ApiError::BadRequest { message: \1 }/g'

    # BusinessRule 错误
    's/ApiError::BusinessRule(\([^)]*\))/ApiError::BusinessRule { message: \1 }/g'

    # Unauthorized 错误
    's/ApiError::Unauthorized(\([^)]*\))/ApiError::Unauthorized { message: \1 }/g'

    # Forbidden 错误
    's/ApiError::Forbidden(\([^)]*\))/ApiError::Forbidden { message: \1 }/g'

    # Conflict 错误
    's/ApiError::Conflict(\([^)]*\))/ApiError::Conflict { message: \1 }/g'

    # Configuration 错误
    's/ApiError::Configuration(\([^)]*\))/ApiError::Configuration { message: \1 }/g'
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "修复文件: $file"

        # 备份原文件
        cp "$file" "$file.backup"

        # 应用所有修复模式
        for pattern in "${PATTERNS[@]}"; do
            sed -i "$pattern" "$file"
        done

        echo "✅ $file 修复完成"
    else
        echo "⚠️ 文件不存在: $file"
    fi
done

echo ""
echo "🎉 ApiError 修复完成！"
echo ""
echo "📋 修复摘要:"
echo "- 修复了 ${#FILES[@]} 个文件"
echo "- 应用了 ${#PATTERNS[@]} 个修复模式"
echo "- 原文件已备份为 .backup 文件"
echo ""
echo "📝 注意事项:"
echo "1. 请检查备份文件以确保没有意外修改"
echo "2. ValidationError 需要手动检查，因为它可能包含字段信息"
echo "3. 运行 'cargo check' 验证修复结果"
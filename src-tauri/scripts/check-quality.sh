#!/bin/bash

# AI Manager 代码质量检查脚本
#
# 此脚本用于自动执行代码质量检查，包括：
# - Rust 格式化检查 (rustfmt)
# - Clippy 静态分析
# - 单元测试
# - 文档检查
# - 依赖安全审计

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 计数器
CHECKS_PASSED=0
CHECKS_FAILED=0
TOTAL_CHECKS=0

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((CHECKS_PASSED++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((CHECKS_FAILED++))
}

# 开始检查
echo "🚀 AI Manager 代码质量检查开始..."
echo "=================================="

# 检查 Rust 工具链
log_info "检查 Rust 工具链..."
if ! command -v cargo &> /dev/null; then
    log_error "Cargo 未找到，请确保已安装 Rust"
    exit 1
fi

# 1. 代码格式化检查
log_info "1/6 运行 Rust 格式化检查..."
((TOTAL_CHECKS++))
if cargo fmt --all -- --check; then
    log_success "代码格式检查通过"
else
    log_error "代码格式检查失败，请运行 'cargo fmt' 修复格式问题"
    echo "   建议运行: cargo fmt"
fi

# 2. Clippy 静态分析
log_info "2/6 运行 Clippy 静态分析..."
((TOTAL_CHECKS++))
if cargo clippy --all-targets --all-features -- -D warnings; then
    log_success "Clippy 检查通过"
else
    log_error "Clippy 检查失败，请修复警告和错误"
    echo "   建议运行: cargo clippy --fix"
fi

# 3. 编译检查
log_info "3/6 运行编译检查..."
((TOTAL_CHECKS++))
if cargo check --all-targets --all-features; then
    log_success "编译检查通过"
else
    log_error "编译检查失败，项目无法编译"
fi

# 4. 单元测试
log_info "4/6 运行单元测试..."
((TOTAL_CHECKS++))
if cargo test --lib --bins; then
    log_success "单元测试通过"
else
    log_error "单元测试失败"
    echo "   建议运行: cargo test 查看详细错误"
fi

# 5. 文档生成检查
log_info "5/6 检查文档生成..."
((TOTAL_CHECKS++))
if cargo doc --no-deps --document-private-items; then
    log_success "文档生成检查通过"
else
    log_error "文档生成失败，请检查文档注释"
fi

# 6. 依赖安全审计
log_info "6/6 运行依赖安全审计..."
((TOTAL_CHECKS++))
if cargo audit; then
    log_success "依赖安全审计通过"
else
    log_warning "依赖安全审计发现问题，请查看报告"
    echo "   建议运行: cargo audit fix"
fi

# 统计结果
echo "=================================="
echo "📊 检查结果统计:"
echo "   总检查项: $TOTAL_CHECKS"
echo "   通过: $CHECKS_PASSED"
echo "   失败: $CHECKS_FAILED"

if [ $CHECKS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 所有代码质量检查通过！${NC}"
    echo "项目可以安全提交和部署。"
    exit 0
else
    echo -e "\n${RED}❌ 有 $CHECKS_FAILED 项检查失败${NC}"
    echo "请修复上述问题后重新运行检查。"
    exit 1
fi
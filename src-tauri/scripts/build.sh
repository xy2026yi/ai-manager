#!/bin/bash

# AI Manager 自动化构建脚本
# 支持多平台构建和签名

set -e

# 配置变量
APP_NAME="AI Manager"
VERSION="0.1.0"
BUILD_DIR="target/release"
BUNDLE_DIR="$BUILD_DIR/bundle"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查构建依赖..."

    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo 未安装"
        exit 1
    fi

    if ! command -v npm &> /dev/null; then
        log_error "Node.js/npm 未安装"
        exit 1
    fi

    log_info "依赖检查完成"
}

# 清理构建环境
clean_build() {
    log_info "清理构建环境..."
    cargo clean
    rm -rf $BUILD_DIR
}

# 安装前端依赖
install_frontend_deps() {
    log_info "安装前端依赖..."
    if [ ! -d "node_modules" ]; then
        npm install
    fi
}

# 构建前端
build_frontend() {
    log_info "构建前端..."
    npm run build
}

# 构建Rust后端
build_rust() {
    log_info "构建Rust后端（Release模式）..."
    cargo build --release
}

# 创建应用包
create_bundles() {
    log_info "创建应用包..."
    cargo tauri build -- --bundles all
}

# 优化包大小
optimize_packages() {
    log_info "优化包大小..."

    # 压缩安装包
    find $BUNDLE_DIR -name "*.exe" -exec upx --best {} \; 2>/dev/null || true
    find $BUNDLE_DIR -name "*.AppImage" -exec upx --best {} \; 2>/dev/null || true
}

# 生成构建报告
generate_build_report() {
    log_info "生成构建报告..."

    REPORT_FILE="build-report-$(date +%Y%m%d-%H%M%S).txt"

    cat > $REPORT_FILE << EOF
AI Manager 构建报告
==================

构建时间: $(date)
版本: $VERSION
构建环境: $(uname -a)
Rust版本: $(rustc --version)
Node.js版本: $(node --version)

文件大小统计:
EOF

    if [ -d "$BUNDLE_DIR" ]; then
        find $BUNDLE_DIR -type f -exec ls -lh {} \; | awk '{print $9 ": " $5}' >> $REPORT_FILE
    fi

    log_info "构建报告已生成: $REPORT_FILE"
}

# 主构建流程
main() {
    log_info "开始构建 $APP_NAME v$VERSION"

    check_dependencies
    clean_build
    install_frontend_deps
    build_frontend
    build_rust
    create_bundles
    optimize_packages
    generate_build_report

    log_info "构建完成！"
    log_info "构建产物位于: $BUNDLE_DIR"

    # 显示主要文件大小
    if [ -d "$BUNDLE_DIR" ]; then
        log_info "主要文件大小:"
        find $BUNDLE_DIR -name "*.exe" -o -name "*.dmg" -o -name "*.AppImage" -o -name "*.deb" | while read file; do
            size=$(ls -lh "$file" | awk '{print $5}')
            log_info "  $(basename $file): $size"
        done
    fi
}

# 解析命令行参数
case "${1:-build}" in
    "clean")
        clean_build
        ;;
    "deps")
        check_dependencies
        install_frontend_deps
        ;;
    "build")
        main
        ;;
    "release")
        log_warn "发布构建将创建所有平台的安装包"
        main
        ;;
    *)
        echo "用法: $0 [clean|deps|build|release]"
        echo "  clean   - 清理构建环境"
        echo "  deps    - 安装依赖"
        echo "  build   - 执行完整构建"
        echo "  release - 发布构建（所有平台）"
        exit 1
        ;;
esac
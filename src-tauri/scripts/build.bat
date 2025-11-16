@echo off
REM AI Manager Windows 构建脚本

setlocal enabledelayedexpansion

set APP_NAME=AI Manager
set VERSION=0.1.0
set BUILD_DIR=target\release

echo [INFO] 开始构建 %APP_NAME% v%VERSION%

REM 检查依赖
echo [INFO] 检查构建依赖...
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust/Cargo 未安装
    exit /b 1
)

where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js/npm 未安装
    exit /b 1
)

echo [INFO] 依赖检查完成

REM 清理构建环境
echo [INFO] 清理构建环境...
if exist %BUILD_DIR% rmdir /s /q %BUILD_DIR%
cargo clean

REM 安装前端依赖
echo [INFO] 安装前端依赖...
if not exist node_modules npm install

REM 构建前端
echo [INFO] 构建前端...
npm run build
if %errorlevel% neq 0 (
    echo [ERROR] 前端构建失败
    exit /b 1
)

REM 构建Rust后端
echo [INFO] 构建Rust后端（Release模式）...
cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Rust构建失败
    exit /b 1
)

REM 创建应用包
echo [INFO] 创建应用包...
cargo tauri build -- --bundles all
if %errorlevel% neq 0 (
    echo [ERROR] 应用包创建失败
    exit /b 1
)

echo [INFO] 构建完成！
echo [INFO] 构建产物位于: %BUILD_DIR%\bundle

REM 显示主要文件大小
if exist %BUILD_DIR%\bundle (
    echo [INFO] 主要文件大小:
    for %%f in (%BUILD_DIR%\bundle\msi\*.msi %BUILD_DIR%\bundle\nsis\*.exe %BUILD_DIR%\bundle\msi\*.msi) do (
        if exist "%%f" (
            for %%A in ("%%f") do echo   %%~nxA: %%~zA bytes
        )
    )
)

pause
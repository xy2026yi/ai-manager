# AI Manager 安装指南

## 概述

本指南详细介绍了 AI Manager 在不同操作系统上的安装方法、系统要求和配置步骤。

## 系统要求

### 通用要求

- **处理器**: 双核 2.0GHz 或更高
- **内存**: 2GB RAM (推荐 4GB 或更高)
- **存储空间**: 100MB 可用磁盘空间
- **网络**: 稳定的互联网连接（用于AI服务API调用）

### 操作系统支持

| 操作系统 | 最低版本 | 架构支持 |
|---------|---------|---------|
| Windows | 10 (1903) | x64 |
| macOS | 10.13 (High Sierra) | x64, ARM64 |
| Linux | Ubuntu 18.04+ | x64 |

## 安装方法

### Windows 安装

#### 方法一：使用安装包（推荐）

1. **下载安装包**
   ```cmd
   # 从官网下载或使用 curl
   curl -o AI-Manager-Setup.exe https://releases.ai-manager.com/v0.1.0/AI-Manager_0.1.0_x64-setup.exe
   ```

2. **验证文件完整性**（可选）
   ```cmd
   certutil -hashfile AI-Manager-Setup.exe SHA256
   ```
   比对哈希值与官网提供的一致。

3. **运行安装程序**
   - 右键点击 `AI-Manager-Setup.exe`
   - 选择"以管理员身份运行"
   - 按照安装向导完成安装

4. **安装选项**
   - **安装位置**: 默认 `C:\Program Files\AI Manager`
   - **开始菜单**: 创建快捷方式
   - **桌面快捷方式**: 可选创建
   - **开机启动**: 可选（不推荐）

5. **完成安装**
   - 点击"完成"启动应用
   - 或从开始菜单启动

#### 方法二：便携版安装

1. **下载便携版**
   ```cmd
   curl -o AI-Manager-Portable.zip https://releases.ai-manager.com/v0.1.0/AI-Manager_0.1.0_x64_portable.zip
   ```

2. **解压到目标目录**
   ```cmd
   # 创建安装目录
   mkdir C:\AI-Manager

   # 解压文件
   tar -xf AI-Manager-Portable.zip -C C:\AI-Manager
   ```

3. **首次运行**
   ```cmd
   cd C:\AI-Manager
   AI-Manager.exe
   ```

#### Windows 配置

**防火墙设置**:
- 首次运行时，Windows 可能会弹出防火墙提示
- 选择"允许访问"以启用网络功能

**杀毒软件白名单**:
- 如果杀毒软件误报，请将 AI Manager 添加到白名单
- 官方下载的安装包已通过数字签名验证

### macOS 安装

#### 方法一：使用安装包（推荐）

1. **下载安装包**
   ```bash
   # Intel Mac
   curl -O https://releases.ai-manager.com/v0.1.0/AI-Manager_0.1.0_x64.app.tar.gz

   # Apple Silicon Mac
   curl -O https://releases.ai-manager.com/v0.1.0/AI-Manager_0.1.0_aarch64.app.tar.gz
   ```

2. **验证下载**
   ```bash
   # Intel Mac
   shasum -a 256 AI-Manager_0.1.0_x64.app.tar.gz

   # Apple Silicon Mac
   shasum -a 256 AI-Manager_0.1.0_aarch64.app.tar.gz
   ```

3. **解压安装**
   ```bash
   # Intel Mac
   tar -xzf AI-Manager_0.1.0_x64.app.tar.gz
   mv AI-Manager.app /Applications/

   # Apple Silicon Mac
   tar -xzf AI-Manager_0.1.0_aarch64.app.tar.gz
   mv AI-Manager.app /Applications/
   ```

4. **首次启动**
   ```bash
   open /Applications/AI-Manager.app
   ```

#### 方法二：使用 Homebrew

1. **添加 Homebrew 仓库**
   ```bash
   brew tap ai-manager/homebrew-ai-manager
   ```

2. **安装应用**
   ```bash
   brew install ai-manager
   ```

#### 方法三：使用 MacPorts

1. **安装 MacPorts**（如果未安装）
   访问 https://www.macports.org/install.php

2. **安装应用**
   ```bash
   sudo port install ai-manager
   ```

#### macOS 配置

**安全设置**:
首次启动时可能需要在"系统偏好设置"中允许应用运行：

1. 打开"系统偏好设置" > "安全性与隐私"
2. 在"通用"标签页中，点击"仍要打开"
3. 输入管理员密码确认

**权限设置**:
- 文件系统访问：允许访问用户文档和下载文件夹
- 网络访问：允许访问网络以调用 AI 服务 API

### Linux 安装

#### 方法一：AppImage（推荐，通用）

1. **下载 AppImage**
   ```bash
   wget https://releases.ai-manager.com/v0.1.0/AI-Manager_0.1.0_amd64.AppImage
   ```

2. **设置执行权限**
   ```bash
   chmod +x AI-Manager_0.1.0_amd64.AppImage
   ```

3. **首次运行**
   ```bash
   ./AI-Manager_0.1.0_amd64.AppImage
   ```

4. **创建桌面快捷方式**（可选）
   ```bash
   # 创建 .desktop 文件
   cat > ~/.local/share/applications/ai-manager.desktop << EOF
   [Desktop Entry]
   Version=1.0
   Type=Application
   Name=AI Manager
   Comment=AI 工具管理平台
   Exec=$(pwd)/AI-Manager_0.1.0_amd64.AppImage
   Icon=$(pwd)/AI-Manager_0.1.0_amd64.AppImage
   Terminal=false
   Categories=Utility;Development;
   EOF

   # 更新桌面数据库
   update-desktop-database ~/.local/share/applications/
   ```

#### 方法二：Debian/Ubuntu 使用 .deb 包

1. **下载 .deb 包**
   ```bash
   wget https://releases.ai-manager.com/v0.1.0/ai-manager_0.1.0_amd64.deb
   ```

2. **安装依赖**
   ```bash
   sudo apt-get update
   sudo apt-get install -y libwebkit2gtk-4.0-37 libnotify4 libnss3 libxss1 libxtst6 xdg-utils libatspi2.0-0 libdrm2 libxcomposite1 libxdamage1 libxrandr2 libgbm1 libxkbcommon0 libxfixes3
   ```

3. **安装应用**
   ```bash
   sudo dpkg -i ai-manager_0.1.0_amd64.deb

   # 如果出现依赖问题，运行以下命令
   sudo apt-get install -f
   ```

#### 方法三：Fedora/CentOS 使用 .rpm 包

1. **下载 .rpm 包**
   ```bash
   wget https://releases.ai-manager.com/v0.1.0/ai-manager-0.1.0-1.x86_64.rpm
   ```

2. **安装依赖**
   ```bash
   # Fedora
   sudo dnf install -y webkit2gtk3-devel.x86_64 libXScrnSaver.x86_64

   # CentOS/RHEL
   sudo yum install -y webkit2gtk3-devel.x86_64 libXScrnSaver.x86_64
   ```

3. **安装应用**
   ```bash
   # Fedora
   sudo dnf install -y ai-manager-0.1.0-1.x86_64.rpm

   # CentOS/RHEL
   sudo yum install -y ai-manager-0.1.0-1.x86_64.rpm
   ```

#### 方法四：使用包管理器

**Arch Linux (AUR)**:
```bash
# 使用 yay
yay -S ai-manager

# 使用 paru
paru -S ai-manager
```

**openSUSE**:
```bash
# 添加软件源
sudo zypper addrepo https://download.opensuse.org/repositories/home:/ai-manager/openSUSE_Tumbleweed/home:ai-manager.repo

# 安装应用
sudo zypper install ai-manager
```

#### Linux 配置

**依赖安装**:
确保安装了必要的运行时依赖：

**Ubuntu/Debian**:
```bash
sudo apt-get install -y \
    libwebkit2gtk-4.0-37 \
    libnotify4 \
    libnss3 \
    libxss1 \
    libxtst6 \
    xdg-utils \
    libatspi2.0-0 \
    libdrm2 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    libgbm1 \
    libxkbcommon0 \
    libxfixes3
```

**Fedora**:
```bash
sudo dnf install -y \
    webkit2gtk3-devel.x86_64 \
    libXScrnSaver.x86_64 \
    atk.x86_64 \
    cups-libs.x86_64 \
    gtk3.x86_64 \
    libXrandr.x86_64 \
    libXcomposite.x86_64 \
    libXdamage.x86_64 \
    libXcursor.x86_64 \
    libXi.x86_64 \
    libXtst.x86_64 \
    pango.x86_64 \
    libXScrnSaver.x86_64 \
    libXrandr.x86_64 \
    alsa-lib.x86_64 \
    gtk3.x86_64 \
    nss.x86_64 \
    libgbm.x86_64
```

**桌面集成**:
- 创建桌面快捷方式
- 添加到应用菜单
- 设置文件关联

## 验证安装

### 检查应用版本

启动应用后，在"关于"页面查看版本信息：

- **应用版本**: 应显示 v0.1.0
- **构建信息**: 显示构建日期和平台信息
- **更新状态**: 检查是否有可用更新

### 功能测试

1. **启动测试**
   - 应用能正常启动
   - 界面显示正常
   - 响应速度正常

2. **配置测试**
   - 能添加 Claude 供应商
   - 能保存配置
   - 能测试连接

3. **网络测试**
   - 能访问 AI 服务 API
   - 网络请求正常

### 命令行验证

**检查安装路径**:
```bash
# Windows
where ai-manager

# macOS
which ai-manager

# Linux
which ai-manager
```

**检查版本信息**:
```bash
# 如果支持命令行参数
ai-manager --version
```

## 卸载指南

### Windows 卸载

1. **使用控制面板**
   - 打开"控制面板" > "程序和功能"
   - 找到 "AI Manager"
   - 点击"卸载"

2. **手动清理**（可选）
   ```cmd
   # 删除配置文件
   rmdir /s "%APPDATA%\ai-manager"

   # 删除缓存文件
   rmdir /s "%LOCALAPPDATA%\ai-manager"
   ```

### macOS 卸载

1. **删除应用**
   ```bash
   rm -rf /Applications/AI-Manager.app
   ```

2. **清理配置文件**
   ```bash
   rm -rf ~/Library/Application\ Support/ai-manager
   rm -rf ~/Library/Preferences/com.aimanager.migration.plist
   rm -rf ~/Library/Caches/ai-manager
   ```

### Linux 卸载

1. **使用包管理器卸载**
   ```bash
   # Debian/Ubuntu
   sudo apt-get remove ai-manager
   sudo apt-get autoremove

   # Fedora
   sudo dnf remove ai-manager

   # Arch (AUR)
   yay -R ai-manager
   ```

2. **手动卸载 AppImage**
   ```bash
   # 删除 AppImage 文件
   rm AI-Manager_0.1.0_amd64.AppImage

   # 删除配置文件
   rm -rf ~/.config/ai-manager
   rm -rf ~/.local/share/ai-manager
   ```

## 故障排除

### 常见安装问题

**问题 1**: Windows 安装时出现"无法验证发布者"
**解决方案**:
- 右键点击安装包 > 属性
- 勾选"解除锁定"
- 重新运行安装程序

**问题 2**: macOS 提示"已损坏，无法打开"
**解决方案**:
```bash
# 移除隔离属性
xattr -d com.apple.quarantine /Applications/AI-Manager.app
```

**问题 3**: Linux AppImage 无法启动
**解决方案**:
```bash
# 安装 FUSE
sudo apt-get install libfuse2

# 或者使用 --no-sandbox 参数
./AI-Manager_0.1.0_amd64.AppImage --no-sandbox
```

**问题 4**: 依赖库缺失
**解决方案**:
- 参考上述各平台的依赖安装命令
- 使用包管理器安装所需的库文件

### 性能问题

**启动缓慢**:
- 检查磁盘空间是否充足
- 关闭杀毒软件实时扫描
- 重启应用清理缓存

**内存占用过高**:
- 减少同时打开的配置数量
- 定期重启应用
- 检查是否有内存泄漏

### 获取帮助

如果遇到安装问题，请：

1. **查看日志文件**:
   - Windows: `%APPDATA%\ai-manager\logs\`
   - macOS: `~/Library/Logs/ai-manager/`
   - Linux: `~/.local/share/ai-manager/logs/`

2. **访问支持页面**:
   - 官方文档: https://docs.ai-manager.com
   - 问题反馈: https://github.com/ai-manager/migration/issues
   - 技术支持: support@ai-manager.com

3. **提供系统信息**:
   - 操作系统版本
   - 错误截图或日志
   - 安装包版本和哈希值

---

**版本**: 0.1.0
**最后更新**: 2025年11月16日
**支持平台**: Windows, macOS, Linux
# 全平台图标使用指南

本目录包含基于 `docs/tool.svg` 生成的全平台图标文件。

## 📁 目录结构

```
icons/
├── web/          # Web 平台图标
├── windows/      # Windows 平台图标
├── macos/        # macOS 平台图标
├── linux/        # Linux 平台图标
├── mobile/       # 移动平台图标
├── sources/      # 源文件和高分辨率版本
└── README.md     # 本说明文件
```

## 🌐 Web 平台图标 (`web/`)

### 文件列表
- `favicon.ico` - 网站图标，包含 16x16, 32x32, 48x48 尺寸
- `icon-16x16.png` 到 `icon-512x512.png` - 各种尺寸的 PNG 图标
- `icon-64x64-dark.png` - 深色背景版本
- `icon-64x64-light.png` - 浅色背景版本

### 使用方法
```html
<!-- HTML 中使用 favicon -->
<link rel="icon" href="icons/web/favicon.ico" type="image/x-icon">

<!-- 不同尺寸的图标 -->
<link rel="icon" sizes="16x16" href="icons/web/icon-16x16.png" type="image/png">
<link rel="icon" sizes="32x32" href="icons/web/icon-32x32.png" type="image/png">
<link rel="icon" sizes="192x192" href="icons/web/icon-192x192.png" type="image/png">

<!-- PWA 应用图标 -->
<link rel="apple-touch-icon" sizes="180x180" href="icons/web/icon-180x180.png">
<link rel="manifest" href="manifest.json">
```

## 🪟 Windows 平台图标 (`windows/`)

### 文件列表
- `app.ico` - 应用程序图标，包含多种尺寸
- `icon-16x16.png` 到 `icon-256x256.png` - 各种尺寸的 PNG 图标

### 使用方法
- **应用程序图标**: 使用 `app.ico` 文件
- **Windows 资源文件**: 在 `.rc` 文件中引用对应的 PNG 文件
- **Windows 11**: 推荐使用 PNG 格式的高分辨率图标

## 🍎 macOS 平台图标 (`macos/`)

### 文件列表
- `icon_16x16.png`, `icon_32x32.png`, `icon_64x64.png`, `icon_128x128.png`, `icon_256x256.png`, `icon_512x512.png`, `icon_1024x1024.png` - 标准尺寸
- `icon_*x*@2x.png` - Retina 高分辨率版本

### 生成 ICNS 文件
在 macOS 上使用 `iconutil` 工具生成 ICNS 文件：
```bash
# 创建图标集目录
mkdir -p MyIcon.iconset

# 复制对应尺寸的文件
cp icons/macos/icon_16x16.png MyIcon.iconset/icon_16x16.png
cp icons/macos/icon_16x16@2x.png MyIcon.iconset/icon_16x16@2x.png
cp icons/macos/icon_32x32.png MyIcon.iconset/icon_32x32.png
cp icons/macos/icon_32x32@2x.png MyIcon.iconset/icon_32x32@2x.png
# ... 继续复制其他尺寸

# 生成 ICNS 文件
iconutil -c icns MyIcon.iconset
```

## 🐧 Linux 平台图标 (`linux/`)

### 文件列表
- `icon-16x16.png` 到 `icon-512x512.png` - 符合 freedesktop.org 标准的尺寸

### 使用方法
- **桌面应用**: 复制对应尺寸到 `/usr/share/icons/hicolor/<size>/apps/`
- **主题图标**: 遵循 freedesktop.org 图标主题规范

## 📱 移动平台图标 (`mobile/`)

### iOS 图标
- `ios-57x57.png` 到 `ios-220x220.png` - iOS 应用所需的各种尺寸

### Android 图标
- `android-36x36.png` 到 `android-512x512.png` - Android 应用图标

### 使用方法
- **iOS**: 在 Xcode 项目中设置 App Icon
- **Android**: 在 `android:icon` 属性中引用对应图标文件

## 🔧 源文件 (`sources/`)

- `icon-1024x1024.png` - 高分辨率源文件
- `icon-2048x2048.png` - 超高分辨率源文件

## 📏 图标尺寸参考

| 平台 | 用途 | 推荐尺寸 |
|------|------|----------|
| Web | Favicon | 16x16, 32x32, 48x48 |
| Web | PWA | 192x192, 512x512 |
| Windows | 应用图标 | 16x16 - 256x256 |
| macOS | 应用图标 | 16x16 - 1024x1024 |
| Linux | 桌面图标 | 16x16 - 512x512 |
| iOS | 应用图标 | 57x57 - 220x220 |
| Android | 应用图标 | 36x36 - 512x512 |

## 🎨 设计说明

- **原始设计**: 基于 Feather Icons 的工具图标
- **风格**: 线条风格，stroke-width="2"
- **颜色**: 使用 currentColor，支持主题适配
- **背景**: 透明背景，可适配各种背景色

## 🔄 重新生成图标

如需重新生成图标，运行：
```bash
python3 generate_icons.py
```

## 📄 许可证

本图标基于原始 SVG 文件生成，请遵循相应的开源许可证。
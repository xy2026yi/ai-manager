#!/usr/bin/env python3
"""
全平台图标生成脚本
基于 docs/tool.svg 生成各平台所需的各种尺寸图标
"""

import os
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import cairosvg
from io import BytesIO

def ensure_dir(path):
    """确保目录存在"""
    Path(path).mkdir(parents=True, exist_ok=True)

def svg_to_png(svg_path, size, output_path, background_color=None):
    """
    将 SVG 转换为 PNG

    Args:
        svg_path: SVG 文件路径
        size: 输出尺寸（正方形）
        output_path: 输出 PNG 文件路径
        background_color: 背景色，None 表示透明背景
    """
    try:
        # 使用 CairoSVG 将 SVG 转换为 PNG
        png_data = cairosvg.svg2png(
            url=svg_path,
            output_width=size,
            output_height=size,
            background_color=background_color
        )

        # 使用 PIL 优化图像
        img = Image.open(BytesIO(png_data))

        # 如果指定了背景色，确保图像有该背景
        if background_color:
            background = Image.new('RGBA', (size, size), background_color)
            if img.mode == 'RGBA':
                background.paste(img, (0, 0), img)
            else:
                background.paste(img, (0, 0))
            img = background

        # 保存为 PNG
        img.save(output_path, 'PNG', optimize=True)
        print(f"✓ 生成: {output_path} ({size}x{size})")
        return True
    except Exception as e:
        print(f"✗ 生成失败 {output_path}: {e}")
        return False

def create_favicon(sizes, output_path):
    """创建 ICO 文件（favicon）"""
    try:
        images = []
        for size in sizes:
            png_path = f"temp_{size}x{size}.png"
            if svg_to_png('docs/tool.svg', size, png_path):
                img = Image.open(png_path)
                images.append(img)
                os.remove(png_path)

        if images:
            images[0].save(output_path, format='ICO', sizes=[(img.width, img.height) for img in images])
            print(f"✓ 生成: {output_path} (favicon)")
            return True
    except Exception as e:
        print(f"✗ 生成 favicon 失败: {e}")
    return False

def create_icns(macos_sizes, output_path):
    """创建 ICNS 文件（macOS 图标）"""
    # macOS 的 ICNS 创建需要额外工具，这里先生成 PNG 文件
    print("注意: ICNS 文件需要额外工具（如 iconutil）在 macOS 上生成")
    print("这里先生成所需的 PNG 文件")

    for size in macos_sizes:
        png_path = f"icons/macos/icon_{size}x{size}.png"
        svg_to_png('docs/tool.svg', size, png_path)

        # 生成 @2x 版本
        if size <= 512:
            png_path_2x = f"icons/macos/icon_{size}x{size}@2x.png"
            svg_to_png('docs/tool.svg', size * 2, png_path_2x)

def generate_platform_icons():
    """生成所有平台的图标"""

    # 确保输出目录存在
    ensure_dir('icons/web')
    ensure_dir('icons/windows')
    ensure_dir('icons/macos')
    ensure_dir('icons/linux')
    ensure_dir('icons/mobile')

    print("🚀 开始生成全平台图标...")
    print("=" * 50)

    # Web 平台图标
    print("\n📱 生成 Web 平台图标:")
    web_sizes = [16, 32, 48, 64, 128, 256, 512]
    for size in web_sizes:
        svg_to_png('docs/tool.svg', size, f'icons/web/icon-{size}x{size}.png')

    # 生成 favicon.ico
    favicon_sizes = [16, 32, 48]
    create_favicon(favicon_sizes, 'icons/web/favicon.ico')

    # Windows 平台图标
    print("\n🪟 生成 Windows 平台图标:")
    windows_sizes = [16, 32, 48, 64, 128, 256]
    for size in windows_sizes:
        svg_to_png('docs/tool.svg', size, f'icons/windows/icon-{size}x{size}.png')

    # 生成 Windows ICO
    create_favicon(windows_sizes, 'icons/windows/app.ico')

    # macOS 平台图标
    print("\n🍎 生成 macOS 平台图标:")
    macos_sizes = [16, 32, 64, 128, 256, 512, 1024]
    create_icns(macos_sizes, 'icons/macos/app.icns')

    # Linux 平台图标
    print("\n🐧 生成 Linux 平台图标:")
    linux_sizes = [16, 24, 32, 48, 64, 128, 256, 512]
    for size in linux_sizes:
        svg_to_png('docs/tool.svg', size, f'icons/linux/icon-{size}x{size}.png')

    # 移动平台图标
    print("\n📲 生成移动平台图标:")

    # iOS 图标
    ios_sizes = [57, 60, 72, 76, 114, 120, 144, 152, 180, 192, 220]
    for size in ios_sizes:
        svg_to_png('docs/tool.svg', size, f'icons/mobile/ios-{size}x{size}.png')

    # Android 图标
    android_sizes = [36, 48, 72, 96, 144, 192, 256, 512]
    for size in android_sizes:
        svg_to_png('docs/tool.svg', size, f'icons/mobile/android-{size}x{size}.png')

    # 生成一些特殊用途的图标
    print("\n🎯 生成特殊用途图标:")

    # 带背景的版本（用于深色主题）
    svg_to_png('docs/tool.svg', 64, 'icons/web/icon-64x64-dark.png', background_color='#ffffff')
    svg_to_png('docs/tool.svg', 64, 'icons/web/icon-64x64-light.png', background_color='#000000')

    # 高分辨率版本
    svg_to_png('docs/tool.svg', 1024, 'icons/sources/icon-1024x1024.png')
    svg_to_png('docs/tool.svg', 2048, 'icons/sources/icon-2048x2048.png')

    print("\n✅ 图标生成完成!")
    print("=" * 50)

    # 生成统计报告
    print("\n📊 生成统计:")
    total_files = 0
    for root, dirs, files in os.walk('icons'):
        for file in files:
            if file.endswith('.png') or file.endswith('.ico'):
                total_files += 1

    print(f"总共生成了 {total_files} 个图标文件")
    print("\n📁 输出目录结构:")
    print("icons/")
    print("├── web/          # Web 平台图标 (favicon, 各种尺寸 PNG)")
    print("├── windows/      # Windows 平台图标 (ICO, PNG)")
    print("├── macos/        # macOS 平台图标 (PNG, 需要转换为 ICNS)")
    print("├── linux/        # Linux 平台图标 (PNG)")
    print("├── mobile/       # 移动平台图标 (iOS, Android)")
    print("└── sources/      # 源文件和高分辨率版本")

if __name__ == '__main__':
    if not os.path.exists('docs/tool.svg'):
        print("错误: 找不到 docs/tool.svg 文件")
        sys.exit(1)

    generate_platform_icons()
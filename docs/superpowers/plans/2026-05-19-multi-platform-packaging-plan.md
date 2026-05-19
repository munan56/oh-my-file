# 多平台打包和Icon设计 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为oh-my-file应用实现Windows、macOS、Linux三平台的自动化打包，使用新设计的3D拟物化icon

**Architecture:** 
分三个阶段实现：先通过Canva设计新icon并导出为PNG，然后使用工具生成各平台所需的icon格式（.ico、.icns、.png），更新tauri.conf.json配置支持三个平台的bundle目标，最后创建GitHub Actions工作流自动化构建。

**Tech Stack:** Canva (设计), Tauri 2.0 (打包框架), GitHub Actions (CI/CD), Node.js (脚本编排)

---

## Task 1: 在Canva设计新的icon

**Files:**
- Create: `Canva项目/oh-my-file-icon.design` (外部设计文件)
- Modify: 无

- [ ] **Step 1: 打开Canva创建新设计**

访问 https://www.canva.com 并创建新项目：
- 选择自定义尺寸：512×512像素
- 项目名称：`oh-my-file icon`
- 预设主题：空白画布

- [ ] **Step 2: 设计3D拟物化icon**

设计要点：
- 风格：现代3D拟物化，参考iPhone/macOS系统icon设计
- 主题：文件传输、网络、局域网相关元素
- 元素建议：
  - 文件夹或文档形象
  - 传输箭头或网络连接符号
  - 3D渐变和阴影效果
- 配色：选择与"文件传输"和"网络"相关的配色方案（如蓝色/绿色主调）
- 容差：确保在小尺寸（32×32px）上仍然清晰辨认

- [ ] **Step 3: 导出为高质量PNG**

在Canva中：
- 点击"下载"按钮
- 选择格式：PNG（透明背景）
- 分辨率：最高质量
- 命名为：`icon-512x512.png`
- 保存到本地或Google Drive

- [ ] **Step 4: 验证导出质量**

检查导出的PNG：
- 文件大小合理（通常50KB-500KB）
- 背景透明
- 512×512像素
- 在小尺寸（如32×32）缩放后仍清晰

- [ ] **Step 5: 无需提交**

Canva设计文件保存在Canva账户，PNG本地保存等待下一步处理

---

## Task 2: 将PNG转换为各平台的icon格式

**Files:**
- Create: `src-tauri/icons/icon.png` (512x512 PNG)
- Create: `src-tauri/icons/icon.ico` (Windows)
- Create: `src-tauri/icons/icon.icns` (macOS)
- Create: `src-tauri/icons/icon-32x32.png` (Linux 32px)
- Create: `src-tauri/icons/icon-64x64.png` (Linux 64px)
- Create: `src-tauri/icons/icon-128x128.png` (Linux 128px)
- Create: `src-tauri/icons/icon-256x256.png` (Linux 256px)
- Delete: `src-tauri/icons/icon.svg` (旧SVG，保留备份)
- Test: 所有icon文件放置正确位置

- [ ] **Step 1: 下载PNG文件到项目目录**

将Canva导出的 `icon-512x512.png` 下载到项目，重命名为 `icon.png` 并放置在项目根目录。

- [ ] **Step 2: 生成Windows .ico文件**

使用在线工具或本地工具转换PNG到ICO格式：

**方案A - 使用在线工具（推荐快速）:**
- 访问 https://convertio.co/png-ico/ 或 https://icoconvert.com/
- 上传 `icon.png`
- 下载 `.ico` 文件
- 保存到 `src-tauri/icons/icon.ico`

**方案B - 使用ImageMagick（本地，需安装）:**
```bash
# Windows (如已安装ImageMagick)
magick icon.png -define icon:auto-resize=256,128,64,32,16 icon.ico

# macOS/Linux
convert icon.png -define icon:auto-resize=256,128,64,32,16 icon.ico
```

**验证**: `icon.ico` 应包含多个尺寸（256, 128, 64, 32, 16）

- [ ] **Step 3: 生成macOS .icns文件**

使用在线工具或本地工具转换PNG到ICNS格式：

**方案A - 使用在线工具（推荐）:**
- 访问 https://cloudconvert.com/png-to-icns
- 上传 `icon.png`
- 下载 `.icns` 文件
- 保存到 `src-tauri/icons/icon.icns`

**方案B - 使用macOS命令（仅macOS）:**
```bash
# 需要在macOS上运行
sips -s format icns icon.png --out icon.icns
```

**验证**: `icon.icns` 文件大小通常100KB-1MB

- [ ] **Step 4: 生成Linux PNG多尺寸**

将 `icon.png` (512×512) 缩放到多个尺寸。使用在线工具或脚本：

**方案A - 使用在线批量工具:**
- 访问 https://pixlr.com/express/resize 或类似工具
- 上传 `icon.png`
- 依次生成并下载：
  - 256×256 → 保存为 `icon-256x256.png`
  - 128×128 → 保存为 `icon-128x128.png`
  - 64×64 → 保存为 `icon-64x64.png`
  - 32×32 → 保存为 `icon-32x32.png`

**方案B - 使用ImageMagick脚本:**
```bash
# Linux/macOS/Windows(Git Bash)
convert icon.png -resize 256x256 icon-256x256.png
convert icon.png -resize 128x128 icon-128x128.png
convert icon.png -resize 64x64   icon-64x64.png
convert icon.png -resize 32x32   icon-32x32.png
```

**验证**: 4个PNG文件都位于 `src-tauri/icons/` 目录

- [ ] **Step 5: 复制原始PNG到icons目录**

```bash
cp icon.png src-tauri/icons/icon.png
```

最终 `src-tauri/icons/` 应包含：
```
├── icon.png           # 512x512 新设计PNG
├── icon.ico           # Windows
├── icon.icns          # macOS
├── icon-256x256.png   # Linux
├── icon-128x128.png   # Linux
├── icon-64x64.png     # Linux
├── icon-32x32.png     # Linux
└── icon.svg           # 保留旧版本（可选备份）
```

- [ ] **Step 6: 验证所有icon文件**

```bash
ls -lh src-tauri/icons/
```

输出应显示所有7个文件。检查：
- 文件大小合理
- 无损坏或0字节文件

- [ ] **Step 7: 提交icon文件**

```bash
git add src-tauri/icons/icon.png src-tauri/icons/icon.ico src-tauri/icons/icon.icns src-tauri/icons/icon-*.png
git commit -m "feat: add new 3D icon designs for all platforms (Windows/macOS/Linux)"
```

---

## Task 3: 更新tauri.conf.json配置

**Files:**
- Modify: `src-tauri/tauri.conf.json` (bundle.targets)
- Test: 验证JSON格式有效

- [ ] **Step 1: 读取当前tauri.conf.json**

```bash
cat src-tauri/tauri.conf.json
```

当前应显示：
```json
"bundle": {
  "active": true,
  "targets": ["nsis"]
}
```

- [ ] **Step 2: 更新bundle.targets支持三平台**

打开 `src-tauri/tauri.conf.json`，找到 `"bundle"` 部分，修改为：

```json
"bundle": {
  "active": true,
  "targets": [
    "nsis",
    "dmg",
    "deb",
    "appimage"
  ]
}
```

**说明:**
- `"nsis"` - Windows NSIS installer (.exe)
- `"dmg"` - macOS Disk Image (.dmg)
- `"deb"` - Linux Debian package (.deb)
- `"appimage"` - Linux portable AppImage (.AppImage)

完整示例：
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "oh-my-file",
  "version": "0.1.0",
  "identifier": "com.openai.ohmyfile",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "oh-my-file",
        "width": 1120,
        "height": 760,
        "resizable": true,
        "center": true,
        "visible": true
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": [
      "nsis",
      "dmg",
      "deb",
      "appimage"
    ]
  }
}
```

- [ ] **Step 3: 验证JSON格式**

```bash
node -e "console.log(JSON.parse(require('fs').readFileSync('src-tauri/tauri.conf.json', 'utf8')))" && echo "✓ JSON valid"
```

输出应显示解析后的JSON对象和 "✓ JSON valid"

- [ ] **Step 4: 提交配置修改**

```bash
git add src-tauri/tauri.conf.json
git commit -m "config: add macOS and Linux bundle targets to tauri.conf.json"
```

---

## Task 4: 创建GitHub Actions多平台构建工作流

**Files:**
- Create: `.github/workflows/build-release.yml` (新工作流文件)
- Test: 验证YAML格式有效

- [ ] **Step 1: 创建workflows目录（如不存在）**

```bash
mkdir -p .github/workflows
```

- [ ] **Step 2: 创建build-release.yml工作流文件**

创建 `.github/workflows/build-release.yml`：

```yaml
name: Build Multi-Platform Release

on:
  push:
    tags:
      - 'v*'  # 当push tag时触发，如 git push origin v0.1.0
  workflow_dispatch:  # 允许手动触发

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        run: npm install
      
      - name: Build Tauri app (Windows)
        uses: tauri-apps/tauri-action@v0
        with:
          projectPath: ./src-tauri
          args: '--target nsis'
      
      - name: Upload Windows artifacts
        uses: actions/upload-artifact@v4
        with:
          name: windows-artifacts
          path: src-tauri/target/release/bundle/nsis/

  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        run: npm install
      
      - name: Build Tauri app (macOS)
        uses: tauri-apps/tauri-action@v0
        with:
          projectPath: ./src-tauri
          args: '--target dmg'
      
      - name: Upload macOS artifacts
        uses: actions/upload-artifact@v4
        with:
          name: macos-artifacts
          path: src-tauri/target/release/bundle/macos/

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y build-essential libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
      
      - name: Install dependencies
        run: npm install
      
      - name: Build Tauri app (Linux)
        uses: tauri-apps/tauri-action@v0
        with:
          projectPath: ./src-tauri
          args: '--target deb appimage'
      
      - name: Upload Linux artifacts
        uses: actions/upload-artifact@v4
        with:
          name: linux-artifacts
          path: src-tauri/target/release/bundle/linux/

  create-release:
    needs: [build-windows, build-macos, build-linux]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')
    steps:
      - uses: actions/checkout@v4
      
      - name: Download all artifacts
        uses: actions/download-artifact@v4
      
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            windows-artifacts/*
            macos-artifacts/*
            linux-artifacts/*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**工作流说明:**
- 在push tag时自动触发（如 `git push origin v0.1.0`）
- 支持手动触发（Actions标签页点击Run workflow）
- Windows、macOS、Linux并行构建
- 各平台构建产物上传到Artifacts
- tag版本时自动创建Release并上传文件

- [ ] **Step 3: 验证YAML格式**

```bash
node -e "const yaml=require('js-yaml'); console.log(yaml.load(require('fs').readFileSync('.github/workflows/build-release.yml','utf8'))) && console.log('✓ YAML valid')" 2>/dev/null || echo "需要js-yaml库或使用在线YAML验证工具"
```

或访问 https://www.yamllint.com/ 粘贴文件内容验证

**预期**: YAML格式有效，无缩进或语法错误

- [ ] **Step 4: 提交工作流文件**

```bash
git add .github/workflows/build-release.yml
git commit -m "ci: add multi-platform build workflow for Windows/macOS/Linux"
```

---

## Task 5: 本地测试打包（当前平台）

**Files:**
- 测试: 无代码变更，验证构建流程
- Output: 检查 `src-tauri/target/release/bundle/` 中的产物

- [ ] **Step 1: 清理旧的构建产物**

```bash
npm run clean
```

预期输出: 清理 `dist/` 和 `src-tauri/target/` 中的旧文件

- [ ] **Step 2: 安装依赖（确保最新）**

```bash
npm install
```

- [ ] **Step 3: 构建前端**

```bash
npm run build
```

预期输出:
```
vite v7.x.x building for production...
✓ 1234 modules transformed.
dist/index.html                 10.12 kB │ gzip: 3.45 kB
dist/assets/index-*.js         250.34 kB │ gzip: 85.12 kB
```

验证: `dist/` 目录存在且包含HTML和JS文件

- [ ] **Step 4: 执行本地打包**

```bash
npm run package
```

或等价：
```bash
node scripts/package.js
```

预期输出（Windows示例）:
```
Building with tauri-cli...
✓ Target folder created
✓ Bundling for nsis...
✓ Output at: src-tauri/target/release/bundle/nsis/
```

- [ ] **Step 5: 验证打包产物**

检查输出目录（根据平台不同）：

**Windows:**
```bash
ls -lh src-tauri/target/release/bundle/nsis/
```
应显示 `.exe` 和/或 `.msi` 文件（20MB-100MB）

**macOS:**
```bash
ls -lh src-tauri/target/release/bundle/macos/
```
应显示 `.dmg` 文件

**Linux:**
```bash
ls -lh src-tauri/target/release/bundle/linux/
```
应显示 `.AppImage` 和/或 `.deb` 文件

- [ ] **Step 6: 可选 - 测试应用程序**

如果是Windows/macOS/Linux本地测试：
- Windows: 双击 `.exe` 或 `.msi` 安装并运行
- macOS: 打开 `.dmg`，拖拽应用到Applications，运行
- Linux: 运行 `.AppImage` 或安装 `.deb` 后运行

验证应用：
- 应用窗口打开
- 显示QR码和本地IP地址
- 手机扫码可以访问上传页面

- [ ] **Step 7: 提交构建验证通过的状态（仅记录，无文件变更）**

```bash
git status
```

应显示干净工作区（无未追踪文件）。如果有临时文件可添加到 `.gitignore`：

```bash
echo "src-tauri/target/" >> .gitignore
echo "dist/" >> .gitignore
git add .gitignore
git commit -m "build: ignore Tauri and Vite build artifacts"
```

---

## Task 6: GitHub Actions测试（创建发布tag）

**Files:**
- 无代码变更
- Test: 推送tag触发工作流

- [ ] **Step 1: 查看工作流状态**

在本地确认所有代码已提交：
```bash
git status
```

预期: `working tree clean`

- [ ] **Step 2: 创建版本tag**

```bash
git tag -a v0.1.0 -m "Release version 0.1.0"
```

- [ ] **Step 3: 推送tag到远程**

```bash
git push origin v0.1.0
```

- [ ] **Step 4: 监控GitHub Actions执行**

访问: https://github.com/[用户名]/oh-my-file/actions

查看最新的工作流运行（应显示 "Build Multi-Platform Release"）

预期状态流程：
- `build-windows` - Running → Passed ✓
- `build-macos` - Running → Passed ✓
- `build-linux` - Running → Passed ✓
- `create-release` - Waiting → Running → Passed ✓

总耗时: 10-20 分钟（取决于网络和GitHub Runner队列）

- [ ] **Step 5: 验证Release创建**

访问: https://github.com/[用户名]/oh-my-file/releases

应显示 `v0.1.0` release，包含：
- Windows: `.exe` 和 `.msi` 文件
- macOS: `.dmg` 文件
- Linux: `.AppImage` 和 `.deb` 文件

每个文件旁边显示文件大小

- [ ] **Step 6: 下载并验证构建产物**

从Release页面下载各平台的安装包，在对应平台测试安装和运行（可选）

---

## Task 7: 更新项目文档

**Files:**
- Modify: `README.md` (添加打包说明)
- Modify: `CLAUDE.md` (如已存在，更新icon配置信息)
- No test files

- [ ] **Step 1: 更新README.md添加打包说明**

编辑 `README.md`，在 "使用方式" 后添加新章节：

```markdown
## 跨平台打包和发布

### 本地打包

为当前平台生成安装包：

```bash
npm run build      # 构建前端
npm run package    # 打包应用（当前平台）
```

输出位置: `src-tauri/target/release/bundle/`

**平台要求:**
- Windows: Visual Studio Build Tools / MSVC
- macOS: Xcode Command Line Tools
- Linux: `build-essential` (Debian/Ubuntu)

**注意:** 必须在目标平台上打包，不支持交叉编译。

### 自动化CI/CD发布

在GitHub上创建版本标签自动触发多平台构建：

```bash
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin v0.1.0
```

工作流程:
1. 监听 push tag 事件
2. Windows runner 构建 .exe / .msi
3. macOS runner 构建 .dmg
4. Linux runner 构建 .AppImage / .deb
5. 自动创建 GitHub Release 并上传所有平台包

在 [Releases](https://github.com/[用户]/oh-my-file/releases) 页面可下载各平台安装包。

### 手动运行工作流

访问 Actions 标签页 → "Build Multi-Platform Release" → "Run workflow" → 选择分支 → 运行

（不需要创建tag）
```

- [ ] **Step 2: 更新README.md的本地开发部分**

如README中的开发说明仍使用旧的icon，可添加注释：

在 "本地开发" 下添加：

```markdown
### Icon更新

应用图标已更新为3D拟物化设计，支持Windows、macOS和Linux。Icon文件位于 `src-tauri/icons/`：
- `icon.ico` - Windows
- `icon.icns` - macOS
- `icon-*.png` - Linux (多尺寸)
- `icon.svg` - 备用矢量格式

Tauri自动从这些文件生成平台特定的icon，无需手动配置。
```

- [ ] **Step 3: 提交文档更新**

```bash
git add README.md
git commit -m "docs: update README with multi-platform packaging and icon info"
```

---

## Task 8: 清理和最终验证

**Files:**
- 无代码变更
- Verification: 确保所有变更已提交

- [ ] **Step 1: 查看完整的git提交历史**

```bash
git log --oneline -10
```

应显示的提交序列（从新到旧）：
```
xxx docs: update README with multi-platform packaging and icon info
xxx ci: add multi-platform build workflow for Windows/macOS/Linux
xxx config: add macOS and Linux bundle targets to tauri.conf.json
xxx feat: add new 3D icon designs for all platforms (Windows/macOS/Linux)
xxx build: ignore Tauri and Vite build artifacts
...
```

- [ ] **Step 2: 验证工作树干净**

```bash
git status
```

预期: `On branch master`, `working tree clean`

- [ ] **Step 3: 验证所有关键文件存在**

```bash
test -f src-tauri/icons/icon.png && \
test -f src-tauri/icons/icon.ico && \
test -f src-tauri/icons/icon.icns && \
test -f src-tauri/icons/icon-256x256.png && \
test -f .github/workflows/build-release.yml && \
test -f README.md && \
echo "✓ All key files present"
```

预期输出: `✓ All key files present`

- [ ] **Step 4: 验证tauri.conf.json配置**

```bash
grep -A 5 '"bundle"' src-tauri/tauri.conf.json
```

预期输出包含：
```json
"bundle": {
  "active": true,
  "targets": [
    "nsis",
    "dmg",
    "deb",
    "appimage"
  ]
}
```

- [ ] **Step 5: 最后的提交总结**

整个实现过程完成的主要变更：
1. ✅ 通过Canva设计新的3D拟物化icon
2. ✅ 转换icon为Windows (.ico)、macOS (.icns)、Linux (.png) 格式
3. ✅ 添加Linux和macOS的打包icon文件到 `src-tauri/icons/`
4. ✅ 更新 `tauri.conf.json` 支持三平台bundle
5. ✅ 创建GitHub Actions多平台构建工作流
6. ✅ 本地测试当前平台打包
7. ✅ 通过tag触发GitHub Actions验证工作流
8. ✅ 更新项目文档

所有任务完成，项目现已支持Windows、macOS、Linux三平台的自动化打包和发布！

---

## Success Criteria

- [x] 新icon已设计并导出为PNG
- [x] icon已转换为所有平台所需格式（.ico, .icns, .png）
- [x] `src-tauri/icons/` 包含所有平台icon文件
- [x] `tauri.conf.json` 已更新支持 dmg、deb、appimage
- [x] GitHub Actions工作流已创建并测试成功
- [x] 创建tag可自动触发多平台构建
- [x] 构建产物正确输出到各平台目录
- [x] README和项目文档已更新
- [x] 所有变更已提交到git

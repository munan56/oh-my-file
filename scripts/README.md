# oh-my-file Build Scripts

跨平台的编辑和打包脚本套件。支持 Windows、macOS 和 Linux 平台。

## 快速开始

### 开发模式

启动开发服务器，支持热更新：

```bash
# Linux/macOS
./scripts/dev.sh

# Windows
scripts\dev.bat

# 或使用 npm
npm run dev
```

### 打包应用

为当前平台生成安装包：

```bash
# Linux/macOS
node scripts/package.js

# Windows
node scripts/package.js

# 或使用 npm
npm run package
```

## 详细用法

### package.js - 多平台打包脚本

生成 Windows (.msi/.nsis)、macOS (.dmg) 和 Linux (.appimage/.deb) 的安装包。

```bash
# 为当前平台打包
node scripts/package.js

# 指定平台
node scripts/package.js win32      # Windows
node scripts/package.js darwin     # macOS
node scripts/package.js linux      # Linux

# 跳过前端构建（如果已构建）
node scripts/package.js --no-build

# 显示帮助
node scripts/package.js --help
```

#### 输出位置

打包结果位于：`src-tauri/target/release/bundle/`

- **Windows**: `nsis/` 目录
- **macOS**: `macos/` 目录
- **Linux**: `linux/` 目录

### build.sh / build.bat - 平台特定构建脚本

便捷脚本，适合特定平台使用。

```bash
# Linux/macOS - 构建当前平台
./scripts/build.sh

# Windows - 构建 Windows 版本
scripts\build.bat
```

### dev.sh / dev.bat - 开发服务器

启动开发环境，实时更新前端代码。

```bash
# Linux/macOS
./scripts/dev.sh

# Windows
scripts\dev.bat
```

### clean.sh / clean.bat - 清理构建

删除构建产物和缓存。

```bash
# Linux/macOS
./scripts/clean.sh

# Windows
scripts\clean.bat

# 或使用 npm
npm run clean
```

## npm 脚本

在 `package.json` 中已配置以下命令：

```json
{
  "scripts": {
    "dev": "node scripts/dev.js",
    "build": "tsc && vite build",
    "package": "node scripts/package.js",
    "clean": "node scripts/clean.js",
    "preview": "vite preview",
    "tauri": "tauri"
  }
}
```

使用：

```bash
npm run dev       # 开发模式
npm run build     # 构建前端
npm run package   # 打包应用
npm run clean     # 清理构建
```

## 跨平台构建注意事项

### Windows 上构建 macOS/Linux

不支持交叉编译。需要在目标平台上运行脚本：

```bash
# ✓ 正确做法
# 在 macOS 上
npm run package darwin

# 在 Linux 上
npm run package linux

# ✗ 不支持
# 在 Windows 上运行
npm run package darwin   # ❌ 会失败
npm run package linux    # ❌ 会失败
```

### 平台要求

- **Windows**: Visual Studio Build Tools 或 MSVC
- **macOS**: Xcode Command Line Tools
- **Linux**: Build essentials (`build-essential` on Debian/Ubuntu)

## 文件结构

```
scripts/
├── package.js      # 主打包脚本（Node.js）
├── build.sh        # Linux/macOS 便捷构建脚本
├── build.bat       # Windows 便捷构建脚本
├── dev.sh          # Linux/macOS 开发脚本
├── dev.bat         # Windows 开发脚本
├── clean.sh        # Linux/macOS 清理脚本
├── clean.bat       # Windows 清理脚本
└── README.md       # 本文档
```

## 生成的安装包

### Windows
- `.msi` - Windows Installer（推荐用于分发）
- `.exe` - NSIS 可执行程序

### macOS
- `.dmg` - Disk Image（推荐用于分发）
- `.app` - Application Bundle

### Linux
- `.AppImage` - 便携式可执行文件
- `.deb` - Debian 包

## 故障排除

### 权限错误

```bash
# Linux/macOS 需要执行权限
chmod +x scripts/*.sh
```

### 构建失败

1. 确保依赖已安装：
   ```bash
   npm install
   ```

2. 清理旧的构建：
   ```bash
   npm run clean
   ```

3. 检查平台兼容性：
   ```bash
   node scripts/package.js --help
   ```

## 自动化 CI/CD

在 GitHub Actions 中使用：

```yaml
- name: Build Windows
  run: node scripts/package.js win32

- name: Build macOS
  run: node scripts/package.js darwin

- name: Build Linux
  run: node scripts/package.js linux
```

## 许可证

与 oh-my-file 项目相同

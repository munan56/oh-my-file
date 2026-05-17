# GitHub Actions 工作流使用指南

本项目配置了两个 GitHub Actions 工作流，用于自动化构建和发布。

## 工作流说明

### 1. Release 工作流 (`release.yml`)

**触发条件：** 推送 tag（`v*` 格式）

**功能：**
- 自动为 Windows、macOS、Linux 各平台构建应用
- 生成对应的安装包
- 自动创建 GitHub Release
- 上传所有安装包到 Release

**生成的文件：**
- **Windows**: `.msi` 和 `.exe`
- **macOS**: `.dmg` 和 `.app.tar.gz`
- **Linux**: `.AppImage` 和 `.deb`

### 2. Build 工作流 (`build.yml`)

**触发条件：** 推送到主分支或创建 PR

**功能：**
- 在 Windows、macOS、Linux 上检查构建
- 验证代码能否成功编译
- 缓存依赖加快构建速度

## 使用方法

### 发布新版本

1. **更新版本号**
   ```bash
   # 编辑 package.json 和 src-tauri/tauri.conf.json 中的 version
   ```

2. **创建 Git 标签**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. **等待自动构建**
   - 访问 GitHub Actions 标签页查看构建进度
   - 构建完成后，自动在 Releases 页面显示所有安装包

4. **下载和测试**
   - 从 GitHub Releases 页面下载对应平台的安装包
   - 测试无误后可以发布公开发行版

### 监控构建状态

访问项目的 **Actions** 标签页查看：
- 所有工作流的运行历史
- 实时的构建进度
- 构建成功/失败的详细日志

## 构建时间估计

首次构建可能需要较长时间（因为需要下载和编译 Rust 依赖）：

- **Windows**: 10-15 分钟
- **macOS**: 15-20 分钟
- **Linux**: 10-15 分钟

后续构建会更快（利用缓存）：

- 各平台: 5-10 分钟

## 配置说明

### Release 工作流的文件输出

```
src-tauri/target/release/bundle/
├── nsis/          # Windows
│   ├── *.msi
│   └── *.exe
├── macos/         # macOS
│   ├── *.dmg
│   └── *.app
└── linux/         # Linux
    ├── *.AppImage
    └── *.deb
```

### 自动上传到 Release

工作流会自动将这些文件上传到 GitHub Release：

```
Release v0.2.0
├── oh-my-file_0.2.0_x64.msi        (Windows Installer)
├── oh-my-file_0.2.0_x64-setup.exe  (Windows Portable)
├── oh-my-file_0.2.0.dmg            (macOS Disk Image)
├── app.tar.gz                       (macOS App Bundle)
├── oh-my-file_0.2.0_amd64.AppImage (Linux Portable)
└── oh-my-file_0.2.0_amd64.deb      (Linux Debian)
```

## 常见问题

### Q: 为什么构建失败？

**检查清单：**
1. 确保代码能在本地成功编译
2. 检查 GitHub Actions 日志中的具体错误
3. 确保所有依赖已安装（`npm install`）
4. 检查 Rust 工具链是否最新

### Q: 如何跳过 Release 工作流？

如果不想发布但需要推送代码：

```bash
# 只推送分支代码，不推送 tag
git push origin main
```

### Q: 如何删除已发布的 Release？

```bash
# 删除本地 tag
git tag -d v0.2.0

# 删除远程 tag
git push origin :refs/tags/v0.2.0

# 然后在 GitHub 网页上删除对应的 Release
```

### Q: 构建缓存何时清除？

GitHub 会在 7 天内自动清除未使用的缓存。如需手动清除：
- 访问项目设置 → Actions → Runners → 清除所有缓存

## 环境变量和密钥

当前工作流使用的环境变量：

- `GITHUB_TOKEN`: 自动提供，用于上传到 Release
- `CI=true`: 标记为 CI 环境

## 优化建议

### 1. 加速构建

如果构建时间过长，可以：
- 只为特定平台构建（注释掉其他 job）
- 增加缓存命中率

### 2. 通知集成

可以集成 Slack 或其他通知服务来获得构建完成提醒。

### 3. 代码签名

对于生产版本，应该添加代码签名（特别是 Windows 和 macOS）。

## 工作流文件位置

```
.github/workflows/
├── release.yml   # 发布工作流
└── build.yml     # 检查构建工作流
```

## 相关资源

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri 发布指南](https://tauri.app/v1/guides/distribution/sign-your-app/)
- [actions/upload-release-asset](https://github.com/actions/upload-release-asset)

# oh-my-file

`oh-my-file` 是一个局域网单向文件传输工具。电脑端运行桌面应用，手机端扫码后在浏览器里打开上传页，把文件直接发到电脑。

## 项目目标

- 电脑端运行 Tauri 应用，展示二维码、连接地址和实时传输列表
- 手机端浏览器零安装上传文件
- 仅支持“手机传电脑”
- 仅面向同一局域网环境

## 本地开发

1. 安装 Node.js 与 Rust 工具链
2. 在项目根目录执行 `npm install`
3. 启动前端开发服务器：`npm run dev`
4. 启动桌面应用开发模式：`npm run tauri dev`

## 使用方式

1. 打开桌面应用
2. 用手机扫描左侧二维码
3. 首次上传时，在电脑端选择保存目录
4. 在手机浏览器里选择文件
5. 等待桌面端进度条完成并显示保存路径

## 保存目录行为

- 应用本次运行期间只会在首次传输时询问一次保存目录
- 后续上传会复用同一个目录
- 如果选择了已存在同名文件，程序会自动保存为 `name(1).ext`

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

在 [Releases](../../releases) 页面可下载各平台安装包。

### Icon信息

应用图标已更新为3D拟物化设计，支持Windows、macOS和Linux。Icon文件位于 `src-tauri/icons/`：
- `icon.ico` - Windows
- `icon.icns` - macOS
- `icon-*.png` - Linux (多尺寸)

Tauri自动从这些文件生成平台特定的icon，无需手动配置。

## 已知限制

- 当前版本不支持电脑向手机传输
- 当前版本不支持公网访问或账号鉴权
- 当前版本只保存内存态会话和任务记录，重启应用后不会保留历史
- 跨平台打包需要在目标平台上执行，不支持交叉编译

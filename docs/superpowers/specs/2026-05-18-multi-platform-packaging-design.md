# 多平台打包和Icon设计方案

**日期:** 2026-05-18  
**状态:** 设计中  
**目标:** 支持Windows、macOS和Linux三个平台的应用打包，提升应用icon视觉质感

## 问题背景

当前项目仅支持Windows打包（nsis），缺少macOS和Linux的打包配置。Icon设计过于简陋，不符合现代应用审美。需要：
1. 设计新的3D拟物化icon
2. 为三个平台生成对应的icon格式
3. 搭建GitHub Actions工作流实现自动化打包

## 设计概览

### 阶段1：Icon设计与资源准备

**Step 1.1 - Canva设计**
- 在Canva上创建512×512px的3D拟物化icon设计
- 风格要求：现代、专业、与"文件传输"应用定位相符
- 导出为PNG（高分辨率，无背景或透明背景）

**Step 1.2 - 格式转换**
为三个平台生成icon：
- **Windows**: 使用 `icon-gen` 或在线工具将PNG转换为 `.ico`（需支持256×256、128×128、64×64、32×32、16×16多尺寸）
- **macOS**: 使用 `png2icns` 或在线工具将PNG转换为 `.icns` 格式
- **Linux**: 保留PNG格式（通常为256×256、128×128、64×64、32×32）

**产物:**
```
src-tauri/icons/
├── icon.svg          # 源SVG（保留）
├── icon.png          # 新设计的PNG（512x512）
├── icon.ico          # Windows icon（多尺寸）
├── icon.icns         # macOS icon
├── icon-32x32.png    # Linux (32x32)
├── icon-64x64.png    # Linux (64x64)
├── icon-128x128.png  # Linux (128x128)
└── icon-256x256.png  # Linux (256x256)
```

### 阶段2：Tauri配置更新

**Step 2.1 - 更新 tauri.conf.json**

当前配置仅包含 Windows：
```json
"bundle": {
  "active": true,
  "targets": ["nsis"]
}
```

需更新为：
```json
"bundle": {
  "active": true,
  "targets": [
    "nsis",      // Windows
    "dmg",       // macOS
    "deb",       // Linux Debian/Ubuntu
    "appimage"   // Linux通用
  ]
}
```

**Step 2.2 - Icon配置**

Tauri会自动查找 `src-tauri/icons/` 目录中的icon文件。无需额外配置，只需确保文件名和格式正确。

### 阶段3：GitHub Actions工作流

**Step 3.1 - 创建打包工作流文件**

路径: `.github/workflows/build-release.yml`

工作流特点：
- 三个并行或顺序的job：build-windows、build-macos、build-linux
- 每个job使用对应平台的runner（windows-latest、macos-latest、ubuntu-latest）
- 使用 `tauri-action` GitHub Action简化构建流程
- 构建产物上传到release或artifact

**工作流架构:**
```
Trigger (push tag / manual dispatch)
  ├─ build-windows (runs-on: windows-latest)
  │  ├─ checkout
  │  ├─ setup node
  │  ├─ npm install
  │  ├─ tauri build --target nsis
  │  └─ upload artifacts
  │
  ├─ build-macos (runs-on: macos-latest)
  │  ├─ checkout
  │  ├─ setup node
  │  ├─ npm install
  │  ├─ tauri build --target dmg
  │  └─ upload artifacts
  │
  └─ build-linux (runs-on: ubuntu-latest)
     ├─ checkout
     ├─ setup node / build-essential
     ├─ npm install
     ├─ tauri build --target deb appimage
     └─ upload artifacts
```

**Step 3.2 - 代码签名处理**

当前阶段不需要签名。所有平台构建跳过签名验证（Tauri默认行为）。

后续如需签名，需：
- Windows: 添加代码签名证书配置
- macOS: 需要Apple开发者账号和证书
- Linux: 通常不需要签名（用户信任级别由分发渠道决定）

## 数据流

```
Canva PNG → 格式转换 → icon文件集 → tauri.conf.json → GitHub Actions → 多平台exe/dmg/deb/AppImage
```

## 关键文件变更

| 文件 | 变更 | 说明 |
|------|------|------|
| `src-tauri/icons/` | 新增 | icon.icns、icon.png、各尺寸PNG |
| `src-tauri/tauri.conf.json` | 修改 | bundle.targets添加macOS和Linux |
| `.github/workflows/build-release.yml` | 新增 | 多平台构建工作流 |

## 成功标准

1. ✅ Canva设计完成，icon已导出为PNG
2. ✅ 所有平台icon文件已生成并放入正确目录
3. ✅ `tauri.conf.json` 已更新，配置三个平台目标
4. ✅ GitHub Actions工作流能够成功构建三个平台的应用包
5. ✅ 本地测试：Windows .exe、macOS .dmg、Linux .deb 或 .AppImage 都能生成

## 实现顺序

1. 在Canva设计icon（用户手动完成）
2. 使用工具生成各平台icon格式
3. 更新tauri.conf.json
4. 创建GitHub Actions工作流
5. 测试本地构建
6. 测试GitHub Actions自动构建

## 后续考虑事项

- **代码签名**: 如需发布到官方应用商店或保证用户信任，需添加签名
- **持续发布**: 配置automatic release，将构建产物自动发布到GitHub Releases
- **版本管理**: 使用git tags触发构建（如 `v0.1.0`）

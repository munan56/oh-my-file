# Phase 3: 桌面前端与手机上传页

**目标：** 实现桌面端 React UI、二维码展示、传输列表状态同步，以及手机端上传页面交互。

---

### Task 1: 创建前端数据类型与状态 Hook

- [ ] **创建 `src/types/transfer.ts`**

```ts
export type TransferStatus =
  | 'pending'
  | 'waiting_directory'
  | 'uploading'
  | 'completed'
  | 'cancelled'
  | 'error';

export interface TransferItemState {
  id: string;
  filename: string;
  deviceName: string;
  totalBytes: number;
  receivedBytes: number;
  status: TransferStatus;
  savePath?: string;
  errorMessage?: string;
}

export interface ServerInfo {
  status: 'starting' | 'running' | 'error';
  port: number;
  token: string;
  ip: string;
  url: string;
  qrCodeDataUrl: string;
  connectedDevices: number;
}
```

- [ ] **创建 `src/hooks/useTransfers.ts`**

```ts
import { useEffect, useMemo, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { ServerInfo, TransferItemState } from '../types/transfer';

type TransferEvent =
  | { type: 'server-ready'; payload: ServerInfo }
  | {
      type: 'transfer-progress';
      payload: {
        id: string;
        filename: string;
        deviceName: string;
        totalBytes: number;
        receivedBytes: number;
        status: TransferItemState['status'];
      };
    }
  | {
      type: 'transfer-complete';
      payload: {
        id: string;
        savePath: string;
      };
    }
  | {
      type: 'transfer-error';
      payload: {
        id: string;
        errorMessage: string;
      };
    }
  | {
      type: 'devices-changed';
      payload: {
        connectedDevices: number;
      };
    };

export function useTransfers() {
  const [serverInfo, setServerInfo] = useState<ServerInfo | null>(null);
  const [transfers, setTransfers] = useState<TransferItemState[]>([]);

  useEffect(() => {
    let disposed = false;

    const bootstrap = async () => {
      const info = await invoke<ServerInfo>('get_server_info');
      if (!disposed) {
        setServerInfo(info);
      }
    };

    const unlistenPromise = listen<TransferEvent>('transfer-event', (event) => {
      const { type, payload } = event.payload;

      if (type === 'server-ready') {
        setServerInfo(payload);
        return;
      }

      if (type === 'devices-changed') {
        setServerInfo((current) =>
          current ? { ...current, connectedDevices: payload.connectedDevices } : current,
        );
        return;
      }

      if (type === 'transfer-progress') {
        setTransfers((current) => {
          const next = [...current];
          const index = next.findIndex((item) => item.id === payload.id);
          const item: TransferItemState = {
            id: payload.id,
            filename: payload.filename,
            deviceName: payload.deviceName,
            totalBytes: payload.totalBytes,
            receivedBytes: payload.receivedBytes,
            status: payload.status,
          };

          if (index === -1) {
            next.unshift(item);
          } else {
            next[index] = { ...next[index], ...item };
          }

          return next;
        });
      }

      if (type === 'transfer-complete') {
        setTransfers((current) =>
          current.map((item) =>
            item.id === payload.id
              ? { ...item, status: 'completed', savePath: payload.savePath }
              : item,
          ),
        );
      }

      if (type === 'transfer-error') {
        setTransfers((current) =>
          current.map((item) =>
            item.id === payload.id
              ? { ...item, status: 'error', errorMessage: payload.errorMessage }
              : item,
          ),
        );
      }
    });

    bootstrap();

    return () => {
      disposed = true;
      void unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const activeCount = useMemo(
    () => transfers.filter((item) => item.status === 'uploading').length,
    [transfers],
  );

  return {
    serverInfo,
    transfers,
    activeCount,
  };
}
```

- [ ] **验证前端类型检查**

```bash
npm run build
```

预期：TypeScript 编译通过。

---

### Task 2: 实现二维码与状态栏组件

- [ ] **创建 `src/components/QRCode.tsx`**

```tsx
import type { ServerInfo } from '../types/transfer';

interface QRCodeProps {
  serverInfo: ServerInfo | null;
}

export function QRCode({ serverInfo }: QRCodeProps) {
  if (!serverInfo) {
    return (
      <section className="qr-card">
        <h2>连接中</h2>
        <p>正在启动本地传输服务...</p>
      </section>
    );
  }

  return (
    <section className="qr-card">
      <h2>扫码上传</h2>
      <div className="qr-image-wrap">
        <img src={serverInfo.qrCodeDataUrl} alt="上传二维码" className="qr-image" />
      </div>
      <p className="server-url">{serverInfo.url}</p>
      <p className="server-meta">
        局域网地址 {serverInfo.ip}:{serverInfo.port}
      </p>
    </section>
  );
}
```

- [ ] **创建 `src/components/StatusBar.tsx`**

```tsx
interface StatusBarProps {
  status: 'starting' | 'running' | 'error';
  connectedDevices: number;
  activeTransfers: number;
}

export function StatusBar({
  status,
  connectedDevices,
  activeTransfers,
}: StatusBarProps) {
  const label =
    status === 'running' ? '运行中' : status === 'starting' ? '启动中' : '异常';

  return (
    <footer className="status-bar">
      <span>状态: {label}</span>
      <span>已连接: {connectedDevices} 台设备</span>
      <span>进行中: {activeTransfers} 个任务</span>
    </footer>
  );
}
```

- [ ] **在 `src/styles/global.css` 添加对应样式**

```css
.qr-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  border-radius: 20px;
  background: linear-gradient(180deg, #f5f9ff 0%, #eef4ff 100%);
  box-shadow: 0 16px 40px rgba(40, 72, 130, 0.12);
}

.qr-image-wrap {
  display: grid;
  place-items: center;
  padding: 16px;
  border-radius: 16px;
  background: #fff;
}

.qr-image {
  width: 220px;
  height: 220px;
  object-fit: contain;
}

.server-url {
  font-size: 14px;
  word-break: break-all;
}

.server-meta,
.status-bar {
  color: #51607a;
  font-size: 13px;
}
```

---

### Task 3: 实现传输列表组件

- [ ] **创建 `src/components/TransferItem.tsx`**

```tsx
import type { TransferItemState } from '../types/transfer';

interface TransferItemProps {
  item: TransferItemState;
  onCancel: (id: string) => void;
}

const formatPercent = (received: number, total: number) =>
  total > 0 ? Math.floor((received / total) * 100) : 0;

export function TransferItem({ item, onCancel }: TransferItemProps) {
  const percent = formatPercent(item.receivedBytes, item.totalBytes);

  return (
    <article className="transfer-item">
      <div className="transfer-item__header">
        <div>
          <h3>{item.filename}</h3>
          <p>{item.deviceName}</p>
        </div>
        <strong>{percent}%</strong>
      </div>

      <div className="transfer-progress">
        <div
          className="transfer-progress__value"
          style={{ width: `${percent}%` }}
        />
      </div>

      <div className="transfer-item__footer">
        <span>{item.status}</span>
        {item.status === 'uploading' ? (
          <button onClick={() => onCancel(item.id)}>取消</button>
        ) : null}
        {item.status === 'completed' && item.savePath ? (
          <span>{item.savePath}</span>
        ) : null}
        {item.status === 'error' && item.errorMessage ? (
          <span>{item.errorMessage}</span>
        ) : null}
      </div>
    </article>
  );
}
```

- [ ] **创建 `src/components/TransferList.tsx`**

```tsx
import type { TransferItemState } from '../types/transfer';
import { TransferItem } from './TransferItem';

interface TransferListProps {
  items: TransferItemState[];
  onCancel: (id: string) => void;
}

export function TransferList({ items, onCancel }: TransferListProps) {
  if (items.length === 0) {
    return (
      <section className="transfer-list empty">
        <h2>文件传输列表</h2>
        <p>等待手机端开始上传...</p>
      </section>
    );
  }

  return (
    <section className="transfer-list">
      <h2>文件传输列表</h2>
      {items.map((item) => (
        <TransferItem key={item.id} item={item} onCancel={onCancel} />
      ))}
    </section>
  );
}
```

- [ ] **补充 `src/styles/global.css` 的列表样式**

```css
.transfer-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 420px;
}

.transfer-item {
  padding: 16px;
  border-radius: 18px;
  background: #fff;
  box-shadow: 0 12px 30px rgba(21, 38, 69, 0.08);
}

.transfer-progress {
  overflow: hidden;
  height: 10px;
  margin: 12px 0;
  border-radius: 999px;
  background: #e6ebf2;
}

.transfer-progress__value {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #1e90ff, #3fc6a8);
}
```

---

### Task 4: 组装 App 主界面

- [ ] **编辑 `src/App.tsx`**

```tsx
import { invoke } from '@tauri-apps/api/core';
import { QRCode } from './components/QRCode';
import { StatusBar } from './components/StatusBar';
import { TransferList } from './components/TransferList';
import { useTransfers } from './hooks/useTransfers';
import './styles/global.css';

export default function App() {
  const { serverInfo, transfers, activeCount } = useTransfers();

  const handleCancel = async (id: string) => {
    await invoke('cancel_transfer', { transferId: id });
  };

  return (
    <main className="app-shell">
      <section className="app-sidebar">
        <QRCode serverInfo={serverInfo} />
      </section>

      <section className="app-main">
        <TransferList items={transfers} onCancel={handleCancel} />
      </section>

      <StatusBar
        status={serverInfo?.status ?? 'starting'}
        connectedDevices={serverInfo?.connectedDevices ?? 0}
        activeTransfers={activeCount}
      />
    </main>
  );
}
```

- [ ] **补充布局样式**

```css
body {
  margin: 0;
  background:
    radial-gradient(circle at top left, rgba(78, 143, 255, 0.16), transparent 32%),
    linear-gradient(180deg, #f7faff 0%, #eef3f9 100%);
  color: #192435;
  font-family: 'Segoe UI', 'PingFang SC', sans-serif;
}

.app-shell {
  display: grid;
  grid-template-columns: 320px 1fr;
  grid-template-rows: 1fr auto;
  gap: 20px;
  min-height: 100vh;
  padding: 24px;
}

.app-sidebar,
.app-main {
  min-height: 0;
}

.status-bar {
  grid-column: 1 / -1;
  display: flex;
  justify-content: space-between;
  padding: 16px 20px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(12px);
}
```

- [ ] **验证桌面 UI 可运行**

```bash
npm run tauri dev
```

预期：左侧显示二维码卡片，右侧显示文件列表，底部状态栏展示服务状态。

---

### Task 5: 实现手机端上传页面

- [ ] **编辑 `phone-www/index.html`**

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta
      name="viewport"
      content="width=device-width, initial-scale=1.0, viewport-fit=cover"
    />
    <title>oh-my-file 传输</title>
    <link rel="stylesheet" href="style.css" />
  </head>
  <body>
    <main class="mobile-shell">
      <header class="hero">
        <p class="eyebrow">oh-my-file</p>
        <h1>把文件直接发到这台电脑</h1>
        <p id="server-name" class="hero-copy">已连接到局域网传输服务</p>
      </header>

      <label class="picker-button">
        <input id="file-input" type="file" multiple hidden />
        选择文件
      </label>

      <section id="upload-list" class="upload-list"></section>
    </main>
    <script src="app.js"></script>
  </body>
</html>
```

- [ ] **编辑 `phone-www/style.css`**

```css
body {
  margin: 0;
  min-height: 100vh;
  background:
    radial-gradient(circle at top, rgba(0, 194, 255, 0.18), transparent 28%),
    #09111f;
  color: #f5f8ff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.mobile-shell {
  max-width: 560px;
  margin: 0 auto;
  padding: 24px 18px 40px;
}

.picker-button {
  display: inline-flex;
  justify-content: center;
  width: 100%;
  padding: 18px 20px;
  border-radius: 18px;
  background: linear-gradient(135deg, #19a1ff, #5ae0b1);
  color: #04101d;
  font-size: 18px;
  font-weight: 700;
}

.upload-list {
  display: grid;
  gap: 14px;
  margin-top: 24px;
}
```

- [ ] **编辑 `phone-www/app.js`**

```javascript
const fileInput = document.getElementById('file-input');
const uploadList = document.getElementById('upload-list');

const token = new URLSearchParams(window.location.search).get('token');

function createUploadItem(file) {
  const root = document.createElement('article');
  root.className = 'upload-item';
  root.innerHTML = `
    <div class="upload-item__row">
      <strong>${file.name}</strong>
      <span data-role="percent">0%</span>
    </div>
    <div class="upload-progress"><div data-role="bar"></div></div>
    <div class="upload-item__row">
      <span data-role="status">等待上传</span>
      <button type="button" data-role="cancel">取消</button>
    </div>
  `;
  uploadList.prepend(root);
  return root;
}

async function uploadFile(file) {
  const item = createUploadItem(file);
  const percent = item.querySelector('[data-role="percent"]');
  const bar = item.querySelector('[data-role="bar"]');
  const status = item.querySelector('[data-role="status"]');
  const cancelButton = item.querySelector('[data-role="cancel"]');

  const xhr = new XMLHttpRequest();
  const formData = new FormData();
  formData.append('file', file);
  formData.append('device', navigator.userAgent);

  xhr.open('POST', `/api/upload?token=${encodeURIComponent(token || '')}`);

  xhr.upload.onprogress = (event) => {
    if (!event.lengthComputable) return;
    const value = Math.floor((event.loaded / event.total) * 100);
    percent.textContent = `${value}%`;
    bar.style.width = `${value}%`;
    status.textContent = '上传中';
  };

  xhr.onload = () => {
    if (xhr.status >= 200 && xhr.status < 300) {
      percent.textContent = '100%';
      bar.style.width = '100%';
      status.textContent = '上传完成';
      cancelButton.remove();
    } else {
      status.textContent = '上传失败';
    }
  };

  xhr.onerror = () => {
    status.textContent = '网络错误';
  };

  cancelButton.addEventListener('click', () => {
    xhr.abort();
    status.textContent = '已取消';
  });

  xhr.send(formData);
}

fileInput.addEventListener('change', () => {
  const files = Array.from(fileInput.files || []);
  files.forEach(uploadFile);
  fileInput.value = '';
});
```

---

### 验证清单

- [ ] `npm run build` 前端构建成功
- [ ] `npm run tauri dev` 可看到二维码、传输列表、状态栏
- [ ] 手机访问上传页可选择多个文件
- [ ] 上传进度可在手机端实时变化

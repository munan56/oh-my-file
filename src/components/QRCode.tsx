import type { ServerInfo } from '../types/transfer';

interface QRCodeProps {
  serverInfo: ServerInfo | null;
  onRefresh: () => Promise<void>;
  onSelectInterface: (interfaceId: string) => Promise<void>;
}

export function QRCode({
  serverInfo,
  onRefresh,
  onSelectInterface,
}: QRCodeProps) {
  if (!serverInfo) {
    return (
      <section className="qr-card">
        <div className="eyebrow">oh-my-file</div>
        <h1>启动中</h1>
        <p className="muted">正在初始化本地传输服务...</p>
      </section>
    );
  }

  return (
    <section className="qr-card">
      <div className="eyebrow">oh-my-file</div>
      <h1>扫码上传到电脑</h1>
      <div className="network-toolbar">
        <select
          className="network-select"
          value={serverInfo.selectedInterfaceId ?? ''}
          onChange={(event) => void onSelectInterface(event.target.value)}
        >
          {serverInfo.interfaces.map((item) => (
            <option key={item.id} value={item.id}>
              {item.name} · {item.ip}
              {item.isDefault ? '（自动推荐）' : ''}
            </option>
          ))}
        </select>
        <button className="ghost-button" onClick={() => void onRefresh()}>
          刷新地址
        </button>
      </div>
      <div className="qr-image-wrap">
        <img className="qr-image" src={serverInfo.qrCodeDataUrl} alt="上传二维码" />
      </div>
      <p className="server-url">{serverInfo.url}</p>
      <p className="muted">
        局域网地址 {serverInfo.ip}:{serverInfo.port}
      </p>
    </section>
  );
}

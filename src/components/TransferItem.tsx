import type { TransferItemState } from '../types/transfer';

interface TransferItemProps {
  item: TransferItemState;
  onCancel: (id: string) => void;
  onOpenDirectory: (path: string) => void;
}

function formatPercent(received: number, total: number) {
  if (total <= 0) {
    return 0;
  }

  return Math.min(100, Math.round((received / total) * 100));
}

function formatSize(bytes: number) {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }

  return `${bytes} B`;
}

function statusLabel(status: TransferItemState['status']) {
  switch (status) {
    case 'waiting_directory':
      return '等待选择目录';
    case 'uploading':
      return '上传中';
    case 'completed':
      return '已完成';
    case 'cancelled':
      return '已取消';
    case 'error':
      return '失败';
    default:
      return '等待中';
  }
}

export function TransferItem({
  item,
  onCancel,
  onOpenDirectory,
}: TransferItemProps) {
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
        <div className="transfer-progress__value" style={{ width: `${percent}%` }} />
      </div>

      <div className="transfer-item__footer">
        <span>
          {statusLabel(item.status)} · {formatSize(item.receivedBytes)} /{' '}
          {formatSize(item.totalBytes)}
        </span>
        {item.status === 'uploading' ? (
          <button className="ghost-button" onClick={() => onCancel(item.id)}>
            取消
          </button>
        ) : null}
        {item.status === 'completed' && item.directoryPath ? (
          <button
            className="ghost-button"
            onClick={() => onOpenDirectory(item.directoryPath!)}
          >
            打开目录
          </button>
        ) : null}
      </div>

      {item.savePath ? <p className="transfer-meta">已保存到 {item.savePath}</p> : null}
      {item.errorMessage ? <p className="transfer-error">{item.errorMessage}</p> : null}
    </article>
  );
}

import type { TransferItemState } from '../types/transfer';
import { TransferItem } from './TransferItem';

interface TransferListProps {
  items: TransferItemState[];
  onCancel: (id: string) => void;
  onOpenDirectory: (path: string) => void;
}

export function TransferList({
  items,
  onCancel,
  onOpenDirectory,
}: TransferListProps) {
  return (
    <section className="transfer-panel">
      <div className="panel-header">
        <div>
          <div className="eyebrow">Transfer</div>
          <h2>文件传输列表</h2>
        </div>
      </div>

      {items.length === 0 ? (
        <div className="empty-state">
          <p>等待手机端开始上传。</p>
          <p className="muted">扫码打开页面后，选择文件就会自动出现在这里。</p>
        </div>
      ) : (
        <div className="transfer-list">
          {items.map((item) => (
            <TransferItem
              key={item.id}
              item={item}
              onCancel={onCancel}
              onOpenDirectory={onOpenDirectory}
            />
          ))}
        </div>
      )}
    </section>
  );
}

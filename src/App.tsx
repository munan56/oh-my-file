import { invoke } from '@tauri-apps/api/core';
import { QRCode } from './components/QRCode';
import { StatusBar } from './components/StatusBar';
import { TransferList } from './components/TransferList';
import { useTransfers } from './hooks/useTransfers';
import './styles/global.css';

export default function App() {
  const {
    serverInfo,
    transfers,
    activeCount,
    refreshNetworkInfo,
    selectNetworkInterface,
  } = useTransfers();

  const handleCancel = async (id: string) => {
    await invoke('cancel_transfer', { transferId: id });
  };

  const handleOpenDirectory = async (path: string) => {
    await invoke('open_directory', { path });
  };

  return (
    <main className="app-shell">
      <aside className="app-sidebar">
        <QRCode
          serverInfo={serverInfo}
          onRefresh={refreshNetworkInfo}
          onSelectInterface={selectNetworkInterface}
        />
      </aside>

      <section className="app-main">
        <TransferList
          items={transfers}
          onCancel={handleCancel}
          onOpenDirectory={handleOpenDirectory}
        />
      </section>

      <StatusBar
        status={serverInfo?.status ?? 'starting'}
        connectedDevices={serverInfo?.connectedDevices ?? 0}
        activeTransfers={activeCount}
      />
    </main>
  );
}

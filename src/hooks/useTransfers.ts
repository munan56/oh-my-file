import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useMemo, useState } from 'react';
import type { NetworkInterface, ServerInfo, TransferItemState } from '../types/transfer';

type RawServerInfo = {
  status: 'starting' | 'running' | 'error';
  port: number;
  token: string;
  ip: string;
  url: string;
  qr_code_data_url: string;
  connected_devices: number;
  selected_interface_id?: string;
  interfaces: Array<{
    id: string;
    name: string;
    ip: string;
    is_default: boolean;
  }>;
};

type TransferEvent =
  | { type: 'server-ready'; payload: RawServerInfo }
  | { type: 'devices-changed'; payload: { connectedDevices: number } }
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
        directoryPath: string;
      };
    }
  | {
      type: 'transfer-error';
      payload: {
        id: string;
        errorMessage: string;
      };
    };

function toServerInfo(raw: RawServerInfo): ServerInfo {
  const interfaces: NetworkInterface[] = raw.interfaces.map((item) => ({
    id: item.id,
    name: item.name,
    ip: item.ip,
    isDefault: item.is_default,
  }));

  return {
    status: raw.status,
    port: raw.port,
    token: raw.token,
    ip: raw.ip,
    url: raw.url,
    qrCodeDataUrl: raw.qr_code_data_url,
    connectedDevices: raw.connected_devices,
    selectedInterfaceId: raw.selected_interface_id,
    interfaces,
  };
}

export function useTransfers() {
  const [serverInfo, setServerInfo] = useState<ServerInfo | null>(null);
  const [transfers, setTransfers] = useState<TransferItemState[]>([]);

  useEffect(() => {
    let disposed = false;

    const bootstrap = async () => {
      const info = await invoke<RawServerInfo>('get_server_info');
      if (!disposed) {
        setServerInfo(toServerInfo(info));
      }
    };

    const refreshTimer = window.setInterval(async () => {
      const info = await invoke<RawServerInfo>('refresh_network_info');
      if (!disposed) {
        setServerInfo(toServerInfo(info));
      }
    }, 5000);

    const unlistenPromise = listen<TransferEvent>('transfer-event', (event) => {
      const message = event.payload;

      switch (message.type) {
        case 'server-ready':
          setServerInfo(toServerInfo(message.payload));
          break;
        case 'devices-changed':
          setServerInfo((current) =>
            current
              ? { ...current, connectedDevices: message.payload.connectedDevices }
              : current,
          );
          break;
        case 'transfer-progress':
          setTransfers((current) => {
            const next = [...current];
            const index = next.findIndex((item) => item.id === message.payload.id);
            const item: TransferItemState = {
              id: message.payload.id,
              filename: message.payload.filename,
              deviceName: message.payload.deviceName,
              totalBytes: message.payload.totalBytes,
              receivedBytes: message.payload.receivedBytes,
              status: message.payload.status,
            };

            if (index === -1) {
              next.unshift(item);
            } else {
              next[index] = { ...next[index], ...item };
            }

            return next;
          });
          break;
        case 'transfer-complete':
          setTransfers((current) =>
            current.map((item) =>
              item.id === message.payload.id
                ? {
                    ...item,
                    status: 'completed',
                    savePath: message.payload.savePath,
                    directoryPath: message.payload.directoryPath,
                  }
                : item,
            ),
          );
          break;
        case 'transfer-error':
          setTransfers((current) =>
            current.map((item) =>
              item.id === message.payload.id
                ? {
                    ...item,
                    status:
                      message.payload.errorMessage === '用户取消传输'
                        ? 'cancelled'
                        : 'error',
                    errorMessage: message.payload.errorMessage,
                  }
                : item,
            ),
          );
          break;
      }
    });

    void bootstrap();

    return () => {
      disposed = true;
      window.clearInterval(refreshTimer);
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
    refreshNetworkInfo: async () => {
      const info = await invoke<RawServerInfo>('refresh_network_info');
      setServerInfo(toServerInfo(info));
    },
    selectNetworkInterface: async (interfaceId: string) => {
      const info = await invoke<RawServerInfo>('select_network_interface', {
        interfaceId,
      });
      setServerInfo(toServerInfo(info));
    },
  };
}

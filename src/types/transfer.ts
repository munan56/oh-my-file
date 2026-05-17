export interface NetworkInterface {
  id: string;
  name: string;
  ip: string;
  isDefault: boolean;
}

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
  directoryPath?: string;
}

export interface ServerInfo {
  status: 'starting' | 'running' | 'error';
  port: number;
  token: string;
  ip: string;
  url: string;
  qrCodeDataUrl: string;
  connectedDevices: number;
  selectedInterfaceId?: string;
  interfaces: NetworkInterface[];
}

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
  const tone =
    status === 'running'
      ? 'status-pill running'
      : status === 'starting'
        ? 'status-pill starting'
        : 'status-pill error';

  return (
    <footer className="status-bar">
      <span className={tone}>状态: {label}</span>
      <span>已连接: {connectedDevices} 台设备</span>
      <span>进行中: {activeTransfers} 个任务</span>
    </footer>
  );
}

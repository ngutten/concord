interface PresenceIndicatorProps {
  status: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const statusColors: Record<string, string> = {
  online: 'bg-green-500',
  idle: 'bg-yellow-500',
  dnd: 'bg-red-500',
  offline: 'bg-gray-500',
};

const statusLabels: Record<string, string> = {
  online: 'Online',
  idle: 'Idle',
  dnd: 'Do Not Disturb',
  offline: 'Offline',
};

const sizes = {
  sm: 'h-2 w-2',
  md: 'h-3 w-3',
  lg: 'h-3.5 w-3.5',
};

export function PresenceIndicator({ status, size = 'md', className = '' }: PresenceIndicatorProps) {
  const color = statusColors[status] || statusColors.offline;
  const sizeClass = sizes[size];

  return (
    <span
      className={`inline-block rounded-full border-2 border-bg-primary ${color} ${sizeClass} ${className}`}
      title={statusLabels[status] || 'Unknown'}
    />
  );
}

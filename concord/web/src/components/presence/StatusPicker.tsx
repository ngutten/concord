import { useState } from 'react';
import { useChatStore } from '../../stores/chatStore';

const statusOptions = [
  { value: 'online', label: 'Online', color: 'bg-green-500' },
  { value: 'idle', label: 'Idle', color: 'bg-yellow-500' },
  { value: 'dnd', label: 'Do Not Disturb', color: 'bg-red-500' },
  { value: 'invisible', label: 'Invisible', color: 'bg-gray-500' },
];

interface StatusPickerProps {
  onClose: () => void;
}

export function StatusPicker({ onClose }: StatusPickerProps) {
  const setPresence = useChatStore((s) => s.setPresence);
  const [customStatus, setCustomStatus] = useState('');
  const [statusEmoji, setStatusEmoji] = useState('');

  const handleSetStatus = (status: string) => {
    setPresence(status, customStatus || undefined, statusEmoji || undefined);
    onClose();
  };

  return (
    <div className="w-64 rounded-lg border border-border bg-bg-primary p-3 shadow-lg">
      <div className="mb-2 text-xs font-semibold uppercase text-text-muted">Set Status</div>
      <div className="mb-3 space-y-1">
        {statusOptions.map((opt) => (
          <button
            key={opt.value}
            onClick={() => handleSetStatus(opt.value)}
            className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm text-text-primary transition-colors hover:bg-bg-hover"
          >
            <span className={`inline-block h-3 w-3 rounded-full ${opt.color}`} />
            {opt.label}
          </button>
        ))}
      </div>
      <div className="border-t border-border pt-2">
        <div className="mb-1 text-xs font-semibold uppercase text-text-muted">Custom Status</div>
        <div className="flex gap-1">
          <input
            type="text"
            value={statusEmoji}
            onChange={(e) => setStatusEmoji(e.target.value)}
            placeholder="😊"
            className="w-10 rounded border border-border bg-bg-secondary px-1 py-1 text-center text-sm"
            maxLength={2}
          />
          <input
            type="text"
            value={customStatus}
            onChange={(e) => setCustomStatus(e.target.value)}
            placeholder="What's happening?"
            className="flex-1 rounded border border-border bg-bg-secondary px-2 py-1 text-sm"
            maxLength={128}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleSetStatus('online');
            }}
          />
        </div>
      </div>
    </div>
  );
}

import { useEffect } from 'react';
import { useChatStore } from '../../stores/chatStore';
import { useUiStore } from '../../stores/uiStore';
import { channelKey } from '../../api/types';
import type { ChannelInfo } from '../../api/types';

const EMPTY_CHANNELS: ChannelInfo[] = [];
const EMPTY_UNREAD: Record<string, number> = {};

export function ChannelList() {
  const activeServer = useUiStore((s) => s.activeServer);
  const channels = useChatStore((s) => (activeServer ? s.channels[activeServer] ?? EMPTY_CHANNELS : EMPTY_CHANNELS));
  const activeChannel = useUiStore((s) => s.activeChannel);
  const setActiveChannel = useUiStore((s) => s.setActiveChannel);
  const joinChannel = useChatStore((s) => s.joinChannel);
  const getMembers = useChatStore((s) => s.getMembers);
  const fetchHistory = useChatStore((s) => s.fetchHistory);
  const servers = useChatStore((s) => s.servers);
  const unreadCounts = useChatStore((s) => s.unreadCounts ?? EMPTY_UNREAD);
  const markRead = useChatStore((s) => s.markRead);
  const getUnreadCounts = useChatStore((s) => s.getUnreadCounts);
  const messages = useChatStore((s) => s.messages);

  const serverName = servers.find((s) => s.id === activeServer)?.name ?? 'Concord';

  // Fetch unread counts when server changes
  useEffect(() => {
    if (activeServer) {
      getUnreadCounts(activeServer);
    }
  }, [activeServer, getUnreadCounts]);

  // Auto-mark-read when viewing a channel (clear unread for active channel)
  useEffect(() => {
    if (!activeServer || !activeChannel) return;
    const key = channelKey(activeServer, activeChannel);
    const channelMessages = messages[key];
    if (channelMessages && channelMessages.length > 0) {
      const lastMsg = channelMessages[channelMessages.length - 1];
      markRead(activeServer, activeChannel, lastMsg.id);
    }
  }, [activeServer, activeChannel, messages, markRead]);

  const handleSelect = (name: string) => {
    if (!activeServer) return;
    setActiveChannel(name);
    joinChannel(activeServer, name);
    getMembers(activeServer, name);
    fetchHistory(activeServer, name);
  };

  return (
    <div className="flex h-full flex-col bg-bg-secondary">
      <div className="flex h-12 items-center border-b border-border-primary px-4">
        <h2 className="font-semibold text-text-primary truncate">{serverName}</h2>
      </div>

      <div className="flex-1 overflow-y-auto px-2 pt-4">
        <div className="mb-2 flex items-center justify-between px-2">
          <span className="text-xs font-semibold uppercase tracking-wide text-text-muted">
            Channels
          </span>
        </div>

        {channels.map((ch) => {
          const key = channelKey(activeServer!, ch.name);
          const unread = unreadCounts[key] || 0;
          const isActive = activeChannel === ch.name;

          return (
            <button
              key={ch.name}
              onClick={() => handleSelect(ch.name)}
              className={`mb-0.5 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm transition-colors ${
                isActive
                  ? 'bg-bg-active text-text-primary'
                  : unread > 0
                    ? 'text-text-primary font-semibold hover:bg-bg-hover'
                    : 'text-text-muted hover:bg-bg-hover hover:text-text-secondary'
              }`}
            >
              <span className="text-lg leading-none text-text-muted">#</span>
              <span className="min-w-0 flex-1 truncate">{ch.name.replace(/^#/, '')}</span>
              {unread > 0 && !isActive && (
                <span className="flex h-5 min-w-5 items-center justify-center rounded-full bg-red-500 px-1.5 text-xs font-bold text-white">
                  {unread > 99 ? '99+' : unread}
                </span>
              )}
            </button>
          );
        })}
      </div>

      <div className="border-t border-border-primary px-2 py-2">
        <UserBar />
      </div>
    </div>
  );
}

function UserBar() {
  const setShowSettings = useUiStore((s) => s.setShowSettings);

  return (
    <button
      onClick={() => setShowSettings(true)}
      className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm text-text-secondary transition-colors hover:bg-bg-hover"
    >
      <div className="flex h-8 w-8 items-center justify-center rounded-full bg-bg-accent text-xs font-bold text-white">
        U
      </div>
      <span className="truncate">Settings</span>
    </button>
  );
}

import { useState, useCallback } from 'react';
import { useChatStore } from '../../stores/chatStore';
import { useUiStore } from '../../stores/uiStore';

export function SearchPanel() {
  const show = useUiStore((s) => s.showSearch);
  const setShow = useUiStore((s) => s.setShowSearch);
  const activeServer = useUiStore((s) => s.activeServer);
  const setActiveChannel = useUiStore((s) => s.setActiveChannel);
  const joinChannel = useChatStore((s) => s.joinChannel);
  const searchMessages = useChatStore((s) => s.searchMessages);
  const clearSearch = useChatStore((s) => s.clearSearch);
  const results = useChatStore((s) => s.searchResults);
  const totalCount = useChatStore((s) => s.searchTotalCount);

  const [query, setQuery] = useState('');
  const [channelFilter, setChannelFilter] = useState('');

  const handleSearch = useCallback(() => {
    if (!query.trim() || !activeServer) return;
    searchMessages(activeServer, query.trim(), channelFilter || undefined);
  }, [query, channelFilter, activeServer, searchMessages]);

  if (!show) return null;

  return (
    <div className="flex h-full w-80 flex-col border-l border-border bg-bg-secondary">
      <div className="flex items-center justify-between border-b border-border px-3 py-2">
        <span className="text-sm font-semibold text-text-primary">Search</span>
        <button onClick={() => { setShow(false); clearSearch(); }} className="text-text-muted hover:text-text-primary">
          <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="space-y-2 p-3">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
          placeholder="Search messages..."
          className="w-full rounded border border-border bg-bg-primary px-2 py-1.5 text-sm text-text-primary outline-none placeholder:text-text-muted focus:border-accent"
          autoFocus
        />
        <input
          type="text"
          value={channelFilter}
          onChange={(e) => setChannelFilter(e.target.value)}
          placeholder="Filter by channel (optional)"
          className="w-full rounded border border-border bg-bg-primary px-2 py-1.5 text-sm text-text-primary outline-none placeholder:text-text-muted focus:border-accent"
        />
        <button
          onClick={handleSearch}
          disabled={!query.trim()}
          className="w-full rounded bg-accent px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-accent/90 disabled:opacity-50"
        >
          Search
        </button>
      </div>

      {results && (
        <div className="flex-1 overflow-y-auto">
          <div className="px-3 py-1 text-xs text-text-muted">
            {totalCount} result{totalCount !== 1 ? 's' : ''}
          </div>
          {results.map((msg) => (
            <button
              key={msg.id}
              onClick={() => {
                if (activeServer && msg.channel_name) {
                  setActiveChannel(msg.channel_name);
                  joinChannel(activeServer, msg.channel_name);
                  setShow(false);
                }
              }}
              className="w-full border-b border-border px-3 py-2 text-left transition-colors hover:bg-bg-hover"
            >
              <div className="flex items-baseline gap-1">
                <span className="text-sm font-medium text-text-primary">{msg.from}</span>
                <span className="text-xs text-text-muted">in #{msg.channel_name}</span>
                <span className="ml-auto text-xs text-text-muted">
                  {new Date(msg.timestamp).toLocaleDateString()}
                </span>
              </div>
              <div className="mt-0.5 text-sm text-text-secondary line-clamp-2">{msg.content}</div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

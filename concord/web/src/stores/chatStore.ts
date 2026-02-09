import { create } from 'zustand';
import type { ChannelInfo, HistoryMessage, MemberInfo, ServerEvent } from '../api/types';
import { WebSocketManager } from '../api/websocket';

interface ChatState {
  connected: boolean;
  nickname: string | null;
  channels: ChannelInfo[];
  messages: Record<string, HistoryMessage[]>;
  members: Record<string, MemberInfo[]>;
  hasMore: Record<string, boolean>;
  /** nickname -> avatar_url cache (populated from Names/Join/Message events) */
  avatars: Record<string, string>;
  ws: WebSocketManager | null;

  connect: (nickname: string) => void;
  disconnect: () => void;
  handleEvent: (event: ServerEvent) => void;
  sendMessage: (channel: string, content: string) => void;
  joinChannel: (channel: string) => void;
  partChannel: (channel: string) => void;
  setTopic: (channel: string, topic: string) => void;
  fetchHistory: (channel: string, before?: string) => void;
  listChannels: () => void;
  getMembers: (channel: string) => void;
}

/** Cache an avatar_url for a nickname if present. */
function cacheAvatar(avatars: Record<string, string>, nickname: string, avatar_url?: string | null): Record<string, string> {
  if (avatar_url && avatars[nickname] !== avatar_url) {
    return { ...avatars, [nickname]: avatar_url };
  }
  return avatars;
}

export const useChatStore = create<ChatState>((set, get) => ({
  connected: false,
  nickname: null,
  channels: [],
  messages: {},
  members: {},
  hasMore: {},
  avatars: {},
  ws: null,

  connect: (nickname: string) => {
    if (get().ws) {
      return;
    }

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws?nickname=${encodeURIComponent(nickname)}`;

    const ws = new WebSocketManager(
      url,
      (event) => {
        get().handleEvent(event);
      },
      (connected) => {
        set({ connected });
        if (connected) {
          ws.send({ type: 'list_channels' });
        }
      },
    );

    set({ ws, nickname });
    ws.connect();
  },

  disconnect: () => {
    get().ws?.disconnect();
    set({ ws: null, connected: false });
  },

  handleEvent: (event: ServerEvent) => {
    switch (event.type) {
      case 'message': {
        const msg: HistoryMessage = {
          id: event.id,
          from: event.from,
          content: event.content,
          timestamp: event.timestamp,
        };
        set((s) => ({
          messages: {
            ...s.messages,
            [event.target]: [...(s.messages[event.target] || []), msg],
          },
          avatars: cacheAvatar(s.avatars, event.from, event.avatar_url),
        }));
        break;
      }

      case 'join': {
        const memberInfo: MemberInfo = { nickname: event.nickname, avatar_url: event.avatar_url };
        set((s) => {
          const current = s.members[event.channel] || [];
          if (current.some((m) => m.nickname === event.nickname)) return s;
          return {
            members: {
              ...s.members,
              [event.channel]: [...current, memberInfo],
            },
            avatars: cacheAvatar(s.avatars, event.nickname, event.avatar_url),
          };
        });
        break;
      }

      case 'part': {
        set((s) => ({
          members: {
            ...s.members,
            [event.channel]: (s.members[event.channel] || []).filter(
              (m) => m.nickname !== event.nickname,
            ),
          },
        }));
        break;
      }

      case 'quit': {
        set((s) => {
          const newMembers = { ...s.members };
          for (const ch in newMembers) {
            newMembers[ch] = newMembers[ch].filter((m) => m.nickname !== event.nickname);
          }
          return { members: newMembers };
        });
        break;
      }

      case 'names': {
        set((s) => {
          let newAvatars = { ...s.avatars };
          for (const m of event.members) {
            if (m.avatar_url) {
              newAvatars[m.nickname] = m.avatar_url;
            }
          }
          return {
            members: { ...s.members, [event.channel]: event.members },
            avatars: newAvatars,
          };
        });
        break;
      }

      case 'topic_change': {
        set((s) => ({
          channels: s.channels.map((ch) =>
            ch.name === event.channel ? { ...ch, topic: event.topic } : ch,
          ),
        }));
        break;
      }

      case 'channel_list': {
        set({ channels: event.channels });
        break;
      }

      case 'history': {
        set((s) => ({
          messages: {
            ...s.messages,
            [event.channel]: [
              ...event.messages.reverse(),
              ...(s.messages[event.channel] || []),
            ],
          },
          hasMore: { ...s.hasMore, [event.channel]: event.has_more },
        }));
        break;
      }

      case 'error': {
        console.error(`Server error [${event.code}]: ${event.message}`);
        break;
      }
    }
  },

  sendMessage: (channel, content) => {
    const { ws, nickname } = get();
    if (!ws || !nickname) return;

    // Add message locally (server excludes sender from broadcast)
    const msg: HistoryMessage = {
      id: crypto.randomUUID(),
      from: nickname,
      content,
      timestamp: new Date().toISOString(),
    };
    set((s) => ({
      messages: {
        ...s.messages,
        [channel]: [...(s.messages[channel] || []), msg],
      },
    }));

    ws.send({ type: 'send_message', channel, content });
  },

  joinChannel: (channel) => {
    get().ws?.send({ type: 'join_channel', channel });
  },

  partChannel: (channel) => {
    get().ws?.send({ type: 'part_channel', channel });
  },

  setTopic: (channel, topic) => {
    get().ws?.send({ type: 'set_topic', channel, topic });
  },

  fetchHistory: (channel, before) => {
    get().ws?.send({ type: 'fetch_history', channel, before, limit: 50 });
  },

  listChannels: () => {
    get().ws?.send({ type: 'list_channels' });
  },

  getMembers: (channel) => {
    get().ws?.send({ type: 'get_members', channel });
  },
}));

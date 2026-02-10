import { create } from 'zustand';
import type { AttachmentInfo, ChannelInfo, HistoryMessage, MemberInfo, ReplyInfo, ServerEvent, ServerInfo } from '../api/types';
import { listServerEmoji } from '../api/client';
import { channelKey } from '../api/types';
import { WebSocketManager } from '../api/websocket';

// Stable empty references to prevent zustand selector re-render loops.
// Inline [] / {} in selectors create new references on every evaluation,
// failing Object.is comparison and causing infinite re-renders with React 19.
const EMPTY_SERVERS: ServerInfo[] = [];
const EMPTY_CHANNELS_MAP: Record<string, ChannelInfo[]> = {};
const EMPTY_MESSAGES_MAP: Record<string, HistoryMessage[]> = {};
const EMPTY_MEMBERS_MAP: Record<string, MemberInfo[]> = {};
const EMPTY_HAS_MORE: Record<string, boolean> = {};
const EMPTY_AVATARS: Record<string, string> = {};
const EMPTY_TYPING: Record<string, string[]> = {};
const EMPTY_UNREAD: Record<string, number> = {};
const EMPTY_EMOJI: Record<string, Record<string, string>> = {};

interface ChatState {
  connected: boolean;
  nickname: string | null;
  servers: ServerInfo[];
  channels: Record<string, ChannelInfo[]>;   // server_id -> channels
  messages: Record<string, HistoryMessage[]>; // channelKey -> messages
  members: Record<string, MemberInfo[]>;      // channelKey -> members
  hasMore: Record<string, boolean>;           // channelKey -> has_more
  /** nickname -> avatar_url cache (populated from Names/Join/Message events) */
  avatars: Record<string, string>;
  /** channelKey -> list of nicknames currently typing */
  typingUsers: Record<string, string[]>;
  /** The message being replied to (if any) */
  replyingTo: ReplyInfo | null;
  /** channelKey -> unread message count */
  unreadCounts: Record<string, number>;
  /** server_id -> { emoji_name -> image_url } */
  customEmoji: Record<string, Record<string, string>>;
  ws: WebSocketManager | null;

  connect: (nickname: string) => void;
  disconnect: () => void;
  handleEvent: (event: ServerEvent) => void;
  sendMessage: (serverId: string, channel: string, content: string, attachments?: AttachmentInfo[]) => void;
  editMessage: (messageId: string, content: string) => void;
  deleteMessage: (messageId: string) => void;
  addReaction: (messageId: string, emoji: string) => void;
  removeReaction: (messageId: string, emoji: string) => void;
  sendTyping: (serverId: string, channel: string) => void;
  setReplyingTo: (reply: ReplyInfo | null) => void;
  markRead: (serverId: string, channel: string, messageId: string) => void;
  getUnreadCounts: (serverId: string) => void;
  joinChannel: (serverId: string, channel: string) => void;
  partChannel: (serverId: string, channel: string) => void;
  setTopic: (serverId: string, channel: string, topic: string) => void;
  fetchHistory: (serverId: string, channel: string, before?: string) => void;
  listChannels: (serverId: string) => void;
  getMembers: (serverId: string, channel: string) => void;
  listServers: () => void;
  createServer: (name: string, iconUrl?: string) => void;
  joinServer: (serverId: string) => void;
  leaveServer: (serverId: string) => void;
  createChannel: (serverId: string, name: string) => void;
  deleteChannel: (serverId: string, channel: string) => void;
  deleteServer: (serverId: string) => void;
  loadServerEmoji: (serverId: string) => void;
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
  servers: EMPTY_SERVERS,
  channels: EMPTY_CHANNELS_MAP,
  messages: EMPTY_MESSAGES_MAP,
  members: EMPTY_MEMBERS_MAP,
  hasMore: EMPTY_HAS_MORE,
  avatars: EMPTY_AVATARS,
  typingUsers: EMPTY_TYPING,
  replyingTo: null,
  unreadCounts: EMPTY_UNREAD,
  customEmoji: EMPTY_EMOJI,
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
          ws.send({ type: 'list_servers' });
        }
      },
    );

    set({ ws, nickname });
    ws.connect();
  },

  disconnect: () => {
    get().ws?.disconnect();
    set({
      ws: null,
      connected: false,
      servers: EMPTY_SERVERS,
      channels: EMPTY_CHANNELS_MAP,
      messages: EMPTY_MESSAGES_MAP,
      members: EMPTY_MEMBERS_MAP,
      hasMore: EMPTY_HAS_MORE,
      avatars: EMPTY_AVATARS,
      typingUsers: EMPTY_TYPING,
      replyingTo: null,
      unreadCounts: EMPTY_UNREAD,
      customEmoji: EMPTY_EMOJI,
    });
  },

  handleEvent: (event: ServerEvent) => {
    switch (event.type) {
      case 'message': {
        const sid = event.server_id || 'default';
        const key = channelKey(sid, event.target);
        const msg: HistoryMessage = {
          id: event.id,
          from: event.from,
          content: event.content,
          timestamp: event.timestamp,
          reply_to: event.reply_to,
          attachments: event.attachments,
        };
        set((s) => {
          // Increment unread count for messages from others
          const newUnread = { ...s.unreadCounts };
          if (event.from !== s.nickname) {
            newUnread[key] = (newUnread[key] || 0) + 1;
          }
          return {
            messages: {
              ...s.messages,
              [key]: [...(s.messages[key] || []), msg],
            },
            avatars: cacheAvatar(s.avatars, event.from, event.avatar_url),
            unreadCounts: newUnread,
          };
        });
        break;
      }

      case 'message_edit': {
        const key = channelKey(event.server_id, event.channel);
        set((s) => ({
          messages: {
            ...s.messages,
            [key]: (s.messages[key] || []).map((m) =>
              m.id === event.id ? { ...m, content: event.content, edited_at: event.edited_at } : m,
            ),
          },
        }));
        break;
      }

      case 'message_delete': {
        const key = channelKey(event.server_id, event.channel);
        set((s) => ({
          messages: {
            ...s.messages,
            [key]: (s.messages[key] || []).filter((m) => m.id !== event.id),
          },
        }));
        break;
      }

      case 'message_embed': {
        const key = channelKey(event.server_id, event.channel);
        set((s) => ({
          messages: {
            ...s.messages,
            [key]: (s.messages[key] || []).map((m) =>
              m.id === event.message_id ? { ...m, embeds: event.embeds } : m,
            ),
          },
        }));
        break;
      }

      case 'reaction_add': {
        const key = channelKey(event.server_id, event.channel);
        set((s) => ({
          messages: {
            ...s.messages,
            [key]: (s.messages[key] || []).map((m) => {
              if (m.id !== event.message_id) return m;
              const reactions = [...(m.reactions || [])];
              const existing = reactions.find((r) => r.emoji === event.emoji);
              if (existing) {
                if (!existing.user_ids.includes(event.user_id)) {
                  existing.user_ids = [...existing.user_ids, event.user_id];
                  existing.count = existing.user_ids.length;
                }
              } else {
                reactions.push({ emoji: event.emoji, count: 1, user_ids: [event.user_id] });
              }
              return { ...m, reactions };
            }),
          },
        }));
        break;
      }

      case 'reaction_remove': {
        const key = channelKey(event.server_id, event.channel);
        set((s) => ({
          messages: {
            ...s.messages,
            [key]: (s.messages[key] || []).map((m) => {
              if (m.id !== event.message_id) return m;
              let reactions = (m.reactions || [])
                .map((r) => {
                  if (r.emoji !== event.emoji) return r;
                  const user_ids = r.user_ids.filter((uid) => uid !== event.user_id);
                  return { ...r, user_ids, count: user_ids.length };
                })
                .filter((r) => r.count > 0);
              if (reactions.length === 0) reactions = [];
              return { ...m, reactions };
            }),
          },
        }));
        break;
      }

      case 'typing_start': {
        const key = channelKey(event.server_id, event.channel);
        const myNick = get().nickname;
        if (event.nickname === myNick) break; // Don't show own typing
        set((s) => {
          const current = s.typingUsers[key] || [];
          if (current.includes(event.nickname)) return s;
          return {
            typingUsers: { ...s.typingUsers, [key]: [...current, event.nickname] },
          };
        });
        // Auto-expire after 8 seconds
        setTimeout(() => {
          set((s) => {
            const current = s.typingUsers[key] || [];
            const filtered = current.filter((n) => n !== event.nickname);
            return {
              typingUsers: { ...s.typingUsers, [key]: filtered },
            };
          });
        }, 8000);
        break;
      }

      case 'join': {
        const key = channelKey(event.server_id, event.channel);
        const memberInfo: MemberInfo = { nickname: event.nickname, avatar_url: event.avatar_url };
        set((s) => {
          const current = s.members[key] || [];
          if (current.some((m) => m.nickname === event.nickname)) return s;
          return {
            members: {
              ...s.members,
              [key]: [...current, memberInfo],
            },
            avatars: cacheAvatar(s.avatars, event.nickname, event.avatar_url),
          };
        });
        break;
      }

      case 'part': {
        const key = channelKey(event.server_id, event.channel);
        set((s) => ({
          members: {
            ...s.members,
            [key]: (s.members[key] || []).filter(
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
        const key = channelKey(event.server_id, event.channel);
        set((s) => {
          let newAvatars = { ...s.avatars };
          for (const m of event.members) {
            if (m.avatar_url) {
              newAvatars[m.nickname] = m.avatar_url;
            }
          }
          return {
            members: { ...s.members, [key]: event.members },
            avatars: newAvatars,
          };
        });
        break;
      }

      case 'topic_change': {
        set((s) => {
          const serverChannels = s.channels[event.server_id];
          if (!serverChannels) return s;
          return {
            channels: {
              ...s.channels,
              [event.server_id]: serverChannels.map((ch) =>
                ch.name === event.channel ? { ...ch, topic: event.topic } : ch,
              ),
            },
          };
        });
        break;
      }

      case 'channel_list': {
        set((s) => ({
          channels: { ...s.channels, [event.server_id]: event.channels },
        }));
        break;
      }

      case 'history': {
        const key = channelKey(event.server_id, event.channel);
        set((s) => ({
          messages: {
            ...s.messages,
            [key]: [
              ...event.messages.reverse(),
              ...(s.messages[key] || []),
            ],
          },
          hasMore: { ...s.hasMore, [key]: event.has_more },
        }));
        break;
      }

      case 'server_list': {
        set({ servers: event.servers });
        break;
      }

      case 'unread_counts': {
        set((s) => {
          const newUnread = { ...s.unreadCounts };
          for (const { channel_name, count } of event.counts) {
            const key = channelKey(event.server_id, channel_name);
            newUnread[key] = count;
          }
          return { unreadCounts: newUnread };
        });
        break;
      }

      case 'error': {
        console.error(`Server error [${event.code}]: ${event.message}`);
        break;
      }
    }
  },

  sendMessage: (serverId, channel, content, attachments) => {
    const { ws, nickname, replyingTo } = get();
    if (!ws || !nickname) return;

    const key = channelKey(serverId, channel);

    // Add message locally (server excludes sender from broadcast)
    const msg: HistoryMessage = {
      id: crypto.randomUUID(),
      from: nickname,
      content,
      timestamp: new Date().toISOString(),
      reply_to: replyingTo,
      attachments: attachments || null,
    };
    set((s) => ({
      messages: {
        ...s.messages,
        [key]: [...(s.messages[key] || []), msg],
      },
      replyingTo: null,
    }));

    ws.send({
      type: 'send_message',
      server_id: serverId,
      channel,
      content,
      reply_to: replyingTo?.id,
      attachment_ids: attachments?.map((a) => a.id),
    });
  },

  editMessage: (messageId, content) => {
    get().ws?.send({ type: 'edit_message', message_id: messageId, content });
  },

  deleteMessage: (messageId) => {
    get().ws?.send({ type: 'delete_message', message_id: messageId });
  },

  addReaction: (messageId, emoji) => {
    get().ws?.send({ type: 'add_reaction', message_id: messageId, emoji });
  },

  removeReaction: (messageId, emoji) => {
    get().ws?.send({ type: 'remove_reaction', message_id: messageId, emoji });
  },

  sendTyping: (serverId, channel) => {
    get().ws?.send({ type: 'typing', server_id: serverId, channel });
  },

  setReplyingTo: (reply) => {
    set({ replyingTo: reply });
  },

  markRead: (serverId, channel, messageId) => {
    const key = channelKey(serverId, channel);
    get().ws?.send({ type: 'mark_read', server_id: serverId, channel, message_id: messageId });
    // Optimistically clear unread count
    set((s) => {
      const newUnread = { ...s.unreadCounts };
      delete newUnread[key];
      return { unreadCounts: newUnread };
    });
  },

  getUnreadCounts: (serverId) => {
    get().ws?.send({ type: 'get_unread_counts', server_id: serverId });
  },

  joinChannel: (serverId, channel) => {
    get().ws?.send({ type: 'join_channel', server_id: serverId, channel });
  },

  partChannel: (serverId, channel) => {
    get().ws?.send({ type: 'part_channel', server_id: serverId, channel });
  },

  setTopic: (serverId, channel, topic) => {
    get().ws?.send({ type: 'set_topic', server_id: serverId, channel, topic });
  },

  fetchHistory: (serverId, channel, before) => {
    get().ws?.send({ type: 'fetch_history', server_id: serverId, channel, before, limit: 50 });
  },

  listChannels: (serverId) => {
    get().ws?.send({ type: 'list_channels', server_id: serverId });
  },

  getMembers: (serverId, channel) => {
    get().ws?.send({ type: 'get_members', server_id: serverId, channel });
  },

  listServers: () => {
    get().ws?.send({ type: 'list_servers' });
  },

  createServer: (name, iconUrl) => {
    get().ws?.send({ type: 'create_server', name, icon_url: iconUrl });
  },

  joinServer: (serverId) => {
    get().ws?.send({ type: 'join_server', server_id: serverId });
  },

  leaveServer: (serverId) => {
    get().ws?.send({ type: 'leave_server', server_id: serverId });
  },

  createChannel: (serverId, name) => {
    get().ws?.send({ type: 'create_channel', server_id: serverId, name });
  },

  deleteChannel: (serverId, channel) => {
    get().ws?.send({ type: 'delete_channel', server_id: serverId, channel });
  },

  deleteServer: (serverId) => {
    get().ws?.send({ type: 'delete_server', server_id: serverId });
  },

  loadServerEmoji: (serverId) => {
    listServerEmoji(serverId)
      .then((emojis) => {
        const map: Record<string, string> = {};
        for (const e of emojis) {
          map[e.name] = e.image_url;
        }
        set((s) => ({
          customEmoji: { ...s.customEmoji, [serverId]: map },
        }));
      })
      .catch((err) => {
        console.error('Failed to load emoji for server', serverId, err);
      });
  },
}));

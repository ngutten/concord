import { useState, useMemo, useCallback } from 'react';
import { useChatStore } from '../../stores/chatStore';
import { useUiStore } from '../../stores/uiStore';
import { channelKey } from '../../api/types';
import { UserProfilePopup } from './UserProfilePopup';
import { PresenceIndicator } from '../presence/PresenceIndicator';
import type { MemberInfo, RoleInfo } from '../../api/types';

const EMPTY_MEMBERS: MemberInfo[] = [];
const EMPTY_ROLES: RoleInfo[] = [];

interface ContextMenuState {
  userId: string;
  nickname: string;
  x: number;
  y: number;
}

export function MemberList() {
  const activeServer = useUiStore((s) => s.activeServer);
  const activeChannel = useUiStore((s) => s.activeChannel);
  const key = activeServer && activeChannel ? channelKey(activeServer, activeChannel) : null;
  const members = useChatStore((s) => (key ? s.members[key] ?? EMPTY_MEMBERS : EMPTY_MEMBERS));
  const roles = useChatStore((s) => (activeServer ? s.roles[activeServer] ?? EMPTY_ROLES : EMPTY_ROLES));
  const avatars = useChatStore((s) => s.avatars);
  const presences = useChatStore((s) => s.presences);
  const [selectedUser, setSelectedUser] = useState<string | null>(null);
  const [popupAnchor, setPopupAnchor] = useState<{ top: number; left: number } | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const kickMember = useChatStore(s => s.kickMember);
  const banMember = useChatStore(s => s.banMember);
  const timeoutMember = useChatStore(s => s.timeoutMember);

  // Get the top (highest position) role with a color for display
  const topRoleColor = useMemo(() => {
    if (roles.length === 0) return null;
    // Sort by position desc — highest position = most prominent role
    const sorted = [...roles].sort((a, b) => b.position - a.position);
    return sorted.find((r) => r.color)?.color ?? null;
  }, [roles]);

  // Group members by their display (for now, all in one group since member-role mapping
  // isn't tracked per-member yet — we show role headers when data is available)
  const roleGroups = useMemo(() => {
    // Simple single-group for now — members don't carry role_ids yet
    // When member_role_update tracking is added, this will group properly
    const sortedRoles = [...roles].sort((a, b) => b.position - a.position);
    const topRole = sortedRoles.find((r) => !r.is_default && r.color);

    return [{
      roleName: topRole?.name ?? 'Members',
      roleColor: topRole?.color ?? null,
      members,
    }];
  }, [members, roles]);

  const handleMemberClick = (nickname: string, e: React.MouseEvent) => {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    setPopupAnchor({ top: rect.top, left: rect.left - 320 });
    setSelectedUser(nickname);
  };

  const handleContextMenu = useCallback((e: React.MouseEvent, member: MemberInfo) => {
    e.preventDefault();
    if (!member.user_id || !activeServer) return;
    setContextMenu({ userId: member.user_id, nickname: member.nickname, x: e.clientX, y: e.clientY });
  }, [activeServer]);

  const handleKick = useCallback(() => {
    if (!contextMenu || !activeServer) return;
    const reason = prompt('Kick reason (optional):') ?? undefined;
    kickMember(activeServer, contextMenu.userId, reason);
    setContextMenu(null);
  }, [contextMenu, activeServer, kickMember]);

  const handleBan = useCallback(() => {
    if (!contextMenu || !activeServer) return;
    const reason = prompt('Ban reason (optional):') ?? undefined;
    const daysStr = prompt('Delete message history (days, 0-7):', '0');
    const days = daysStr ? parseInt(daysStr, 10) : 0;
    banMember(activeServer, contextMenu.userId, reason, isNaN(days) ? 0 : days);
    setContextMenu(null);
  }, [contextMenu, activeServer, banMember]);

  const handleTimeout = useCallback(() => {
    if (!contextMenu || !activeServer) return;
    const minutes = prompt('Timeout duration in minutes:', '10');
    if (!minutes) { setContextMenu(null); return; }
    const mins = parseInt(minutes, 10);
    if (isNaN(mins) || mins <= 0) { setContextMenu(null); return; }
    const until = new Date(Date.now() + mins * 60 * 1000).toISOString();
    const reason = prompt('Timeout reason (optional):') ?? undefined;
    timeoutMember(activeServer, contextMenu.userId, until, reason);
    setContextMenu(null);
  }, [contextMenu, activeServer, timeoutMember]);

  return (
    <div className="flex h-full w-60 flex-col bg-bg-secondary">
      {roleGroups.map((group) => (
        <div key={group.roleName}>
          <div className="px-4 pt-6">
            <h3
              className="mb-2 text-xs font-semibold uppercase tracking-wide"
              style={{ color: group.roleColor ?? undefined }}
            >
              {!group.roleColor && <span className="text-text-muted">{group.roleName} — {group.members.length}</span>}
              {group.roleColor && <>{group.roleName} — {group.members.length}</>}
            </h3>
          </div>

          <div className="flex-1 overflow-y-auto px-2">
            {group.members.map((member) => {
              const avatarUrl = member.avatar_url || avatars[member.nickname];
              const presence = activeServer ? presences[activeServer]?.[member.user_id || ''] : null;
              const statusValue = presence?.status || member.status || 'online';
              return (
                <button
                  key={member.nickname}
                  onClick={(e) => {
                    if (member.user_id) {
                      useUiStore.getState().setShowUserProfile(member.user_id);
                    }
                    handleMemberClick(member.nickname, e);
                  }}
                  onContextMenu={(e) => handleContextMenu(e, member)}
                  className="group flex w-full items-center gap-3 rounded px-2 py-1.5 text-left hover:bg-bg-hover"
                >
                  <div className="relative">
                    {avatarUrl ? (
                      <img
                        src={avatarUrl}
                        alt={member.nickname}
                        className="h-8 w-8 rounded-full object-cover"
                      />
                    ) : (
                      <div className="flex h-8 w-8 items-center justify-center rounded-full bg-bg-accent text-xs font-bold text-white">
                        {member.nickname[0]?.toUpperCase() || '?'}
                      </div>
                    )}
                    <PresenceIndicator
                      status={statusValue}
                      size="md"
                      className="absolute -bottom-0.5 -right-0.5"
                    />
                  </div>
                  <div className="min-w-0 flex-1">
                    <span
                      className="truncate text-sm"
                      style={{ color: topRoleColor ?? undefined }}
                    >
                      {!topRoleColor && <span className="text-text-secondary">{member.nickname}</span>}
                      {topRoleColor && member.nickname}
                    </span>
                    {presence?.custom_status && (
                      <div className="truncate text-xs text-text-muted">
                        {presence.status_emoji && <span className="mr-0.5">{presence.status_emoji}</span>}
                        {presence.custom_status}
                      </div>
                    )}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      ))}

      {selectedUser && popupAnchor && (
        <UserProfilePopup
          nickname={selectedUser}
          position={popupAnchor}
          onClose={() => setSelectedUser(null)}
        />
      )}

      {/* Moderation context menu */}
      {contextMenu && (
        <div
          className="fixed inset-0 z-50"
          onClick={() => setContextMenu(null)}
          onContextMenu={(e) => { e.preventDefault(); setContextMenu(null); }}
        >
          <div
            className="absolute rounded bg-bg-primary shadow-lg border border-border py-1 min-w-[160px]"
            style={{ top: contextMenu.y, left: contextMenu.x }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="px-3 py-1.5 text-xs font-semibold text-text-muted border-b border-border mb-1">
              {contextMenu.nickname}
            </div>
            <button
              onClick={handleKick}
              className="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-yellow-400 hover:bg-bg-hover"
            >
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              Kick
            </button>
            <button
              onClick={handleBan}
              className="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-red-400 hover:bg-bg-hover"
            >
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
              </svg>
              Ban
            </button>
            <button
              onClick={handleTimeout}
              className="flex w-full items-center gap-2 px-3 py-1.5 text-sm text-orange-400 hover:bg-bg-hover"
            >
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Timeout
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

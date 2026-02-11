import { useEffect } from 'react';
import { useChatStore } from '../../stores/chatStore';
import { useUiStore } from '../../stores/uiStore';
import { PresenceIndicator } from '../presence/PresenceIndicator';

export function UserProfileModal() {
  const userId = useUiStore((s) => s.showUserProfile);
  const setShowUserProfile = useUiStore((s) => s.setShowUserProfile);
  const getUserProfile = useChatStore((s) => s.getUserProfile);
  const userProfiles = useChatStore((s) => s.userProfiles);
  const activeServer = useUiStore((s) => s.activeServer);
  const presences = useChatStore((s) => s.presences);

  useEffect(() => {
    if (userId) getUserProfile(userId);
  }, [userId, getUserProfile]);

  if (!userId) return null;

  const profile = userProfiles[userId];
  const presence = activeServer ? presences[activeServer]?.[userId] : null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={() => setShowUserProfile(null)}>
      <div className="w-[340px] overflow-hidden rounded-lg bg-bg-primary shadow-xl" onClick={(e) => e.stopPropagation()}>
        {/* Banner */}
        <div
          className="h-[100px] bg-gradient-to-br from-indigo-500 to-purple-600"
          style={profile?.banner_url ? { backgroundImage: `url(${profile.banner_url})`, backgroundSize: 'cover' } : {}}
        />

        {/* Avatar + Status */}
        <div className="relative px-4">
          <div className="relative -mt-10 mb-2 inline-block">
            <img
              src={profile?.avatar_url || `https://ui-avatars.com/api/?name=${profile?.username || '?'}&background=5865F2&color=fff`}
              className="h-20 w-20 rounded-full border-4 border-bg-primary"
              alt=""
            />
            {presence && (
              <PresenceIndicator
                status={presence.status}
                size="lg"
                className="absolute bottom-0 right-0"
              />
            )}
          </div>
        </div>

        <div className="px-4 pb-4">
          <div className="text-lg font-bold text-text-primary">{profile?.username || 'Loading...'}</div>
          {profile?.pronouns && (
            <div className="text-sm text-text-muted">{profile.pronouns}</div>
          )}
          {presence?.custom_status && (
            <div className="mt-1 text-sm text-text-secondary">
              {presence.status_emoji && <span className="mr-1">{presence.status_emoji}</span>}
              {presence.custom_status}
            </div>
          )}

          {profile?.bio && (
            <div className="mt-3 border-t border-border pt-3">
              <div className="mb-1 text-xs font-semibold uppercase text-text-muted">About Me</div>
              <div className="text-sm text-text-secondary">{profile.bio}</div>
            </div>
          )}

          <div className="mt-3 border-t border-border pt-3">
            <div className="mb-1 text-xs font-semibold uppercase text-text-muted">Member Since</div>
            <div className="text-sm text-text-secondary">
              {profile?.created_at ? new Date(profile.created_at).toLocaleDateString() : '---'}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

import { useEffect, useRef, useState } from 'react';
import { useChatStore } from '../../stores/chatStore';
import { getUserProfile } from '../../api/client';
import type { PublicUserProfile } from '../../api/types';

interface Props {
  nickname: string;
  position: { top: number; left: number };
  onClose: () => void;
}

export function UserProfilePopup({ nickname, position, onClose }: Props) {
  const avatars = useChatStore((s) => s.avatars);
  const avatarUrl = avatars[nickname];
  const [profile, setProfile] = useState<PublicUserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const popupRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getUserProfile(nickname)
      .then(setProfile)
      .catch(() => setProfile(null))
      .finally(() => setLoading(false));
  }, [nickname]);

  // Close on click outside
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (popupRef.current && !popupRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [onClose]);

  // Close on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [onClose]);

  // Clamp position to stay within viewport
  const style: React.CSSProperties = {
    position: 'fixed',
    top: Math.max(8, Math.min(position.top, window.innerHeight - 340)),
    left: Math.max(8, Math.min(position.left, window.innerWidth - 320)),
    zIndex: 50,
  };

  const isAtproto = profile?.provider === 'atproto';
  const did = profile?.provider_id;
  const bskyUrl = did
    ? `https://bsky.app/profile/${did}`
    : null;

  return (
    <div ref={popupRef} style={style} className="w-[300px] overflow-hidden rounded-lg border border-bg-hover bg-bg-primary shadow-xl">
      {/* Banner */}
      <div className="h-16 bg-gradient-to-r from-indigo-600 to-purple-600" />

      {/* Avatar overlapping banner */}
      <div className="relative px-4">
        <div className="-mt-10 mb-2">
          {avatarUrl || profile?.avatar_url ? (
            <img
              src={avatarUrl || profile?.avatar_url || ''}
              alt={nickname}
              className="h-20 w-20 rounded-full border-4 border-bg-primary object-cover"
            />
          ) : (
            <div className="flex h-20 w-20 items-center justify-center rounded-full border-4 border-bg-primary bg-bg-accent text-2xl font-bold text-white">
              {nickname[0]?.toUpperCase() || '?'}
            </div>
          )}
        </div>

        {/* Username */}
        <h3 className="text-lg font-bold text-text-primary">{nickname}</h3>

        {/* Profile info */}
        <div className="mt-2 space-y-2 pb-4">
          {loading && (
            <p className="text-sm text-text-muted">Loading profile...</p>
          )}

          {!loading && isAtproto && did && (
            <>
              <div className="rounded bg-bg-secondary px-3 py-2">
                <p className="text-xs font-semibold uppercase tracking-wide text-text-muted">Bluesky / AT Protocol</p>
                <p className="mt-0.5 break-all text-sm text-text-secondary">{did}</p>
              </div>

              {bskyUrl && (
                <a
                  href={bskyUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-2 rounded bg-blue-600 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700"
                >
                  <svg className="h-4 w-4" viewBox="0 0 600 530" fill="currentColor">
                    <path d="m135.72 44.03c66.496 49.921 138.02 151.14 164.28 205.46 26.262-54.316 97.782-155.54 164.28-205.46 47.98-36.021 125.72-63.892 125.72 24.795 0 17.712-10.155 148.79-16.111 170.07-20.703 73.984-96.144 92.854-163.25 81.433 117.3 19.964 147.14 86.092 82.697 152.22-122.39 125.59-175.91-31.511-189.63-71.766-2.514-7.3797-3.6904-10.832-3.7077-7.8964-0.0174-2.9357-1.1937 0.51669-3.7077 7.8964-13.72 40.255-67.233 197.36-189.63 71.766-64.444-66.128-34.605-132.26 82.697-152.22-67.108 11.421-142.55-7.4491-163.25-81.433-5.9562-21.282-16.111-152.36-16.111-170.07 0-88.687 77.742-60.816 125.72-24.795z" />
                  </svg>
                  View on Bluesky
                </a>
              )}
            </>
          )}

          {!loading && !isAtproto && (
            <div className="rounded bg-bg-secondary px-3 py-2">
              <p className="text-xs font-semibold uppercase tracking-wide text-text-muted">
                {profile?.provider ? profile.provider.charAt(0).toUpperCase() + profile.provider.slice(1) : 'Local'} User
              </p>
            </div>
          )}

          {!loading && !profile && (
            <div className="rounded bg-bg-secondary px-3 py-2">
              <p className="text-sm text-text-muted">No profile information available</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

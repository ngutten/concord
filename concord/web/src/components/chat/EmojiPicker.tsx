import { useState, useRef, useEffect } from 'react';

interface EmojiPickerProps {
  onSelect: (emoji: string) => void;
  onClose: () => void;
}

const EMOJI_CATEGORIES: { name: string; emojis: string[] }[] = [
  {
    name: 'Smileys',
    emojis: [
      '😀', '😃', '😄', '😁', '😆', '😅', '🤣', '😂', '🙂', '😊',
      '😇', '🥰', '😍', '🤩', '😘', '😗', '😚', '😙', '🥲', '😋',
      '😛', '😜', '🤪', '😝', '🤑', '🤗', '🤭', '🤫', '🤔', '🫡',
      '🤐', '🤨', '😐', '😑', '😶', '🫥', '😏', '😒', '🙄', '😬',
      '😮‍💨', '🤥', '😌', '😔', '😪', '🤤', '😴', '😷', '🤒', '🤕',
      '🤢', '🤮', '🥴', '😵', '🤯', '🥳', '🥸', '😎', '🤓', '🧐',
      '😕', '🫤', '😟', '🙁', '😮', '😯', '😲', '😳', '🥺', '🥹',
      '😦', '😧', '😨', '😰', '😥', '😢', '😭', '😱', '😖', '😣',
      '😞', '😓', '😩', '😫', '🥱', '😤', '😡', '😠', '🤬', '😈',
      '👿', '💀', '☠️', '💩', '🤡', '👹', '👺', '👻', '👽', '🤖',
    ],
  },
  {
    name: 'Gestures',
    emojis: [
      '👋', '🤚', '🖐️', '✋', '🖖', '🫱', '🫲', '🫳', '🫴', '🫷',
      '🫸', '👌', '🤌', '🤏', '✌️', '🤞', '🫰', '🤟', '🤘', '🤙',
      '👈', '👉', '👆', '🖕', '👇', '☝️', '🫵', '👍', '👎', '✊',
      '👊', '🤛', '🤜', '👏', '🙌', '🫶', '👐', '🤲', '🤝', '🙏',
      '✍️', '💅', '🤳', '💪', '🦾', '🦿', '🦵', '🦶', '👂', '🦻',
    ],
  },
  {
    name: 'Hearts',
    emojis: [
      '❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '🤍', '🤎', '💔',
      '❤️‍🔥', '❤️‍🩹', '❣️', '💕', '💞', '💓', '💗', '💖', '💘', '💝',
      '💟', '♥️', '💋', '💯', '🔥', '✨', '⭐', '🌟', '💫', '💥',
    ],
  },
  {
    name: 'Objects',
    emojis: [
      '🎉', '🎊', '🎈', '🎁', '🎗️', '🏆', '🥇', '🥈', '🥉', '⚽',
      '🏀', '🏈', '⚾', '🎾', '🎮', '🕹️', '🎲', '🎯', '🎵', '🎶',
      '🎤', '🎧', '📱', '💻', '⌨️', '🖥️', '📷', '📹', '🔒', '🔑',
      '🔨', '🪓', '⚔️', '💣', '🪄', '💊', '🧪', '🔬', '🔭', '📡',
    ],
  },
  {
    name: 'Food',
    emojis: [
      '🍕', '🍔', '🍟', '🌭', '🍿', '🧂', '🥓', '🥚', '🍳', '🧇',
      '🥞', '🧈', '🍞', '🥐', '🥨', '🧀', '🥩', '🍗', '🍖', '🌮',
      '🌯', '🥙', '🍝', '🍜', '🍲', '🍛', '🍣', '🍱', '🥟', '🍤',
      '🍩', '🍪', '🎂', '🍰', '🧁', '🍫', '🍬', '🍭', '🍮', '🍯',
      '🍺', '🍻', '🥂', '🍷', '🍸', '🍹', '☕', '🍵', '🧃', '🥤',
    ],
  },
  {
    name: 'Nature',
    emojis: [
      '🐶', '🐱', '🐭', '🐹', '🐰', '🦊', '🐻', '🐼', '🐻‍❄️', '🐨',
      '🐯', '🦁', '🐮', '🐷', '🐸', '🐵', '🙈', '🙉', '🙊', '🐔',
      '🐧', '🐦', '🦅', '🦆', '🦉', '🐺', '🐗', '🐴', '🦄', '🐝',
      '🌸', '🌹', '🌺', '🌻', '🌼', '🌷', '🌱', '🌲', '🌳', '🍀',
      '🍁', '🍂', '🍃', '🌍', '🌎', '🌏', '🌑', '🌕', '⭐', '🌈',
    ],
  },
];

export function EmojiPicker({ onSelect, onClose }: EmojiPickerProps) {
  const [activeCategory, setActiveCategory] = useState(0);
  const [search, setSearch] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);
  const pickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Close on click outside
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [onClose]);

  const allEmojis = EMOJI_CATEGORIES.flatMap((c) => c.emojis);
  const filtered = search
    ? allEmojis.filter(() => true) // Can't search by name without a mapping, just show all when searching
    : EMOJI_CATEGORIES[activeCategory].emojis;

  return (
    <div
      ref={pickerRef}
      className="absolute bottom-full right-0 mb-1 flex w-80 flex-col overflow-hidden rounded-lg border border-border bg-bg-secondary shadow-lg"
    >
      {/* Search */}
      <div className="border-b border-border px-3 py-2">
        <input
          ref={inputRef}
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search emoji..."
          className="w-full bg-transparent text-sm text-text-primary placeholder-text-muted outline-none"
          onKeyDown={(e) => {
            if (e.key === 'Escape') onClose();
          }}
        />
      </div>

      {/* Category tabs */}
      {!search && (
        <div className="flex border-b border-border px-1">
          {EMOJI_CATEGORIES.map((cat, i) => (
            <button
              key={cat.name}
              onClick={() => setActiveCategory(i)}
              className={`flex-1 px-1 py-1.5 text-center text-xs transition-colors ${
                i === activeCategory
                  ? 'border-b-2 border-blue-400 text-text-primary'
                  : 'text-text-muted hover:text-text-secondary'
              }`}
              title={cat.name}
            >
              {cat.emojis[0]}
            </button>
          ))}
        </div>
      )}

      {/* Emoji grid */}
      <div className="h-52 overflow-y-auto p-2">
        <div className="grid grid-cols-8 gap-0.5">
          {filtered.map((emoji, i) => (
            <button
              key={`${emoji}-${i}`}
              onClick={() => {
                onSelect(emoji);
                onClose();
              }}
              className="flex h-8 w-8 items-center justify-center rounded text-xl transition-colors hover:bg-bg-hover"
              title={emoji}
            >
              {emoji}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

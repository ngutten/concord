import { useState } from 'react';
import { useChatStore } from '../../stores/chatStore';
import { useUiStore } from '../../stores/uiStore';
import type { RoleInfo, CategoryInfo } from '../../api/types';
import { hasPermission, Permissions } from '../../api/types';

const EMPTY_ROLES: RoleInfo[] = [];
const EMPTY_CATEGORIES: CategoryInfo[] = [];

type Tab = 'roles' | 'categories';

export function ServerSettings() {
  const activeServer = useUiStore((s) => s.activeServer);
  const setShowServerSettings = useUiStore((s) => s.setShowServerSettings);
  const roles = useChatStore((s) => (activeServer ? s.roles[activeServer] ?? EMPTY_ROLES : EMPTY_ROLES));
  const categories = useChatStore((s) => (activeServer ? s.categories[activeServer] ?? EMPTY_CATEGORIES : EMPTY_CATEGORIES));
  const servers = useChatStore((s) => s.servers);
  const createRole = useChatStore((s) => s.createRole);
  const updateRole = useChatStore((s) => s.updateRole);
  const deleteRole = useChatStore((s) => s.deleteRole);
  const createCategory = useChatStore((s) => s.createCategory);
  const updateCategory = useChatStore((s) => s.updateCategory);
  const deleteCategory = useChatStore((s) => s.deleteCategory);

  const [tab, setTab] = useState<Tab>('roles');

  const serverName = servers.find((s) => s.id === activeServer)?.name ?? 'Server';

  if (!activeServer) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <div className="max-h-[80vh] w-full max-w-2xl overflow-y-auto rounded-lg bg-bg-secondary p-6">
        <div className="mb-6 flex items-center justify-between">
          <h2 className="text-xl font-bold text-text-primary">{serverName} Settings</h2>
          <button
            onClick={() => setShowServerSettings(false)}
            className="rounded p-1 text-text-muted transition-colors hover:text-text-primary"
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Tab bar */}
        <div className="mb-6 flex gap-1 rounded-lg bg-bg-primary p-1">
          <button
            onClick={() => setTab('roles')}
            className={`flex-1 rounded-md px-4 py-2 text-sm font-medium transition-colors ${
              tab === 'roles' ? 'bg-bg-accent text-white' : 'text-text-muted hover:text-text-primary'
            }`}
          >
            Roles
          </button>
          <button
            onClick={() => setTab('categories')}
            className={`flex-1 rounded-md px-4 py-2 text-sm font-medium transition-colors ${
              tab === 'categories' ? 'bg-bg-accent text-white' : 'text-text-muted hover:text-text-primary'
            }`}
          >
            Categories
          </button>
        </div>

        {tab === 'roles' && (
          <RolesTab
            serverId={activeServer}
            roles={roles}
            createRole={createRole}
            updateRole={updateRole}
            deleteRole={deleteRole}
          />
        )}
        {tab === 'categories' && (
          <CategoriesTab
            serverId={activeServer}
            categories={categories}
            createCategory={createCategory}
            updateCategory={updateCategory}
            deleteCategory={deleteCategory}
          />
        )}
      </div>
    </div>
  );
}

function RolesTab({
  serverId,
  roles,
  createRole,
  updateRole,
  deleteRole,
}: {
  serverId: string;
  roles: RoleInfo[];
  createRole: (serverId: string, name: string, color?: string, permissions?: number) => void;
  updateRole: (serverId: string, roleId: string, updates: { name?: string; color?: string; permissions?: number; position?: number }) => void;
  deleteRole: (serverId: string, roleId: string) => void;
}) {
  const [newName, setNewName] = useState('');
  const [newColor, setNewColor] = useState('#99aab5');
  const [editingRole, setEditingRole] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [editColor, setEditColor] = useState('');

  const sortedRoles = [...roles].sort((a, b) => b.position - a.position);

  const handleCreate = () => {
    if (!newName.trim()) return;
    createRole(serverId, newName.trim(), newColor);
    setNewName('');
  };

  const startEdit = (role: RoleInfo) => {
    setEditingRole(role.id);
    setEditName(role.name);
    setEditColor(role.color || '#99aab5');
  };

  const saveEdit = (roleId: string) => {
    updateRole(serverId, roleId, { name: editName.trim() || undefined, color: editColor });
    setEditingRole(null);
  };

  const permissionLabels: { flag: number; label: string }[] = [
    { flag: Permissions.MANAGE_CHANNELS, label: 'Manage Channels' },
    { flag: Permissions.MANAGE_ROLES, label: 'Manage Roles' },
    { flag: Permissions.MANAGE_SERVER, label: 'Manage Server' },
    { flag: Permissions.MANAGE_MESSAGES, label: 'Manage Messages' },
    { flag: Permissions.KICK_MEMBERS, label: 'Kick Members' },
    { flag: Permissions.BAN_MEMBERS, label: 'Ban Members' },
    { flag: Permissions.MENTION_EVERYONE, label: 'Mention Everyone' },
    { flag: Permissions.ADMINISTRATOR, label: 'Administrator' },
  ];

  return (
    <div>
      {/* Create role */}
      <div className="mb-4 flex gap-2">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="New role name"
          className="flex-1 rounded bg-bg-input px-3 py-2 text-sm text-text-primary placeholder-text-muted outline-none"
          onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
        />
        <input
          type="color"
          value={newColor}
          onChange={(e) => setNewColor(e.target.value)}
          className="h-9 w-9 cursor-pointer rounded border-0 bg-transparent"
        />
        <button
          onClick={handleCreate}
          className="rounded bg-bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-bg-accent-hover"
        >
          Create
        </button>
      </div>

      {/* Role list */}
      <div className="space-y-2">
        {sortedRoles.map((role) => (
          <div key={role.id} className="rounded-md bg-bg-tertiary p-3">
            {editingRole === role.id ? (
              <div className="space-y-3">
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    className="flex-1 rounded bg-bg-input px-3 py-1.5 text-sm text-text-primary outline-none"
                  />
                  <input
                    type="color"
                    value={editColor}
                    onChange={(e) => setEditColor(e.target.value)}
                    className="h-8 w-8 cursor-pointer rounded border-0 bg-transparent"
                  />
                </div>

                {/* Permission toggles */}
                <div className="grid grid-cols-2 gap-2">
                  {permissionLabels.map(({ flag, label }) => {
                    const has = hasPermission(role.permissions, flag);
                    return (
                      <label key={flag} className="flex items-center gap-2 text-sm text-text-secondary">
                        <input
                          type="checkbox"
                          checked={has}
                          onChange={() => {
                            const newPerms = has ? (role.permissions & ~flag) : (role.permissions | flag);
                            updateRole(serverId, role.id, { permissions: newPerms });
                          }}
                          className="rounded"
                          disabled={role.is_default && role.name === '@everyone'}
                        />
                        {label}
                      </label>
                    );
                  })}
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={() => saveEdit(role.id)}
                    className="rounded bg-bg-accent px-3 py-1 text-sm text-white hover:bg-bg-accent-hover"
                  >
                    Save
                  </button>
                  <button
                    onClick={() => setEditingRole(null)}
                    className="rounded px-3 py-1 text-sm text-text-muted hover:text-text-primary"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            ) : (
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div
                    className="h-3 w-3 rounded-full"
                    style={{ backgroundColor: role.color || '#99aab5' }}
                  />
                  <span className="text-sm font-medium text-text-primary">{role.name}</span>
                  {role.is_default && (
                    <span className="rounded bg-bg-primary px-1.5 py-0.5 text-xs text-text-muted">default</span>
                  )}
                  <span className="text-xs text-text-muted">pos: {role.position}</span>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => startEdit(role)}
                    className="rounded px-2 py-1 text-xs text-text-muted hover:text-text-primary"
                  >
                    Edit
                  </button>
                  {!role.is_default && (
                    <button
                      onClick={() => deleteRole(serverId, role.id)}
                      className="rounded px-2 py-1 text-xs text-bg-danger hover:bg-bg-danger/10"
                    >
                      Delete
                    </button>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function CategoriesTab({
  serverId,
  categories,
  createCategory,
  updateCategory,
  deleteCategory,
}: {
  serverId: string;
  categories: CategoryInfo[];
  createCategory: (serverId: string, name: string) => void;
  updateCategory: (serverId: string, categoryId: string, updates: { name?: string; position?: number }) => void;
  deleteCategory: (serverId: string, categoryId: string) => void;
}) {
  const [newName, setNewName] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');

  const sorted = [...categories].sort((a, b) => a.position - b.position);

  const handleCreate = () => {
    if (!newName.trim()) return;
    createCategory(serverId, newName.trim());
    setNewName('');
  };

  const startEdit = (cat: CategoryInfo) => {
    setEditingId(cat.id);
    setEditName(cat.name);
  };

  const saveEdit = (catId: string) => {
    if (editName.trim()) {
      updateCategory(serverId, catId, { name: editName.trim() });
    }
    setEditingId(null);
  };

  return (
    <div>
      {/* Create category */}
      <div className="mb-4 flex gap-2">
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="New category name"
          className="flex-1 rounded bg-bg-input px-3 py-2 text-sm text-text-primary placeholder-text-muted outline-none"
          onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
        />
        <button
          onClick={handleCreate}
          className="rounded bg-bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-bg-accent-hover"
        >
          Create
        </button>
      </div>

      {/* Category list */}
      <div className="space-y-2">
        {sorted.map((cat) => (
          <div key={cat.id} className="flex items-center justify-between rounded-md bg-bg-tertiary p-3">
            {editingId === cat.id ? (
              <div className="flex flex-1 gap-2">
                <input
                  type="text"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  className="flex-1 rounded bg-bg-input px-3 py-1.5 text-sm text-text-primary outline-none"
                  onKeyDown={(e) => e.key === 'Enter' && saveEdit(cat.id)}
                />
                <button
                  onClick={() => saveEdit(cat.id)}
                  className="rounded bg-bg-accent px-3 py-1 text-sm text-white hover:bg-bg-accent-hover"
                >
                  Save
                </button>
                <button
                  onClick={() => setEditingId(null)}
                  className="rounded px-3 py-1 text-sm text-text-muted hover:text-text-primary"
                >
                  Cancel
                </button>
              </div>
            ) : (
              <>
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-text-primary">{cat.name}</span>
                  <span className="text-xs text-text-muted">pos: {cat.position}</span>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => startEdit(cat)}
                    className="rounded px-2 py-1 text-xs text-text-muted hover:text-text-primary"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => deleteCategory(serverId, cat.id)}
                    className="rounded px-2 py-1 text-xs text-bg-danger hover:bg-bg-danger/10"
                  >
                    Delete
                  </button>
                </div>
              </>
            )}
          </div>
        ))}

        {sorted.length === 0 && (
          <p className="py-4 text-center text-sm text-text-muted">No categories yet. Create one to organize your channels.</p>
        )}
      </div>
    </div>
  );
}

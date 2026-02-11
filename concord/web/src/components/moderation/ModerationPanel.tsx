import { useState, useEffect } from 'react';
import { useChatStore } from '../../stores/chatStore';
import type { AuditLogEntry, BanInfo, AutomodRuleInfo } from '../../api/types';

interface Props {
  serverId: string;
  onClose: () => void;
}

type Tab = 'bans' | 'audit' | 'automod';

export function ModerationPanel({ serverId, onClose }: Props) {
  const [tab, setTab] = useState<Tab>('bans');
  const bans = useChatStore(s => s.bans[serverId] ?? []);
  const auditLog = useChatStore(s => s.auditLog[serverId] ?? []);
  const automodRules = useChatStore(s => s.automodRules[serverId] ?? []);
  const listBans = useChatStore(s => s.listBans);
  const getAuditLog = useChatStore(s => s.getAuditLog);
  const listAutomodRules = useChatStore(s => s.listAutomodRules);
  const unbanMember = useChatStore(s => s.unbanMember);
  const deleteAutomodRule = useChatStore(s => s.deleteAutomodRule);

  useEffect(() => {
    listBans(serverId);
    getAuditLog(serverId);
    listAutomodRules(serverId);
  }, [serverId, listBans, getAuditLog, listAutomodRules]);

  // Close on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [onClose]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      <div className="w-full max-w-2xl max-h-[80vh] flex flex-col rounded-lg bg-bg-primary shadow-xl" onClick={e => e.stopPropagation()}>
        <div className="flex items-center justify-between border-b border-border p-4">
          <h2 className="text-lg font-bold text-text-primary">Moderation</h2>
          <button onClick={onClose} className="text-text-muted hover:text-text-primary">&times;</button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-border">
          {(['bans', 'audit', 'automod'] as Tab[]).map(t => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`px-4 py-2 text-sm font-medium capitalize ${
                tab === t ? 'border-b-2 border-bg-accent text-text-primary' : 'text-text-muted hover:text-text-secondary'
              }`}
            >
              {t === 'audit' ? 'Audit Log' : t === 'automod' ? 'AutoMod' : 'Bans'}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {tab === 'bans' && (
            <BanListTab bans={bans} serverId={serverId} onUnban={unbanMember} />
          )}
          {tab === 'audit' && (
            <AuditLogTab entries={auditLog} />
          )}
          {tab === 'automod' && (
            <AutomodTab rules={automodRules} serverId={serverId} onDelete={deleteAutomodRule} />
          )}
        </div>
      </div>
    </div>
  );
}

function BanListTab({ bans, serverId, onUnban }: { bans: BanInfo[]; serverId: string; onUnban: (serverId: string, userId: string) => void }) {
  if (bans.length === 0) {
    return <p className="text-text-muted text-sm">No bans.</p>;
  }
  return (
    <div className="space-y-2">
      {bans.map(ban => (
        <div key={ban.id} className="flex items-center justify-between rounded bg-bg-secondary p-3">
          <div>
            <span className="text-sm font-medium text-text-primary">User: {ban.user_id}</span>
            {ban.reason && <p className="text-xs text-text-muted">Reason: {ban.reason}</p>}
            <p className="text-xs text-text-muted">Banned by: {ban.banned_by} on {new Date(ban.created_at).toLocaleDateString()}</p>
          </div>
          <button
            onClick={() => onUnban(serverId, ban.user_id)}
            className="rounded bg-red-600 px-3 py-1 text-xs font-medium text-white hover:bg-red-700"
          >
            Unban
          </button>
        </div>
      ))}
    </div>
  );
}

function AuditLogTab({ entries }: { entries: AuditLogEntry[] }) {
  if (entries.length === 0) {
    return <p className="text-text-muted text-sm">No audit log entries.</p>;
  }

  const actionLabels: Record<string, string> = {
    member_kick: 'Kicked',
    member_ban: 'Banned',
    member_unban: 'Unbanned',
    member_timeout: 'Timed out',
  };

  return (
    <div className="space-y-2">
      {entries.map(entry => (
        <div key={entry.id} className="rounded bg-bg-secondary p-3">
          <div className="flex items-center gap-2">
            <span className="rounded bg-bg-accent/20 px-2 py-0.5 text-xs font-medium text-bg-accent">
              {actionLabels[entry.action_type] ?? entry.action_type}
            </span>
            <span className="text-xs text-text-muted">
              by {entry.actor_id}
            </span>
            <span className="ml-auto text-xs text-text-muted">
              {new Date(entry.created_at).toLocaleString()}
            </span>
          </div>
          {entry.target_id && (
            <p className="mt-1 text-xs text-text-secondary">Target: {entry.target_id}</p>
          )}
          {entry.reason && (
            <p className="mt-1 text-xs text-text-muted">Reason: {entry.reason}</p>
          )}
        </div>
      ))}
    </div>
  );
}

function AutomodTab({ rules, serverId, onDelete }: { rules: AutomodRuleInfo[]; serverId: string; onDelete: (serverId: string, ruleId: string) => void }) {
  if (rules.length === 0) {
    return <p className="text-text-muted text-sm">No automod rules configured.</p>;
  }
  return (
    <div className="space-y-2">
      {rules.map(rule => (
        <div key={rule.id} className="flex items-center justify-between rounded bg-bg-secondary p-3">
          <div>
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-text-primary">{rule.name}</span>
              <span className={`rounded px-1.5 py-0.5 text-xs ${rule.enabled ? 'bg-green-600/20 text-green-400' : 'bg-gray-600/20 text-gray-400'}`}>
                {rule.enabled ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <p className="text-xs text-text-muted">Type: {rule.rule_type} | Action: {rule.action_type}</p>
          </div>
          <button
            onClick={() => onDelete(serverId, rule.id)}
            className="rounded bg-red-600 px-3 py-1 text-xs font-medium text-white hover:bg-red-700"
          >
            Delete
          </button>
        </div>
      ))}
    </div>
  );
}

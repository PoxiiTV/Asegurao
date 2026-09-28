import { invoke } from '@tauri-apps/api/core';

export type AuthKind = 'pin' | 'password';

export interface Intruder {
  enabled: boolean;
  after_fails: number;
}
export interface Settings {
  theme: string;
  autolock_minutes: number;
  level: string;
  intruder: Intruder;
  start_with_windows: boolean;
}
export interface AppView {
  display: string;
  exe: string;
  path: string;
  icon: string;
  auth_kind: string; // 'own' | 'master'
  trust_minutes: number;
  enabled: boolean;
}
export interface Snapshot {
  configured: boolean;
  auth_kind: string;
  settings: Settings;
  apps: AppView[];
}
export interface AppInfo {
  display: string;
  exe: string;
  path: string;
  icon: string;
}
export interface HistoryEntry {
  ts: number;
  exe: string;
  display: string;
  result: string; // 'allowed' | 'denied' | 'tamper'
  photo: string;
}

export const api = {
  snapshot: () => invoke<Snapshot>('get_snapshot'),
  sessionStatus: () => invoke<boolean>('session_status'),
  touch: () => invoke('touch'),
  lockSession: () => invoke('lock_session'),

  setup: (kind: AuthKind, secret: string) => invoke<string>('setup', { kind, secret }),
  login: (secret: string) => invoke<boolean>('login', { secret }),
  recover: (code: string, kind: AuthKind, newSecret: string) =>
    invoke<string>('recover', { code, kind, newSecret }),
  changeMaster: (current: string, kind: AuthKind, newSecret: string) =>
    invoke('change_master', { current, kind, newSecret }),

  setTheme: (theme: string) => invoke('set_theme', { theme }),
  saveSettings: (settings: Settings) => invoke('save_settings', { settings }),
  setAutostart: (on: boolean) => invoke('set_autostart', { on }),

  listInstalled: () => invoke<AppInfo[]>('list_installed'),
  listRunning: () => invoke<AppInfo[]>('list_running'),
  appFromPath: (path: string) => invoke<AppInfo>('app_from_path', { path }),

  addApp: (a: {
    display: string;
    exe: string;
    path: string;
    icon: string;
    authKind: string;
    appSecret: string;
    trustMinutes: number;
  }) => invoke('add_app', a),
  setAppTrust: (exe: string, minutes: number) => invoke('set_app_trust', { exe, minutes }),
  changeAppPassword: (exe: string, authKind: string, appSecret: string) =>
    invoke('change_app_password', { exe, authKind, appSecret }),
  setAppEnabled: (exe: string, enabled: boolean, master: string) =>
    invoke('set_app_enabled', { exe, enabled, master }),
  removeApp: (exe: string, master: string) => invoke('remove_app', { exe, master }),

  getHistory: () => invoke<HistoryEntry[]>('get_history'),
  clearHistory: (master: string) => invoke('clear_history', { master }),

  unlockAttempt: (exe: string, secret: string) =>
    invoke<string>('unlock_attempt', { exe, secret }),
  unlockCancel: (exe: string) => invoke('unlock_cancel', { exe }),
  unlockPending: <T>() => invoke<T | null>('unlock_pending'),
  quitApp: (master: string) => invoke('quit_app', { master }),
  prepareUpdate: () => invoke('prepare_update')
};

export function applyTheme(theme: string) {
  document.documentElement.setAttribute('data-theme', theme || 'grafito');
}
